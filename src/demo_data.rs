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
    let stations_csv = demo_data_dir.join("rzd_stations.csv");
    let monthly_csv = demo_data_dir.join("rzd_monthly_stats.csv");
    let cargo_csv = demo_data_dir.join("rzd_cargo_types.csv");
    
    if stations_csv.exists() {
        import_stations(&conn, &stations_csv)?;
    } else {
        println!("   ⚠️ Файл не найден: {}", stations_csv.display());
    }
    
    if monthly_csv.exists() {
        import_monthly_stats(&conn, &monthly_csv)?;
    } else {
        println!("   ⚠️ Файл не найден: {}", monthly_csv.display());
    }
    
    if cargo_csv.exists() {
        import_cargo_types(&conn, &cargo_csv)?;
    } else {
        println!("   ⚠️ Файл не найден: {}", cargo_csv.display());
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
    
    Ok(())
}

/// Import stations from CSV
fn import_stations(conn: &Connection, csv_path: &Path) -> Result<()> {
    println!("   🚉 Импорт станций...");
    
    // Clear existing data
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
