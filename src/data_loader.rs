//! High-performance data loader module
//! 
//! Uses `polars` for fast reading and schema inference.
//! Writes to SQLite using batch transactions.

use anyhow::{Context, Result, anyhow};
use polars::prelude::*;
use rusqlite::Connection;
use std::path::Path;
use tracing::info;
use std::fs::File;

/// Load a file (Excel or CSV) into the SQLite database
pub fn load_file(file_path: &Path, table_name: &str, db_path: &Path) -> Result<String> {
    info!("🚀 Loading data from: {}", file_path.display());
    
    // Detect extension
    let ext = file_path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
        
    let conn = Connection::open(db_path)
        .context("Failed to open database")?;
        
    // Use Polars to read file into DataFrame
    let df = match ext.as_str() {
        "csv" => {
             CsvReader::from_path(file_path)?
                .has_header(true)
                .finish()?
        },
        // Polars doesn't support generic Excel reading easily without feature flags or extra crates properly set up
        // usually people use `polars-excel` or just `calamine` to build df.
        // For this plan, we'll keep it simple: Use CsvReader for CSV (super fast)
        // And for Excel, we stick to calamine manually or convert to generic.
        // However, let's try to use internal logic if possible.
        // Since we are in `superset-launcher`, let's optimize CSV first.
        // For Excel, we will simple failback to manual `calamine` if polars excel is not enabled, 
        // but wait, we added `calamine` dependency.
        // Let's implement a helper for Excel -> DataFrame if needed, or just write directly.
        // The plan requested Polars.
        "xlsx" | "xls" | "xlsb" => {
            // For now, let's use the manual loader for Excel as it's robust,
            // or we could convert to CSV then load with Polars? No, that's double work.
            // Let's just stick to the manual implementation for Excel for now,
            // as Polars Excel support requires `connector-arrow` or specific features we might not have enabled fully.
            // ACTUALLY: Let's use our manual loader for Excel but optimized.
            return legacy_load_excel(file_path, table_name, &conn);
        }
        _ => return Err(anyhow!("Unsupported file extension: {}", ext)),
    };

    info!("📊 Schema detected: {:?}", df.schema());
    let rows_count = df.height();
    
    // Write DF to SQLite
    write_df_to_sqlite(&df, table_name, &conn)?;
    
    info!("✅ Loaded {} rows into table '{}'", rows_count, table_name);
    Ok(format!("Successfully loaded {} rows into {}", rows_count, table_name))
}

fn write_df_to_sqlite(df: &DataFrame, table_name: &str, conn: &Connection) -> Result<()> {
    // 1. Create table based on DataFrame columns
    let columns = df.get_columns();
    let has_id = columns.iter().any(|c| c.name() == "id");
    
    let mut field_defs = Vec::new();
    
    if !has_id {
        field_defs.push("id INTEGER PRIMARY KEY AUTOINCREMENT".to_string());
    }
    
    for c in columns.iter() {
        let name = c.name();
        let dtype = c.dtype();
        let sql_type = match dtype {
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "INTEGER",
            DataType::Float32 | DataType::Float64 => "REAL",
            DataType::String => "TEXT",
            DataType::Boolean => "INTEGER",
            _ => "TEXT", // Fallback
        };
        // If it's the ID column, make it Primary Key if it's integer?
        // relying on user data for PK is risky if not unique. 
        // But for "id" collision, let's just let it be a normal column if it exists.
        // Or if the user provided "id", maybe they want it to be the ID.
        // For simplicity: If "id" exists, we don't add our own. We just treat "id" as a normal column (SQLite auto-rowid handles internal storage).
        // If they want it to be PK, they'd need schema inference to be smarter.
        field_defs.push(format!("\"{}\" {}", name, sql_type));
    }
    
    let fields_sql = field_defs.join(", ");
    
    conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), [])?;
    let create_sql = format!("CREATE TABLE {} ({})", table_name, fields_sql);
    conn.execute(&create_sql, [])?;
    
    // 2. Insert data
    conn.execute("BEGIN TRANSACTION", [])?;
    
    let n_rows = df.height();
    let n_cols = columns.len();
    
    // Prepare statement
    let placeholders = (0..n_cols).map(|_| "?").collect::<Vec<_>>().join(", ");
    let col_names = columns.iter().map(|c| format!("\"{}\"", c.name())).collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, col_names, placeholders);
    
    let mut stmt = conn.prepare(&insert_sql)?;
    
    // Iterate rows
    for i in 0..n_rows {
        let mut params = Vec::with_capacity(n_cols);
        for col in columns {
             // col.get(i) returns AnyValue, not Result
             let val = col.get(i).unwrap(); 
             params.push(val_to_sql_param(val));
        }
        
        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.execute(&*params_ref)?;
    }
    
    conn.execute("COMMIT", [])?;
    
    Ok(())
}

