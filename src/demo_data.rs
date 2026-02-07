//! Demo data import module for RZD analytics data
//! 
//! Imports CSV data into SQLite database for offline Superset dashboards.
//! Designed for air-gapped networks without internet access.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// Import all RZD demo data into the examples database
pub fn import_demo_data(root: &Path) -> Result<()> {
    let examples_db = root.join("examples.db");
    let demo_data_dir = root.join("docs").join("demo_data");
    
    println!("📦 Импорт демо-данных РЖД...");
    println!("   База: {}", examples_db.display());
    println!("   Данные: {}", demo_data_dir.display());
    
    // Open or create the database
    let conn = Connection::open(&examples_db)
        .context("Не удалось открыть базу данных examples.db")?;
    
    // Create tables
    create_tables(&conn)?;
    
    // Import data from CSV files
    let files = [
        ("rzd_stations_full.csv", import_stations as fn(&Connection, &Path) -> Result<()>),
        ("rzd_stations.csv", import_stations), // Fallback if full not found
        ("rzd_routes.csv", import_routes),
        ("rzd_monthly_stats.csv", import_monthly_stats),
        ("rzd_cargo_types.csv", import_cargo_types),
        ("rzd_daily_operations.csv", import_daily_operations),
        ("rzd_incidents.csv", import_incidents),
        ("rzd_kpi_metrics.csv", import_kpi_metrics),
    ];
    
    for (filename, import_fn) in files {
        let csv_path = demo_data_dir.join(filename);
        if csv_path.exists() {
            import_fn(&conn, &csv_path)?;
        } else {
            println!("   ⚠️ Файл не найден: {}", csv_path.display());
        }
    }
    
    println!("✅ Импорт завершён!");
    Ok(())
}

/// Create RZD tables if they don't exist
fn create_tables(conn: &Connection) -> Result<()> {
    println!("   📋 Создание таблиц...");
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_stations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            city TEXT,
            region TEXT,
            latitude REAL,
            longitude REAL,
            passengers_day INTEGER,
            cargo_tons_year INTEGER,
            railway_branch TEXT,
            station_class INTEGER
        )",
        [],
    ).context("Ошибка создания таблицы rzd_stations")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_monthly_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month INTEGER NOT NULL,
            year INTEGER NOT NULL,
            passengers_mln REAL,
            cargo_mln_tons REAL,
            revenue_bln_rub REAL,
            on_time_pct REAL
        )",
        [],
    ).context("Ошибка создания таблицы rzd_monthly_stats")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_cargo_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cargo_type TEXT NOT NULL,
            volume_mln_tons REAL,
            share_pct REAL,
            revenue_bln_rub REAL
        )",
        [],
    ).context("Ошибка создания таблицы rzd_cargo_types")?;
    
    // New tables for comprehensive analytics
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_daily_operations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            region TEXT,
            route_type TEXT,
            passengers_thousands REAL,
            cargo_tons_thousands REAL,
            revenue_mln_rub REAL,
            avg_speed_kmh REAL,
            delay_minutes INTEGER,
            trains_count INTEGER,
            occupancy_pct REAL
        )",
        [],
    ).context("Ошибка создания таблицы rzd_daily_operations")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_incidents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            incident_id TEXT NOT NULL,
            date TEXT,
            time TEXT,
            region TEXT,
            railway_branch TEXT,
            incident_type TEXT,
            severity TEXT,
            duration_minutes INTEGER,
            affected_trains INTEGER,
            resolved TEXT,
            cause TEXT,
            description TEXT
        )",
        [],
    ).context("Ошибка создания таблицы rzd_incidents")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_kpi_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER,
            quarter TEXT,
            metric_name TEXT,
            metric_value REAL,
            unit TEXT,
            yoy_change_pct REAL,
            target_value REAL,
            target_met TEXT
        )",
        [],
    ).context("Ошибка создания таблицы rzd_kpi_metrics")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_kpi_metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER,
            quarter TEXT,
            metric_name TEXT,
            metric_value REAL,
            unit TEXT,
            yoy_change_pct REAL,
            target_value REAL,
            target_met TEXT
        )",
        [],
    ).context("Ошибка создания таблицы rzd_kpi_metrics")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS rzd_routes (
            id INTEGER PRIMARY KEY,
            origin_id INTEGER,
            origin_name TEXT,
            dest_id INTEGER,
            dest_name TEXT,
            distance_km REAL,
            distance_km REAL,
            trains_per_day INTEGER,
            geometry TEXT
        )",
        [],
    ).context("Ошибка создания таблицы rzd_routes")?;
    
    Ok(())
}

