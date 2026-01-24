#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Создание датасетов РЖД в Superset через базу данных.
"""

import sqlite3
from pathlib import Path
import json
from datetime import datetime

SCRIPT_DIR = Path(__file__).parent
ROOT_DIR = SCRIPT_DIR.parent
SUPERSET_HOME = ROOT_DIR / "superset_home"
DB_PATH = SUPERSET_HOME / "superset.db"

def get_main_db_id(cursor):
    """Получить ID основной базы данных"""
    cursor.execute("SELECT id FROM dbs WHERE database_name = 'examples' OR database_name = 'main' LIMIT 1")
    row = cursor.fetchone()
    if row:
        return row[0]
    # Создать новую базу если не существует
    cursor.execute("""
        INSERT INTO dbs (database_name, sqlalchemy_uri, cache_timeout, expose_in_sqllab, allow_run_async, 
                         allow_ctas, allow_cvas, allow_dml, force_ctas_schema, extra, uuid)
        VALUES ('examples', 'sqlite:///superset_home/superset.db', NULL, 1, 0, 0, 0, 0, '', '{}', ?)
    """, (str(datetime.now().timestamp()),))
    return cursor.lastrowid

def create_dataset(cursor, db_id, table_name, sql_table_name):
    """Создать датасет если не существует"""
    cursor.execute("SELECT id FROM tables WHERE table_name = ?", (table_name,))
    if cursor.fetchone():
        print(f"[SKIP] Dataset {table_name} already exists")
        return
    
    now = datetime.now().isoformat()
    cursor.execute("""
        INSERT INTO tables (table_name, schema, database_id, sql, is_sqllab_view, 
                           created_on, changed_on, uuid)
        VALUES (?, '', ?, NULL, 0, ?, ?, ?)
    """, (table_name, db_id, now, now, str(datetime.now().timestamp())))
    print(f"[OK] Created dataset: {table_name}")
    return cursor.lastrowid

def main():
    print("=" * 50)
    print("  Registering RZD Datasets in Superset")
    print("=" * 50)
    
    conn = sqlite3.connect(str(DB_PATH))
    cursor = conn.cursor()
    
    try:
        db_id = get_main_db_id(cursor)
        print(f"[OK] Using database ID: {db_id}")
        
        # Создать датасеты
        create_dataset(cursor, db_id, "rzd_stations", "rzd_stations")
        create_dataset(cursor, db_id, "rzd_monthly_stats", "rzd_monthly_stats")
        create_dataset(cursor, db_id, "rzd_cargo_types", "rzd_cargo_types")
        
        conn.commit()
        print()
        print("[OK] Datasets registered!")
        print()
        print("Open http://localhost:8088")
        print("Go to: Data -> Datasets")
        print("You should see: rzd_stations, rzd_monthly_stats, rzd_cargo_types")
        
    finally:
        conn.close()

if __name__ == "__main__":
    main()
