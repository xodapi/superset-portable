#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
=============================================================
  Создание демо-дашборда «РЖД Аналитика» для Portable Superset
=============================================================

Единый скрипт, который:
  1. Создаёт чистую examples.db из CSV-файлов
  2. Исправляет подключение к БД в superset.db (портативный URI)
  3. Регистрирует датасеты РЖД с колонками
  4. Создаёт графики (charts / slices) с ПРАВИЛЬНЫМ форматом params
  5. Собирает дашборд с разметкой

Формат params скопирован с реально работающих графиков Superset.

Запуск:
    cd <project_root>
    python\\python.exe setup\\create_rzd_dashboard.py
"""

import csv
import json
import os
import sqlite3
import sys
import uuid as uuid_mod
from datetime import datetime
from pathlib import Path

# ─── Пути ───────────────────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent
SUPERSET_HOME = ROOT_DIR / "superset_home"
SUPERSET_DB = SUPERSET_HOME / "superset.db"
EXAMPLES_DB = ROOT_DIR / "examples.db"
DEMO_DATA_DIR = ROOT_DIR / "docs" / "demo_data"

# ─── Утилиты ────────────────────────────────────────────────────────────────


def uuid_bytes(hex_str=None):
    """UUID как 16-байтный blob для SQLite (формат Superset)."""
    if hex_str:
        return uuid_mod.UUID(hex_str).bytes
    return uuid_mod.uuid4().bytes


def now_iso():
    return datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S.%f")


def load_csv(filename):
    """Загрузить CSV из папки demo_data."""
    path = DEMO_DATA_DIR / filename
    if not path.exists():
        print(f"  [SKIP] Файл не найден: {path.name}")
        return [], []
    with open(path, "r", encoding="utf-8") as f:
        reader = csv.reader(f)
        headers = next(reader)
        rows = list(reader)
    return headers, rows


def guess_sqlite_type(values):
    """Грубо определить тип колонки по значениям."""
    for v in values:
        if not v:
            continue
        try:
            int(v)
            continue
        except ValueError:
            pass
        try:
            float(v)
            continue
        except ValueError:
            return "TEXT"
    # Все числа — проверяем int vs float
    for v in values:
        if not v:
            continue
        try:
            int(v)
        except ValueError:
            return "REAL"
    return "INTEGER"


def infer_schema(headers, rows):
    """Определить типы колонок по данным."""
    types = []
    for i, h in enumerate(headers):
        col_values = [r[i] for r in rows[:50] if i < len(r)]
        types.append(guess_sqlite_type(col_values))
    return list(zip(headers, types))


# ─── Фаза 1: Создание examples.db ──────────────────────────────────────────

CSV_FILES = [
    "rzd_stations.csv",
    "rzd_stations_full.csv",
    "rzd_monthly_stats.csv",
    "rzd_cargo_types.csv",
    "rzd_daily_operations.csv",
    "rzd_incidents.csv",
    "rzd_kpi_metrics.csv",
    "rzd_routes.csv",
]


def create_examples_db():
    """Создать чистую examples.db из CSV-файлов."""
    print("\n" + "=" * 60)
    print("  ФАЗА 1: Создание examples.db из CSV")
    print("=" * 60)

    if EXAMPLES_DB.exists():
        EXAMPLES_DB.unlink()
        print(f"  [OK] Старая examples.db удалена")

    conn = sqlite3.connect(str(EXAMPLES_DB))
    total_rows = 0

    for filename in CSV_FILES:
        table_name = filename.replace(".csv", "")
        headers, rows = load_csv(filename)
        if not headers:
            continue

        schema = infer_schema(headers, rows)

        # CREATE TABLE
        cols_sql = ", ".join(
            f'"{col}" {typ}' for col, typ in schema
        )
        conn.execute(f'DROP TABLE IF EXISTS "{table_name}"')
        conn.execute(f'CREATE TABLE "{table_name}" ({cols_sql})')

        # INSERT DATA
        placeholders = ", ".join(["?"] * len(headers))
        for row in rows:
            converted = []
            for i, (col, typ) in enumerate(schema):
                val = row[i] if i < len(row) else None
                if val == "" or val is None:
                    converted.append(None)
                elif typ == "INTEGER":
                    try:
                        converted.append(int(val))
                    except (ValueError, TypeError):
                        converted.append(val)
                elif typ == "REAL":
                    try:
                        converted.append(float(val))
                    except (ValueError, TypeError):
                        converted.append(val)
                else:
                    converted.append(val)
            conn.execute(
                f'INSERT INTO "{table_name}" VALUES ({placeholders})',
                converted,
            )

        total_rows += len(rows)
        print(f"  [OK] {table_name}: {len(rows)} строк, {len(headers)} колонок")

    conn.commit()
    conn.close()

    size_kb = EXAMPLES_DB.stat().st_size / 1024
    print(f"\n  Итого: {total_rows} строк, {size_kb:.0f} КБ")
    return True


# ─── Фаза 2: Настройка Superset metadata ───────────────────────────────────

# Фиксированные UUID для стабильности
UUIDS = {
    "db_examples":     "a2dc77af-e654-49bb-b321-40f6b559a1ee",
    "ds_stations":     "d1000001-0001-0001-0001-000000000001",
    "ds_monthly":      "d1000002-0002-0002-0002-000000000002",
    "ds_cargo":        "d1000003-0003-0003-0003-000000000003",
    "ds_daily":        "d1000004-0004-0004-0004-000000000004",
    "ds_incidents":    "d1000005-0005-0005-0005-000000000005",
    "ds_kpi":          "d1000006-0006-0006-0006-000000000006",
    "ch_total_pass":   "c2000001-0001-0001-0001-000000000001",
    "ch_monthly_bar":  "c2000002-0002-0002-0002-000000000002",
    "ch_cargo_pie":    "c2000003-0003-0003-0003-000000000003",
    "ch_stations_tbl": "c2000004-0004-0004-0004-000000000004",
    "ch_daily_line":   "c2000005-0005-0005-0005-000000000005",
    "ch_incidents_bar": "c2000006-0006-0006-0006-000000000006",
    "dashboard":       "d3000001-0001-0001-0001-000000000001",
}


# Описания датасетов
DATASETS = [
    {
        "key": "ds_stations",
        "table_name": "rzd_stations",
        "description": "Станции РЖД — 50 крупнейших станций России",
        "csv": "rzd_stations.csv",
        "main_dttm_col": None,
    },
    {
        "key": "ds_monthly",
        "table_name": "rzd_monthly_stats",
        "description": "Месячная статистика перевозок РЖД за 2024-2025",
        "csv": "rzd_monthly_stats.csv",
        "main_dttm_col": None,
    },
    {
        "key": "ds_cargo",
        "table_name": "rzd_cargo_types",
        "description": "Типы грузов и объёмы перевозок",
        "csv": "rzd_cargo_types.csv",
        "main_dttm_col": None,
    },
    {
        "key": "ds_daily",
        "table_name": "rzd_daily_operations",
        "description": "Ежедневные операции по регионам и типам маршрутов",
        "csv": "rzd_daily_operations.csv",
        "main_dttm_col": "date",
    },
    {
        "key": "ds_incidents",
        "table_name": "rzd_incidents",
        "description": "Инциденты на железной дороге за 2024 год",
        "csv": "rzd_incidents.csv",
        "main_dttm_col": "date",
    },
    {
        "key": "ds_kpi",
        "table_name": "rzd_kpi_metrics",
        "description": "Ключевые показатели эффективности (KPI) по кварталам",
        "csv": "rzd_kpi_metrics.csv",
        "main_dttm_col": None,
    },
]


def get_columns_from_csv(csv_filename):
    """Получить информацию о колонках из CSV."""
    headers, rows = load_csv(csv_filename)
    if not headers:
        return []
    schema = infer_schema(headers, rows)
    result = []
    for col_name, col_type in schema:
        superset_type = "STRING"
        is_dttm = False
        groupby = True
        filterable = True
        if col_type == "INTEGER":
            superset_type = "INTEGER"
        elif col_type == "REAL":
            superset_type = "FLOAT"
            groupby = False
        if col_name == "date":
            is_dttm = True
            superset_type = "STRING"  # stored as TEXT in SQLite
        result.append({
            "column_name": col_name,
            "type": superset_type,
            "is_dttm": is_dttm,
            "groupby": groupby,
            "filterable": filterable,
        })
    return result


def clean_old_rzd_data(cur):
    """Удалить старые записи РЖД из superset.db (агрессивная очистка)."""
    print("\n  Очистка старых данных РЖД...")

    # 1. Собираем ID всех РЖД датасетов
    cur.execute("SELECT id FROM tables WHERE table_name LIKE 'rzd_%'")
    rzd_table_ids = [r[0] for r in cur.fetchall()]

    # 2. Собираем ID всех графиков, привязанных к этим датасетам
    rzd_slice_ids = []
    if rzd_table_ids:
        placeholders = ",".join("?" * len(rzd_table_ids))
        cur.execute(
            f"SELECT id FROM slices WHERE datasource_id IN ({placeholders})",
            rzd_table_ids,
        )
        rzd_slice_ids.extend(r[0] for r in cur.fetchall())

    # Также ловим графики по имени
    cur.execute("""
        SELECT id FROM slices WHERE
            slice_name LIKE '%РЖД%' OR slice_name LIKE '%RZD%'
            OR slice_name LIKE '%Пассажир%' OR slice_name LIKE '%грузов%'
            OR slice_name LIKE '%Станци%' OR slice_name LIKE '%Инцидент%'
            OR slice_name LIKE '%KPI%' OR slice_name LIKE '%Выручка%'
            OR datasource_name LIKE 'rzd_%'
    """)
    rzd_slice_ids.extend(r[0] for r in cur.fetchall())
    rzd_slice_ids = list(set(rzd_slice_ids))

    # 3. Удаляем dashboard_slices
    cur.execute("""
        DELETE FROM dashboard_slices
        WHERE dashboard_id IN (
            SELECT id FROM dashboards WHERE slug = 'rzd_analytics'
        )
    """)
    if rzd_slice_ids:
        placeholders = ",".join("?" * len(rzd_slice_ids))
        cur.execute(
            f"DELETE FROM dashboard_slices WHERE slice_id IN ({placeholders})",
            rzd_slice_ids,
        )

    # 4. Удаляем charts
    if rzd_slice_ids:
        placeholders = ",".join("?" * len(rzd_slice_ids))
        cur.execute(f"DELETE FROM slices WHERE id IN ({placeholders})", rzd_slice_ids)
    cur.execute("""
        DELETE FROM slices WHERE
            slice_name LIKE '%РЖД%' OR slice_name LIKE '%RZD%'
            OR slice_name LIKE '%Пассажир%' OR slice_name LIKE '%грузов%'
            OR slice_name LIKE '%Станци%' OR slice_name LIKE '%Инцидент%'
            OR slice_name LIKE '%KPI%' OR slice_name LIKE '%Выручка%'
            OR datasource_name LIKE 'rzd_%'
    """)

    # 5. Удаляем колонки
    if rzd_table_ids:
        placeholders = ",".join("?" * len(rzd_table_ids))
        cur.execute(
            f"DELETE FROM table_columns WHERE table_id IN ({placeholders})",
            rzd_table_ids,
        )

    # 6. Удаляем datasets
    cur.execute("DELETE FROM tables WHERE table_name LIKE 'rzd_%'")

    # 7. Удаляем dashboard
    cur.execute("DELETE FROM dashboards WHERE slug = 'rzd_analytics'")

    # 8. Удаляем осиротевшие table_columns
    cur.execute("""
        DELETE FROM table_columns
        WHERE table_id NOT IN (SELECT id FROM tables)
    """)

    print(f"  [OK] Очищено: {len(rzd_table_ids)} датасетов, {len(rzd_slice_ids)} графиков")


def fix_examples_db_connection(cur):
    """Исправить URI подключения к examples.db — абсолютный путь от конфига."""
    print("\n  Настройка подключения к examples.db...")

    # Абсолютный путь к examples.db — вычисляем от корня проекта
    examples_abs = str(EXAMPLES_DB).replace("\\", "/")
    portable_uri = f"sqlite:///{examples_abs}"

    db_uuid = uuid_bytes(UUIDS["db_examples"])
    now = now_iso()

    cur.execute("SELECT id FROM dbs WHERE database_name = 'examples'")
    row = cur.fetchone()

    if row:
        db_id = row[0]
        cur.execute("""
            UPDATE dbs SET
                sqlalchemy_uri = ?,
                uuid = ?,
                allow_file_upload = 1,
                expose_in_sqllab = 1,
                allow_dml = 1,
                changed_on = ?
            WHERE id = ?
        """, (portable_uri, db_uuid, now, db_id))
        print(f"  [OK] Обновлено подключение (id={db_id})")
        print(f"       URI: {portable_uri}")
    else:
        extra = json.dumps({
            "metadata_params": {},
            "engine_params": {},
            "metadata_cache_timeout": {},
            "schemas_allowed_for_file_upload": [],
        })
        cur.execute("""
            INSERT INTO dbs (
                database_name, sqlalchemy_uri, uuid, extra,
                expose_in_sqllab, allow_dml, allow_file_upload,
                allow_ctas, allow_cvas, allow_run_async,
                select_as_create_table_as, impersonate_user,
                created_on, changed_on, created_by_fk, changed_by_fk
            ) VALUES (
                'examples', ?, ?, ?,
                1, 1, 1,
                0, 0, 0,
                0, 0,
                ?, ?, 1, 1
            )
        """, (portable_uri, db_uuid, extra, now, now))
        db_id = cur.lastrowid
        print(f"  [OK] Создано подключение (id={db_id})")
        print(f"       URI: {portable_uri}")

    return db_id


def register_datasets(cur, db_id):
    """Зарегистрировать датасеты РЖД в метаданных Superset."""
    print("\n  Регистрация датасетов...")
    now = now_iso()
    dataset_ids = {}

    for ds in DATASETS:
        ds_uuid = uuid_bytes(UUIDS[ds["key"]])
        perm = f"[examples].[{ds['table_name']}](id:{db_id})"

        cur.execute("""
            INSERT INTO tables (
                table_name, database_id, schema, description,
                is_sqllab_view, filter_select_enabled, is_featured,
                uuid, perm, main_dttm_col,
                created_on, changed_on,
                created_by_fk, changed_by_fk,
                is_managed_externally, normalize_columns,
                always_filter_main_dttm
            ) VALUES (
                ?, ?, '', ?,
                0, 1, 0,
                ?, ?, ?,
                ?, ?,
                1, 1,
                0, 0,
                0
            )
        """, (
            ds["table_name"], db_id, ds["description"],
            ds_uuid, perm, ds.get("main_dttm_col"),
            now, now,
        ))
        table_id = cur.lastrowid
        dataset_ids[ds["key"]] = table_id
        print(f"  [OK] Датасет: {ds['table_name']} (id={table_id})")

        # Удаляем старые колонки на случай коллизии id
        cur.execute("DELETE FROM table_columns WHERE table_id = ?", (table_id,))

        # Регистрируем колонки
        columns = get_columns_from_csv(ds["csv"])
        for col in columns:
            col_uuid = uuid_bytes()
            cur.execute("""
                INSERT INTO table_columns (
                    table_id, column_name, type,
                    is_dttm, is_active, groupby, filterable,
                    uuid, created_on, changed_on,
                    created_by_fk, changed_by_fk
                ) VALUES (
                    ?, ?, ?,
                    ?, 1, ?, ?,
                    ?, ?, ?,
                    1, 1
                )
            """, (
                table_id, col["column_name"], col["type"],
                1 if col["is_dttm"] else 0,
                1 if col["groupby"] else 0,
                1 if col["filterable"] else 0,
                col_uuid, now, now,
            ))

        print(f"         + {len(columns)} колонок")

    return dataset_ids


# ─── Фаза 3: Создание графиков ─────────────────────────────────────────────

# Params format copied from WORKING Superset charts.
# Key fields required by Superset frontend:
#   - viz_type (must match)
#   - granularity_sqla (even if null — must be present)
#   - time_range (must be present — "No filter" for non-temporal)
#   - metrics / metric (format depends on viz_type)
#   - groupby (list of column names)
#   - row_limit (integer)

def create_charts(cur, dataset_ids):
    """Создать графики (slices) для дашборда."""
    print("\n  Создание графиков...")
    now = now_iso()
    chart_ids = {}

    charts = [
        # ── 1. Big Number Total: Суммарный пассажиропоток ──
        {
            "key": "ch_total_pass",
            "name": "Пассажиропоток (млн)",
            "viz_type": "big_number_total",
            "dataset_key": "ds_monthly",
            "params": {
                "viz_type": "big_number_total",
                "granularity_sqla": None,
                "time_range": "No filter",
                "metric": {
                    "aggregate": "SUM",
                    "column": {
                        "column_name": "passengers_mln",
                        "type": "FLOAT",
                    },
                    "expressionType": "SIMPLE",
                    "label": "SUM(passengers_mln)",
                    "optionName": "metric_rzd_1",
                },
                "subheader": "млн пасс. за 2024 год",
                "y_axis_format": ",.1f",
                "row_limit": 10000,
            },
        },
        # ── 2. Echarts Bar: Выручка по месяцам ──
        {
            "key": "ch_monthly_bar",
            "name": "Выручка по месяцам (млрд ₽)",
            "viz_type": "echarts_timeseries_bar",
            "dataset_key": "ds_monthly",
            "params": {
                "viz_type": "echarts_timeseries_bar",
                "granularity_sqla": None,
                "time_range": "No filter",
                "x_axis": "month",
                "x_axis_sort_asc": True,
                "metrics": [
                    {
                        "aggregate": "SUM",
                        "column": {
                            "column_name": "revenue_bln_rub",
                            "type": "FLOAT",
                        },
                        "expressionType": "SIMPLE",
                        "label": "Выручка (млрд ₽)",
                        "optionName": "metric_rzd_2",
                    }
                ],
                "groupby": [],
                "row_limit": 10000,
                "order_desc": True,
                "show_legend": True,
                "rich_tooltip": True,
                "y_axis_format": ",.1f",
                "truncate_metric": True,
            },
        },
        # ── 3. Pie: Типы грузов ──
        {
            "key": "ch_cargo_pie",
            "name": "Распределение грузов",
            "viz_type": "pie",
            "dataset_key": "ds_cargo",
            "params": {
                "viz_type": "pie",
                "granularity_sqla": None,
                "time_range": "No filter",
                "groupby": ["cargo_type"],
                "metric": {
                    "aggregate": "SUM",
                    "column": {
                        "column_name": "volume_mln_tons",
                        "type": "FLOAT",
                    },
                    "expressionType": "SIMPLE",
                    "label": "Объём (млн тонн)",
                    "optionName": "metric_rzd_3",
                },
                "row_limit": 100,
                "sort_by_metric": True,
                "color_scheme": "supersetColors",
                "show_labels": True,
                "show_legend": True,
                "label_type": "key_percent",
                "number_format": ",.1f",
            },
        },
        # ── 4. Table: Станции РЖД ──
        {
            "key": "ch_stations_tbl",
            "name": "Крупнейшие станции РЖД",
            "viz_type": "table",
            "dataset_key": "ds_stations",
            "params": {
                "viz_type": "table",
                "granularity_sqla": None,
                "time_range": "No filter",
                "query_mode": "raw",
                "all_columns": [
                    "name",
                    "city",
                    "region",
                    "railway_branch",
                    "passengers_day",
                    "cargo_tons_year",
                    "station_class",
                ],
                "order_by_cols": [
                    '["passengers_day", false]'
                ],
                "row_limit": 50,
                "include_search": True,
                "page_length": 15,
                "color_pn": True,
            },
        },
        # ── 5. Echarts Line: Пассажиры по регионам ──
        {
            "key": "ch_daily_line",
            "name": "Пассажиры по регионам (тыс.)",
            "viz_type": "echarts_timeseries_line",
            "dataset_key": "ds_daily",
            "params": {
                "viz_type": "echarts_timeseries_line",
                "granularity_sqla": "date",
                "time_range": "No filter",
                "metrics": [
                    {
                        "aggregate": "SUM",
                        "column": {
                            "column_name": "passengers_thousands",
                            "type": "FLOAT",
                        },
                        "expressionType": "SIMPLE",
                        "label": "Пассажиров (тыс.)",
                        "optionName": "metric_rzd_5",
                    }
                ],
                "groupby": ["region"],
                "row_limit": 10000,
                "order_desc": True,
                "show_legend": True,
                "rich_tooltip": True,
                "y_axis_format": ",.0f",
                "color_scheme": "supersetColors",
            },
        },
        # ── 6. Echarts Bar: Инциденты по типам и серьёзности ──
        {
            "key": "ch_incidents_bar",
            "name": "Инциденты по типам",
            "viz_type": "echarts_timeseries_bar",
            "dataset_key": "ds_incidents",
            "params": {
                "viz_type": "echarts_timeseries_bar",
                "granularity_sqla": None,
                "time_range": "No filter",
                "x_axis": "incident_type",
                "metrics": [
                    {
                        "aggregate": "COUNT",
                        "column": {
                            "column_name": "incident_id",
                            "type": "STRING",
                        },
                        "expressionType": "SIMPLE",
                        "label": "Количество",
                        "optionName": "metric_rzd_6",
                    }
                ],
                "groupby": ["severity"],
                "row_limit": 10000,
                "order_desc": True,
                "color_scheme": "supersetColors",
                "show_legend": True,
                "stack": True,
                "y_axis_format": ",.0f",
            },
        },
    ]

    for chart_def in charts:
        ds_key = chart_def["dataset_key"]
        ds_id = dataset_ids[ds_key]
        chart_uuid = uuid_bytes(UUIDS[chart_def["key"]])

        params = chart_def["params"].copy()
        # datasource field in params: "{id}__{type}"
        params["datasource"] = f"{ds_id}__table"

        perm_name = next(
            d["table_name"] for d in DATASETS if d["key"] == ds_key
        )

        cur.execute("""
            INSERT INTO slices (
                slice_name, viz_type, datasource_type,
                datasource_id, datasource_name,
                params, uuid,
                created_on, changed_on,
                created_by_fk, changed_by_fk,
                is_managed_externally
            ) VALUES (
                ?, ?, 'table',
                ?, ?,
                ?, ?,
                ?, ?,
                1, 1,
                0
            )
        """, (
            chart_def["name"], chart_def["viz_type"],
            ds_id, perm_name,
            json.dumps(params, ensure_ascii=False), chart_uuid,
            now, now,
        ))
        chart_id = cur.lastrowid
        chart_ids[chart_def["key"]] = chart_id
        print(f"  [OK] График: {chart_def['name']} (id={chart_id}, {chart_def['viz_type']})")

    return chart_ids


# ─── Фаза 4: Создание дашборда ─────────────────────────────────────────────

def create_dashboard(cur, chart_ids):
    """Создать дашборд «РЖД Аналитика» с разметкой."""
    print("\n  Создание дашборда...")
    now = now_iso()
    dash_uuid = uuid_bytes(UUIDS["dashboard"])

    ch_total = chart_ids["ch_total_pass"]
    ch_bar = chart_ids["ch_monthly_bar"]
    ch_pie = chart_ids["ch_cargo_pie"]
    ch_line = chart_ids["ch_daily_line"]
    ch_table = chart_ids["ch_stations_tbl"]
    ch_inc = chart_ids["ch_incidents_bar"]

    # Position JSON — формат скопирован с рабочего дашборда Superset
    # Ключ: meta.chartId должен совпадать с id в slices
    # Ключ: meta.width — в колонках (из 12)
    # Ключ: meta.height — в единицах (примерно 50 = полэкрана)
    # children у ROW — без parents (Superset добавит сам)
    position = {
        "DASHBOARD_VERSION_KEY": "v2",
        "ROOT_ID": {
            "id": "ROOT_ID",
            "type": "ROOT",
            "children": ["GRID_ID"],
        },
        "GRID_ID": {
            "id": "GRID_ID",
            "type": "GRID",
            "children": [
                "ROW-1",
                "ROW-2",
                "ROW-3",
            ],
            "parents": ["ROOT_ID"],
        },
        "HEADER_ID": {
            "id": "HEADER_ID",
            "type": "HEADER",
            "meta": {
                "text": "РЖД Аналитика",
            },
        },
        # ── Row 1: Big Number + Monthly Bar ──
        "ROW-1": {
            "id": "ROW-1",
            "type": "ROW",
            "children": ["CHART-total", "CHART-bar"],
            "meta": {"background": "BACKGROUND_TRANSPARENT"},
        },
        "CHART-total": {
            "id": "CHART-total",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_total,
                "width": 4,
                "height": 50,
                "sliceName": "Пассажиропоток (млн)",
                "uuid": UUIDS["ch_total_pass"],
            },
        },
        "CHART-bar": {
            "id": "CHART-bar",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_bar,
                "width": 8,
                "height": 50,
                "sliceName": "Выручка по месяцам (млрд руб)",
                "uuid": UUIDS["ch_monthly_bar"],
            },
        },
        # ── Row 2: Pie + Line ──
        "ROW-2": {
            "id": "ROW-2",
            "type": "ROW",
            "children": ["CHART-pie", "CHART-line"],
            "meta": {"background": "BACKGROUND_TRANSPARENT"},
        },
        "CHART-pie": {
            "id": "CHART-pie",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_pie,
                "width": 4,
                "height": 50,
                "sliceName": "Распределение грузов",
                "uuid": UUIDS["ch_cargo_pie"],
            },
        },
        "CHART-line": {
            "id": "CHART-line",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_line,
                "width": 8,
                "height": 50,
                "sliceName": "Пассажиры по регионам (тыс.)",
                "uuid": UUIDS["ch_daily_line"],
            },
        },
        # ── Row 3: Table + Incidents Bar ──
        "ROW-3": {
            "id": "ROW-3",
            "type": "ROW",
            "children": ["CHART-table", "CHART-inc"],
            "meta": {"background": "BACKGROUND_TRANSPARENT"},
        },
        "CHART-table": {
            "id": "CHART-table",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_table,
                "width": 8,
                "height": 50,
                "sliceName": "Крупнейшие станции РЖД",
                "uuid": UUIDS["ch_stations_tbl"],
            },
        },
        "CHART-inc": {
            "id": "CHART-inc",
            "type": "CHART",
            "children": [],
            "meta": {
                "chartId": ch_inc,
                "width": 4,
                "height": 50,
                "sliceName": "Инциденты по типам",
                "uuid": UUIDS["ch_incidents_bar"],
            },
        },
    }

    # ── JSON metadata дашборда ──
    json_metadata = json.dumps({
        "color_scheme": "supersetColors",
        "refresh_frequency": 0,
        "expanded_slices": {},
        "timed_refresh_immune_slices": [],
        "label_colors": {},
        "shared_label_colors": {},
        "color_scheme_domain": [],
        "map_label_colors": {},
    }, ensure_ascii=False)

    position_json = json.dumps(position, ensure_ascii=False)

    cur.execute("""
        INSERT INTO dashboards (
            dashboard_title, slug, position_json,
            json_metadata, css, description,
            published, uuid,
            created_on, changed_on,
            created_by_fk, changed_by_fk,
            is_managed_externally
        ) VALUES (
            ?, ?, ?,
            ?, '', ?,
            1, ?,
            ?, ?,
            1, 1,
            0
        )
    """, (
        "РЖД Аналитика",
        "rzd_analytics",
        position_json,
        json_metadata,
        "Демо-дашборд: аналитика перевозок Российских железных дорог",
        dash_uuid,
        now, now,
    ))
    dashboard_id = cur.lastrowid
    print(f"  [OK] Дашборд: РЖД Аналитика (id={dashboard_id})")

    # ── Связь дашборда с графиками ──
    for chart_key, chart_id in chart_ids.items():
        cur.execute("""
            INSERT INTO dashboard_slices (dashboard_id, slice_id)
            VALUES (?, ?)
        """, (dashboard_id, chart_id))

    print(f"  [OK] Привязано {len(chart_ids)} графиков к дашборду")

    return dashboard_id


# ─── MAIN ───────────────────────────────────────────────────────────────────

def main():
    print()
    print("=" * 60)
    print("  Создание демо-дашборда РЖД Аналитика")
    print("  Portable Apache Superset")
    print("=" * 60)

    # Проверки
    if not DEMO_DATA_DIR.exists():
        print(f"\n[ОШИБКА] Папка с данными не найдена: {DEMO_DATA_DIR}")
        sys.exit(1)

    if not SUPERSET_DB.exists():
        print(f"\n[ОШИБКА] База Superset не найдена: {SUPERSET_DB}")
        print("  Сначала запустите Superset хотя бы раз!")
        sys.exit(1)

    # Фаза 1: Создание examples.db
    create_examples_db()

    # Фаза 2-4: Работа с superset.db
    print("\n" + "=" * 60)
    print("  ФАЗА 2-4: Настройка метаданных Superset")
    print("=" * 60)

    conn = sqlite3.connect(str(SUPERSET_DB))
    cur = conn.cursor()

    try:
        # Очистка старых данных
        clean_old_rzd_data(cur)

        # Настройка подключения к examples.db
        db_id = fix_examples_db_connection(cur)

        # Регистрация датасетов
        dataset_ids = register_datasets(cur, db_id)

        # Создание графиков
        chart_ids = create_charts(cur, dataset_ids)

        # Создание дашборда
        dashboard_id = create_dashboard(cur, chart_ids)

        conn.commit()

        print("\n" + "=" * 60)
        print("  ГОТОВО!")
        print("=" * 60)
        print()
        print("  Запустите Superset:")
        print("    start_superset.bat")
        print()
        print("  Откройте дашборд:")
        print("    http://localhost:8088/superset/dashboard/rzd_analytics/")
        print()
        print("  Логин: admin / Пароль: admin")
        print()

    except Exception as e:
        conn.rollback()
        print(f"\n[ОШИБКА] {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