fn val_to_sql_param(val: AnyValue) -> Box<dyn rusqlite::ToSql> {
    match val {
        AnyValue::Int8(v) => Box::new(v as i64),
        AnyValue::Int16(v) => Box::new(v as i64),
        AnyValue::Int32(v) => Box::new(v as i64),
        AnyValue::Int64(v) => Box::new(v),
        AnyValue::UInt8(v) => Box::new(v as i64),
        AnyValue::UInt16(v) => Box::new(v as i64),
        AnyValue::UInt32(v) => Box::new(v as i64),
        AnyValue::UInt64(v) => Box::new(v as i64),
        AnyValue::Float32(v) => Box::new(v as f64),
        AnyValue::Float64(v) => Box::new(v),
        AnyValue::String(v) => Box::new(v.to_string()),
        AnyValue::StringOwned(v) => Box::new(v.to_string()),
        AnyValue::Boolean(v) => Box::new(v),
        AnyValue::Null => Box::new(Option::<String>::None),
        _ => Box::new(val.to_string()),
    }
}

/// Fallback for Excel using Calamine (Polars Excel reader is optional/heavy)
fn legacy_load_excel(file_path: &Path, table_name: &str, conn: &Connection) -> Result<String> {
    use calamine::{Reader, open_workbook, Data, Xlsx};
    
    let mut workbook: Xlsx<std::io::BufReader<std::fs::File>> = open_workbook(file_path)
        .context("Cannot open Excel file")?;
        
    let sheet_name = workbook.sheet_names().first()
        .ok_or_else(|| anyhow!("No sheets in workbook"))?
        .to_owned();
        
    let range = workbook.worksheet_range(&sheet_name)
        .context("Cannot read sheet")?;
        
    let mut rows = range.rows();
    
    let headers: Vec<String> = rows.next()
        .ok_or_else(|| anyhow!("Empty file"))?
        .iter()
        .map(|c| c.to_string())
        .collect();
        
    // Create table (legacy string-based)
    conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), [])?;
    let columns = headers.iter().map(|h| format!("\"{}\" TEXT", h)).collect::<Vec<_>>().join(", ");
    conn.execute(&format!("CREATE TABLE {} (id INTEGER PRIMARY KEY AUTOINCREMENT, {})", table_name, columns), [])?;
    
    conn.execute("BEGIN TRANSACTION", [])?;
    
    let placeholders = headers.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let columns_sql = headers.iter().map(|h| format!("\"{}\"", h)).collect::<Vec<_>>().join(", ");
    let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, columns_sql, placeholders);
    let mut stmt = conn.prepare(&sql)?;
    
    let mut count = 0;
    for row in rows {
        let params: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        stmt.execute(&*params_ref)?;
        count += 1;
    }
    
    conn.execute("COMMIT", [])?;
    
    Ok(format!("Successfully loaded {} rows into {} (Legacy Excel Mode)", count, table_name))
}
