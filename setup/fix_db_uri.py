#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Fix examples.db URI in superset.db to match the current installation path.

Run this BEFORE starting Superset to ensure the database connection works
regardless of where the portable release was extracted.

Usage:
    python\python.exe setup\fix_db_uri.py
"""

import os
import sqlite3
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent
SUPERSET_DB = ROOT_DIR / "superset_home" / "superset.db"
EXAMPLES_DB = ROOT_DIR / "examples.db"


def fix_uri():
    if not SUPERSET_DB.exists():
        print(f"[ERROR] superset.db not found: {SUPERSET_DB}")
        return False

    if not EXAMPLES_DB.exists():
        print(f"[WARN] examples.db not found: {EXAMPLES_DB}")
        print("       Dashboard demo data will not be available.")

    # Build absolute URI with forward slashes (SQLAlchemy requirement)
    abs_path = str(EXAMPLES_DB).replace("\\", "/")
    new_uri = f"sqlite:///{abs_path}"

    conn = sqlite3.connect(str(SUPERSET_DB))
    cur = conn.cursor()

    try:
        # Check current URI
        cur.execute(
            "SELECT id, sqlalchemy_uri FROM dbs WHERE database_name = 'examples'"
        )
        row = cur.fetchone()

        if not row:
            print("[INFO] No 'examples' database registered — nothing to fix.")
            return True

        db_id, current_uri = row

        if current_uri == new_uri:
            print(f"[OK] URI already correct: {new_uri}")
            return True

        # Update
        cur.execute(
            "UPDATE dbs SET sqlalchemy_uri = ? WHERE id = ?",
            (new_uri, db_id),
        )
        conn.commit()
        print(f"[OK] Updated examples DB URI")
        print(f"     Old: {current_uri}")
        print(f"     New: {new_uri}")
        return True

    except Exception as e:
        print(f"[ERROR] {e}")
        conn.rollback()
        return False
    finally:
        conn.close()


if __name__ == "__main__":
    success = fix_uri()
    sys.exit(0 if success else 1)