fn import_stations(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   🚉 Импорт станций ({})", csv_path.file_name().unwrap_or_default().to_string_lossy());
    
    // Clear existing data only if importing full dataset or if table is empty
    let count: i32 = conn.query_row("SELECT count(*) FROM rzd_stations", [], |row| row.get(0)).unwrap_or(0);
    if count > 0 && csv_path.file_name().unwrap().to_string_lossy() == "rzd_stations.csv" {
         // If we already have data (likely from full dataset), skip the small one
         println!("      Пропуск rzd_stations.csv так как данные уже есть");
         return Ok(());
    }
    
    conn.execute("DELETE FROM rzd_stations", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла станций")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        conn.execute(
            "INSERT INTO rzd_stations 
             (id, name, city, region, latitude, longitude, passengers_day, cargo_tons_year, railway_branch, station_class)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                record.get(0).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(1).unwrap_or(""),
                record.get(2).unwrap_or(""),
                record.get(3).unwrap_or(""),
                record.get(4).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(5).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(6).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(7).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(8).unwrap_or(""),
                record.get(9).unwrap_or("0").parse::<i32>().unwrap_or(0),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано станций: {}", count);
    Ok(())
}

/// Import monthly statistics from CSV
fn import_monthly_stats(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   📊 Импорт месячной статистики...");
    
    conn.execute("DELETE FROM rzd_monthly_stats", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла статистики")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        // Skip empty rows
        if record.len() < 6 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_monthly_stats 
             (month, year, passengers_mln, cargo_mln_tons, revenue_bln_rub, on_time_pct)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                record.get(0).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(1).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(2).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(3).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(4).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(5).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано записей: {}", count);
    Ok(())
}

/// Import cargo types from CSV
fn import_cargo_types(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   📦 Импорт типов грузов...");
    
    conn.execute("DELETE FROM rzd_cargo_types", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла грузов")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        // Skip empty rows
        if record.len() < 4 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_cargo_types 
             (cargo_type, volume_mln_tons, share_pct, revenue_bln_rub)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                record.get(0).unwrap_or(""),
                record.get(1).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(2).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(3).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано типов: {}", count);
    Ok(())
}

/// Import daily operations from CSV
fn import_daily_operations(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   📈 Импорт ежедневных операций...");
    
    conn.execute("DELETE FROM rzd_daily_operations", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла операций")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        if record.len() < 10 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_daily_operations 
             (date, region, route_type, passengers_thousands, cargo_tons_thousands, 
              revenue_mln_rub, avg_speed_kmh, delay_minutes, trains_count, occupancy_pct)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                record.get(0).unwrap_or(""),
                record.get(1).unwrap_or(""),
                record.get(2).unwrap_or(""),
                record.get(3).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(4).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(5).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(6).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(7).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(8).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(9).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано операций: {}", count);
    Ok(())
}

/// Import incidents from CSV
fn import_incidents(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   ⚠️ Импорт инцидентов...");
    
    conn.execute("DELETE FROM rzd_incidents", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла инцидентов")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        if record.len() < 12 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_incidents 
             (incident_id, date, time, region, railway_branch, incident_type, 
              severity, duration_minutes, affected_trains, resolved, cause, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                record.get(0).unwrap_or(""),
                record.get(1).unwrap_or(""),
                record.get(2).unwrap_or(""),
                record.get(3).unwrap_or(""),
                record.get(4).unwrap_or(""),
                record.get(5).unwrap_or(""),
                record.get(6).unwrap_or(""),
                record.get(7).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(8).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(9).unwrap_or(""),
                record.get(10).unwrap_or(""),
                record.get(11).unwrap_or(""),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано инцидентов: {}", count);
    Ok(())
}

/// Import KPI metrics from CSV
fn import_kpi_metrics(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   📊 Импорт KPI метрик...");
    
    conn.execute("DELETE FROM rzd_kpi_metrics", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла KPI")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        if record.len() < 8 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_kpi_metrics 
             (year, quarter, metric_name, metric_value, unit, yoy_change_pct, target_value, target_met)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                record.get(0).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(1).unwrap_or(""),
                record.get(2).unwrap_or(""),
                record.get(3).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(4).unwrap_or(""),
                record.get(5).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(6).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(7).unwrap_or(""),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано KPI: {}", count);
    Ok(())
}

/// Import routes from CSV
fn import_routes(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   🛤️ Импорт маршрутов...");
    
    conn.execute("DELETE FROM rzd_routes", [])?;
    
    let mut rdr = csv::Reader::from_path(csv_path)
        .context("Ошибка чтения CSV файла маршрутов")?;
    
    let mut count = 0;
    for result in rdr.records() {
        let record = result?;
        
        if record.len() < 8 || record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        
        conn.execute(
            "INSERT INTO rzd_routes 
             (id, origin_id, origin_name, dest_id, dest_name, distance_km, trains_per_day, geometry)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                record.get(0).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(1).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(2).unwrap_or(""),
                record.get(3).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(4).unwrap_or(""),
                record.get(5).unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                record.get(6).unwrap_or("0").parse::<i32>().unwrap_or(0),
                record.get(7).unwrap_or(""),
            ],
        )?;
        count += 1;
    }
    
    println!("      Импортировано маршрутов: {}", count);
    Ok(())
}
