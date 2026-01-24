#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Скрипт автоматического создания демо-дашборда РЖД в Superset.

Запуск:
    python setup_rzd_dashboard.py

Требования:
    - Superset должен быть запущен на http://localhost:8088
    - Пользователь admin:admin
"""

import os
import sys
import csv
import json
import sqlite3
from pathlib import Path

# Пути
SCRIPT_DIR = Path(__file__).parent
ROOT_DIR = SCRIPT_DIR.parent
SUPERSET_HOME = ROOT_DIR / "superset_home"
DB_PATH = SUPERSET_HOME / "superset.db"
DEMO_DATA_DIR = ROOT_DIR / "docs" / "demo_data"

def load_csv(filename):
    """Загрузка CSV файла"""
    filepath = DEMO_DATA_DIR / filename
    with open(filepath, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return list(reader)

def create_rzd_tables(conn):
    """Создание таблиц с данными РЖД"""
    cursor = conn.cursor()
    
    # Таблица станций
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS rzd_stations (
            id INTEGER PRIMARY KEY,
            name TEXT,
            city TEXT,
            region TEXT,
            latitude REAL,
            longitude REAL,
            passengers_day INTEGER,
            cargo_tons_year INTEGER,
            railway_branch TEXT,
            station_class INTEGER
        )
    ''')
    
    # Таблица статистики
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS rzd_monthly_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month INTEGER,
            year INTEGER,
            passengers_mln REAL,
            cargo_mln_tons REAL,
            revenue_bln_rub REAL,
            on_time_pct REAL
        )
    ''')
    
    # Таблица грузов
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS rzd_cargo_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cargo_type TEXT,
            volume_mln_tons REAL,
            share_pct REAL,
            revenue_bln_rub REAL
        )
    ''')
    
    conn.commit()
    print("[OK] Tablicy sozdany")

def import_data(conn):
    """Импорт данных из CSV"""
    cursor = conn.cursor()
    
    # Очистка таблиц
    cursor.execute("DELETE FROM rzd_stations")
    cursor.execute("DELETE FROM rzd_monthly_stats")
    cursor.execute("DELETE FROM rzd_cargo_types")
    
    # Импорт станций
    stations = load_csv("rzd_stations.csv")
    for row in stations:
        cursor.execute('''
            INSERT INTO rzd_stations 
            (id, name, city, region, latitude, longitude, passengers_day, cargo_tons_year, railway_branch, station_class)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            row['id'], row['name'], row['city'], row['region'],
            float(row['latitude']), float(row['longitude']),
            int(row['passengers_day']), int(row['cargo_tons_year']),
            row['railway_branch'], int(row['station_class'])
        ))
    print(f"[OK] Imported {len(stations)} stations")
    
    # Импорт статистики
    stats = load_csv("rzd_monthly_stats.csv")
    for row in stats:
        cursor.execute('''
            INSERT INTO rzd_monthly_stats 
            (month, year, passengers_mln, cargo_mln_tons, revenue_bln_rub, on_time_pct)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (
            int(row['month']), int(row['year']),
            float(row['passengers_mln']), float(row['cargo_mln_tons']),
            float(row['revenue_bln_rub']), float(row['on_time_pct'])
        ))
    print(f"[OK] Imported {len(stats)} monthly stats")
    
    # Импорт грузов
    cargo = load_csv("rzd_cargo_types.csv")
    for row in cargo:
        cursor.execute('''
            INSERT INTO rzd_cargo_types 
            (cargo_type, volume_mln_tons, share_pct, revenue_bln_rub)
            VALUES (?, ?, ?, ?)
        ''', (
            row['cargo_type'],
            float(row['volume_mln_tons']), float(row['share_pct']),
            float(row['revenue_bln_rub'])
        ))
    print(f"[OK] Imported {len(cargo)} cargo types")
    
    conn.commit()

def main():
    print("=" * 50)
    print("  RZD Demo Dashboard Setup")
    print("=" * 50)
    print()
    
    # Проверка существования файлов
    if not DB_PATH.exists():
        print(f"[ERROR] Database not found: {DB_PATH}")
        print("   Please start Superset first!")
        sys.exit(1)
    
    if not DEMO_DATA_DIR.exists():
        print(f"[ERROR] Data folder not found: {DEMO_DATA_DIR}")
        sys.exit(1)
    
    # Подключение к базе
    conn = sqlite3.connect(str(DB_PATH))
    
    try:
        create_rzd_tables(conn)
        import_data(conn)
        
        print()
        print("=" * 50)
        print("[OK] Data imported successfully!")
        print()
        print("Next steps:")
        print("1. Open http://localhost:8088")
        print("2. Go to Data -> Datasets")
        print("3. Click + Dataset")
        print("4. Select examples -> rzd_stations")
        print("5. Create charts and dashboard")
        print("=" * 50)
        
    finally:
        conn.close()

if __name__ == "__main__":
    main()
