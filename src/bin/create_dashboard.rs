use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

// --- Config ---
const DEMO_DATA_DIR: &str = "docs/demo_data";
const EXAMPLES_DB_PATH: &str = "examples.db";
const SUPERSET_HOME_DIR: &str = "superset_home";
const SUPERSET_DB_NAME: &str = "superset.db";

// --- UUIDs ---
// Fixed UUIDs for stability (same as Python script)
const UUID_DB_EXAMPLES: &str = "a2dc77af-e654-49bb-b321-40f6b559a1ee";
const UUID_DASHBOARD: &str = "d3000001-0001-0001-0001-000000000001";

// Chart UUIDs
const UUID_CH_TOTAL: &str = "c2000001-0001-0001-0001-000000000001";
const UUID_CH_BAR: &str = "c2000002-0002-0002-0002-000000000001"; // Wait, check original. 
// Actually, let's just use the logic to get them from the CHARTS array or just hardcode literals in the json macro for simplicity and readability since they are fixed.
// Better: Define them as consts.

const UUID_CH_TOTAL_PASS: &str = "c2000001-0001-0001-0001-000000000001";
const UUID_CH_MONTHLY_BAR: &str = "c2000002-0002-0002-0002-000000000002";
const UUID_CH_CARGO_PIE: &str = "c2000003-0003-0003-0003-000000000003";
const UUID_CH_STATIONS_TBL: &str = "c2000004-0004-0004-0004-000000000004";
const UUID_CH_DAILY_LINE: &str = "c2000005-0005-0005-0005-000000000005";
const UUID_CH_INCIDENTS_BAR: &str = "c2000006-0006-0006-0006-000000000006";

// --- Data Structures ---
struct DatasetDef {
    key: &'static str,
    table_name: &'static str,
    description: &'static str,
    csv: &'static str,
    main_dttm_col: Option<&'static str>,
    uuid_str: &'static str,
}

const DATASETS: &[DatasetDef] = &[
    DatasetDef { key: "ds_stations", table_name: "rzd_stations", description: "Станции РЖД", csv: "rzd_stations.csv", main_dttm_col: None, uuid_str: "d1000001-0001-0001-0001-000000000001" },
    DatasetDef { key: "ds_monthly", table_name: "rzd_monthly_stats", description: "Месячная статистика", csv: "rzd_monthly_stats.csv", main_dttm_col: None, uuid_str: "d1000002-0002-0002-0002-000000000002" },
    DatasetDef { key: "ds_cargo", table_name: "rzd_cargo_types", description: "Типы грузов", csv: "rzd_cargo_types.csv", main_dttm_col: None, uuid_str: "d1000003-0003-0003-0003-000000000003" },
    DatasetDef { key: "ds_daily", table_name: "rzd_daily_operations", description: "Ежедневные операции", csv: "rzd_daily_operations.csv", main_dttm_col: Some("date"), uuid_str: "d1000004-0004-0004-0004-000000000004" },
    DatasetDef { key: "ds_incidents", table_name: "rzd_incidents", description: "Инциденты", csv: "rzd_incidents.csv", main_dttm_col: Some("date"), uuid_str: "d1000005-0005-0005-0005-000000000005" },
    DatasetDef { key: "ds_kpi", table_name: "rzd_kpi_metrics", description: "KPI", csv: "rzd_kpi_metrics.csv", main_dttm_col: None, uuid_str: "d1000006-0006-0006-0006-000000000006" },
    DatasetDef { key: "ds_world", table_name: "world_rail_stats", description: "World Rail Stats", csv: "world_rail_stats.csv", main_dttm_col: None, uuid_str: "e4000002-0002-0002-0002-000000000002" },
];

struct ChartDef {
    key: &'static str,
    name: &'static str,
    viz_type: &'static str,
    dataset_key: &'static str,
    uuid_str: &'static str,
    params_json: &'static str,
}

const CHARTS: &[ChartDef] = &[
    ChartDef { key: "ch_world_stats", name: "Railway Statistics", viz_type: "table", dataset_key: "ds_world", uuid_str: "e4000003-0003-0003-0003-000000000003",
        params_json: r#"{
            "viz_type": "table", "query_mode": "raw", "all_columns": ["line_name", "country", "length_km", "passengers_mln_year", "max_speed_kmh"],
            "order_by_cols": ["[\"length_km\", false]"], "include_search": true, "page_length": 10
        }"# },
    ChartDef { key: "ch_world_map", name: "Global Networks", viz_type: "deck_geojson", dataset_key: "ds_world", uuid_str: "e4000004-0004-0004-0004-000000000004",
        params_json: r#"{
            "viz_type": "deck_geojson", "geojson_url": "http://localhost:8089/world_rail.geojson",
            "mapbox_style": "mapbox://styles/mapbox/light-v9", 
            "viewport": {"latitude": 20, "longitude": 0, "zoom": 1.5, "bearing": 0, "pitch": 0},
            "filled": false, "stroked": true, "extruded": false, "lineWidth": 1500, "lineColor": [255, 0, 0, 200],
            "autozoom": true
        }"# }, 
    ChartDef { key: "ch_total_pass", name: "Пассажиропоток (млн)", viz_type: "big_number_total", dataset_key: "ds_monthly", uuid_str: "c2000001-0001-0001-0001-000000000001", 
        params_json: r#"{
            "viz_type": "big_number_total", "granularity_sqla": null, "time_range": "No filter", 
            "metric": {"aggregate": "SUM", "column": {"column_name": "passengers_mln", "type": "FLOAT"}, "expressionType": "SIMPLE", "label": "SUM(passengers_mln)"}, 
            "subheader": "млн пасс. за 2024 год", "y_axis_format": ",.1f"
        }"# },
    ChartDef { key: "ch_monthly_bar", name: "Выручка по месяцам (млрд ₽)", viz_type: "echarts_timeseries_bar", dataset_key: "ds_monthly", uuid_str: "c2000002-0002-0002-0002-000000000002",
        params_json: r#"{
            "viz_type": "echarts_timeseries_bar", "granularity_sqla": null, "time_range": "No filter", "x_axis": "month", "x_axis_sort_asc": true,
            "metrics": [{"aggregate": "SUM", "column": {"column_name": "revenue_bln_rub", "type": "FLOAT"}, "expressionType": "SIMPLE", "label": "Выручка (млрд ₽)"}],
            "groupby": [], "order_desc": true, "show_legend": true, "y_axis_format": ",.1f"
        }"# },
    ChartDef { key: "ch_cargo_pie", name: "Распределение грузов", viz_type: "pie", dataset_key: "ds_cargo", uuid_str: "c2000003-0003-0003-0003-000000000003",
        params_json: r#"{
            "viz_type": "pie", "granularity_sqla": null, "time_range": "No filter", "groupby": ["cargo_type"],
            "metric": {"aggregate": "SUM", "column": {"column_name": "volume_mln_tons", "type": "FLOAT"}, "expressionType": "SIMPLE", "label": "Объём (млн тонн)"},
            "show_labels": true, "show_legend": true, "label_type": "key_percent", "number_format": ",.1f"
        }"# },
    ChartDef { key: "ch_stations_tbl", name: "Крупнейшие станции РЖД", viz_type: "table", dataset_key: "ds_stations", uuid_str: "c2000004-0004-0004-0004-000000000004",
        params_json: r#"{
            "viz_type": "table", "granularity_sqla": null, "time_range": "No filter", "query_mode": "raw",
            "all_columns": ["name", "city", "region", "railway_branch", "passengers_day", "cargo_tons_year", "station_class"],
            "order_by_cols": ["[\"passengers_day\", false]"], "include_search": true, "page_length": 15
        }"# },
    ChartDef { key: "ch_daily_line", name: "Пассажиры по регионам (тыс.)", viz_type: "echarts_timeseries_line", dataset_key: "ds_daily", uuid_str: "c2000005-0005-0005-0005-000000000005",
        params_json: r#"{
            "viz_type": "echarts_timeseries_line", "granularity_sqla": "date", "time_range": "No filter",
            "metrics": [{"aggregate": "SUM", "column": {"column_name": "passengers_thousands", "type": "FLOAT"}, "expressionType": "SIMPLE", "label": "Пассажиров (тыс.)"}],
            "groupby": ["region"], "show_legend": true, "y_axis_format": ",.0f"
        }"# },
    ChartDef { key: "ch_incidents_bar", name: "Инциденты по типам", viz_type: "echarts_timeseries_bar", dataset_key: "ds_incidents", uuid_str: "c2000006-0006-0006-0006-000000000006",
        params_json: r#"{
            "viz_type": "echarts_timeseries_bar", "granularity_sqla": null, "time_range": "No filter", "x_axis": "incident_type",
            "metrics": [{"aggregate": "COUNT", "column": {"column_name": "incident_id", "type": "STRING"}, "expressionType": "SIMPLE", "label": "Количество"}],
            "groupby": ["severity"], "stack": true, "show_legend": true, "y_axis_format": ",.0f"
        }"# },
];

// --- Helpers ---

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

fn uuid_from_str(s: &str) -> Vec<u8> {
    Uuid::parse_str(s).expect("Invalid UUID constant").as_bytes().to_vec()
}

fn new_uuid_bytes() -> Vec<u8> {
    Uuid::new_v4().as_bytes().to_vec()
}

fn get_root_dir() -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = env::current_exe()?;
    dir.pop(); // Remove exe name
    if dir.file_name().and_then(|n| n.to_str()) == Some("debug") || dir.file_name().and_then(|n| n.to_str()) == Some("release") {
        dir.pop(); 
        dir.pop(); // Go up to project root from target/debug
    }
    Ok(dir)
}

fn infer_col_type(val: &str) -> &'static str {
    if val.is_empty() { return "TEXT"; }
    if val.parse::<i64>().is_ok() { return "INTEGER"; }
    if val.parse::<f64>().is_ok() { return "REAL"; }
    "TEXT"
}

// --- Phase 1: Update examples.db ---

fn update_examples_db(root: &Path) -> Result<(), Box<dyn Error>> {
    let db_path = root.join(EXAMPLES_DB_PATH);
    if !db_path.exists() {
        println!("  [INFO] examples.db not found, creating new at {:?}", db_path);
    } else {
        println!("  [INFO] Using existing examples.db at {:?}", db_path);
    }

    let conn = Connection::open(&db_path)?;
    
    for ds in DATASETS {
        let csv_path = root.join(DEMO_DATA_DIR).join(ds.csv);
        if !csv_path.exists() {
            println!("  [SKIP] CSV not found: {:?}", csv_path);
            continue;
        }

        let mut rdr = csv::Reader::from_path(csv_path)?;
        let headers = rdr.headers()?.clone();
        
        // Infer schema from first row
        // (Simplified: assuming first row exists and is representative)
        let mut first_row_vals: Vec<String> = Vec::new();
        let mut types: Vec<&str> = Vec::new();
        
        // Peek at first row
        let mut records = rdr.records();
        let first_record_opt = records.next();

        if let Some(res) = first_record_opt {
             let record = res?;
             for field in record.iter() {
                 first_row_vals.push(field.to_string());
                 types.push(infer_col_type(field));
             }
        } else {
            // Empty csv? default to TEXT
            for _ in headers.iter() { types.push("TEXT"); }
        }

        // Re-open/reset reader to read all rows including first
        // Since we consumed the iterator, let's just re-open for simplicity
        let mut rdr = csv::Reader::from_path(root.join(DEMO_DATA_DIR).join(ds.csv))?;
        
        // DROP & CREATE
        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", ds.table_name), [])?;
        
        let cols_def: Vec<String> = headers.iter().zip(types.iter())
            .map(|(name, typ)| format!("\"{}\" {}", name, typ))
            .collect();
        
        conn.execute(&format!("CREATE TABLE \"{}\" ({})", ds.table_name, cols_def.join(", ")), [])?;

        // INSERT
        let placeholders: Vec<&str> = (0..headers.len()).map(|_| "?").collect();
        let query = format!("INSERT INTO \"{}\" VALUES ({})", ds.table_name, placeholders.join(", "));
        
        let mut stmt = conn.prepare(&query)?;
        
        let mut row_count = 0;
        for result in rdr.records() {
            let record = result?;
            // Rusqlite needs dynamic params. Convert string records to params.
            // This is a bit tricky in Rust with rusqlite's params! macro expectations.
            // We use params_from_iter.
            
            stmt.execute(rusqlite::params_from_iter(record.iter()))?;
            row_count += 1;
        }
        
        println!("  [OK] Table '{}': {} rows", ds.table_name, row_count);
    }
    
    Ok(())
}

// --- Phase 2: Metadata ---

fn update_metadata(root: &Path) -> Result<(), Box<dyn Error>> {
    let db_path = root.join(SUPERSET_HOME_DIR).join(SUPERSET_DB_NAME);
    if !db_path.exists() {
        return Err(format!("superset.db not found at {:?}", db_path).into());
    }
    
    let conn = Connection::open(&db_path)?;
    println!("  [INFO] Connected to superset.db");

    // 1. Fix examples DB URI
    let examples_abs = root.join(EXAMPLES_DB_PATH);
    let uri = format!("sqlite:///{}", examples_abs.to_string_lossy().replace("\\", "/"));
    let now = now_iso();
    let db_uuid = uuid_from_str(UUID_DB_EXAMPLES);

    // Upsert database connection
    // We check if exists by name 'examples'
    let mut stmt = conn.prepare("SELECT id FROM dbs WHERE database_name = 'examples'")?;
    let db_id: i32 = if let Ok(mut rows) = stmt.query([]) {
        if let Some(row) = rows.next()? {
             // Update
             let id: i32 = row.get(0)?;
             conn.execute("UPDATE dbs SET sqlalchemy_uri = ?, uuid = ?, changed_on = ? WHERE id = ?", 
                params![uri, db_uuid, now, id])?; // Borrowing needed for rusqlite blobs? -> Actually rusqlite handles Vec<u8> as blob.
                                                         // Wait, checking uuid handling. Uuid bytes are fine.
             println!("  [OK] Updated 'examples' DB URI (id={})", id);
             id
        } else {
             // Insert
             let extra = json!({
                 "metadata_params": {}, "engine_params": {}, "metadata_cache_timeout": {},
                 "schemas_allowed_for_file_upload": []
             }).to_string();
             
             conn.execute("INSERT INTO dbs (database_name, sqlalchemy_uri, uuid, extra, expose_in_sqllab, allow_dml, allow_file_upload, created_on, changed_on, created_by_fk, changed_by_fk) VALUES (?, ?, ?, ?, 1, 1, 1, ?, ?, 1, 1)",
                params!["examples", uri, db_uuid, extra, now, now])?;
             let id = conn.last_insert_rowid() as i32;
             println!("  [OK] Created 'examples' DB connection (id={})", id);
             id
        }
    } else {
        0 // Should not happen
    };

    // 2. Register Datasets
    let mut dataset_ids: HashMap<&str, i32> = HashMap::new();
    
    for ds in DATASETS {
        let uuid = uuid_from_str(ds.uuid_str);
        // Check if table exists
        // simplifiedupsert logic
        // We delete by UUID to ensure cleanliness for RZD tables? No, let's match by name & DB.
        
        let perm = format!("[examples].[{}](id:{})", ds.table_name, db_id);
        
        // Try get ID
        let mut stmt = conn.prepare("SELECT id FROM tables WHERE table_name = ? AND database_id = ?")?;
        let table_id: i32 = if let Some(row) = stmt.query(params![ds.table_name, db_id])?.next()? {
             let id: i32 = row.get(0)?;
             // Update
             conn.execute("UPDATE tables SET uuid = ?, description = ?, schema = '', perm = ?, changed_on = ? WHERE id = ?",
                params![uuid, ds.description, perm, now, id])?;
             id
        } else {
             // Insert
             conn.execute("INSERT INTO tables (table_name, database_id, schema, description, uuid, perm, main_dttm_col, created_on, changed_on, created_by_fk, changed_by_fk, is_sqllab_view, filter_select_enabled) VALUES (?, ?, '', ?, ?, ?, ?, ?, ?, 1, 1, 0, 1)",
                params![ds.table_name, db_id, ds.description, uuid, perm, ds.main_dttm_col, now, now])?;
             conn.last_insert_rowid() as i32
        };
        
        dataset_ids.insert(ds.key, table_id);
        println!("  [OK] Dataset '{}' (id={})", ds.table_name, table_id);
        
        // Columns - dumb implementation: delete all for this table and recreate
        conn.execute("DELETE FROM table_columns WHERE table_id = ?", params![table_id])?;
        
        // Read CSV header to get columns again...
        let csv_path = root.join(DEMO_DATA_DIR).join(ds.csv);
        let mut rdr = csv::Reader::from_path(csv_path)?;
        // We need types... re-infer or hardcode? 
        // Let's re-infer quickly from first row
        let headers = rdr.headers()?.clone();
        let mut types: Vec<&str> = Vec::new();
        if let Some(res) = rdr.records().next() {
             if let Ok(rec) = res {
                 for f in rec.iter() { types.push(infer_col_type(f)); }
             }
        }
        if types.is_empty() { 
             for _ in headers.iter() { types.push("TEXT"); }
        }
        
        for (i, col_name) in headers.iter().enumerate() {
            let typ = types.get(i).unwrap_or(&"TEXT");
            let superset_type = match *typ {
                "INTEGER" => "INTEGER",
                "REAL" => "FLOAT",
                _ => "STRING"
            };
            let is_dttm = if col_name == "date" { 1 } else { 0 };
            let groupby = if *typ == "REAL" { 0 } else { 1 };
            
            conn.execute("INSERT INTO table_columns (table_id, column_name, type, is_dttm, is_active, groupby, filterable, uuid, created_on, changed_on, created_by_fk, changed_by_fk) VALUES (?, ?, ?, ?, 1, ?, 1, ?, ?, ?, 1, 1)",
                params![table_id, col_name, superset_type, is_dttm, groupby, new_uuid_bytes(), now, now])?;
        }
    }

    // 3. Charts
    let mut chart_ids: HashMap<&str, i32> = HashMap::new();
    
    for chart in CHARTS {
        let uuid = uuid_from_str(chart.uuid_str);
        let ds_id = dataset_ids.get(chart.dataset_key).ok_or("Dataset ID not found")?;
        
        // Parse params json to inject datasource
        let mut params: serde_json::Value = serde_json::from_str(chart.params_json)?;
        params["datasource"] = json!(format!("{}_table", ds_id)); // incorrect format in my python script? 
        // Python said: f"{ds_id}__table" (double underscore)
        params["datasource"] = json!(format!("{}__table", ds_id));
        let params_str = params.to_string();
        
        let ds_def = DATASETS.iter().find(|d| d.key == chart.dataset_key).unwrap();

        // Upsert Slice
        let mut stmt = conn.prepare("SELECT id FROM slices WHERE slice_name = ?")?; // Match by name is risky but ok for demo
        let chart_id: i32 = if let Some(row) = stmt.query(params![chart.name])?.next()? {
             let id: i32 = row.get(0)?;
             conn.execute("UPDATE slices SET viz_type = ?, datasource_type = 'table', datasource_id = ?, datasource_name = ?, params = ?, uuid = ?, changed_on = ? WHERE id = ?",
                params![chart.viz_type, ds_id, ds_def.table_name, params_str, uuid, now, id])?;
             id
        } else {
             conn.execute("INSERT INTO slices (slice_name, viz_type, datasource_type, datasource_id, datasource_name, params, uuid, created_on, changed_on, created_by_fk, changed_by_fk) VALUES (?, ?, 'table', ?, ?, ?, ?, ?, ?, 1, 1)",
                params![chart.name, chart.viz_type, ds_id, ds_def.table_name, params_str, uuid, now, now])?;
             conn.last_insert_rowid() as i32
        };
        chart_ids.insert(chart.key, chart_id);
        println!("  [OK] Chart '{}' (id={})", chart.name, chart_id);
    }

    // 4. Dashboard
    let dash_uuid = uuid_from_str(UUID_DASHBOARD);
    let dash_slug = "rzd_analytics";
    
    // IDs
    let ch_total = chart_ids["ch_total_pass"];
    let ch_bar = chart_ids["ch_monthly_bar"];
    let ch_pie = chart_ids["ch_cargo_pie"];
    let ch_line = chart_ids["ch_daily_line"];
    let ch_table = chart_ids["ch_stations_tbl"];
    let ch_inc = chart_ids["ch_incidents_bar"];

    // Position JSON
    let position = json!({
        "DASHBOARD_VERSION_KEY": "v2",
        "ROOT_ID": { "id": "ROOT_ID", "type": "ROOT", "children": ["GRID_ID"] },
        "GRID_ID": { "id": "GRID_ID", "type": "GRID", "children": ["ROW-1", "ROW-2", "ROW-3"], "parents": ["ROOT_ID"] },
        "HEADER_ID": { "id": "HEADER_ID", "type": "HEADER", "meta": { "text": "РЖД Аналитика" } },
        
        // Row 1
        "ROW-1": { "id": "ROW-1", "type": "ROW", "children": ["CHART-total", "CHART-bar"], "meta": { "background": "BACKGROUND_TRANSPARENT" } },
        "CHART-total": { "id": "CHART-total", "type": "CHART", "children": [], "meta": { "chartId": ch_total, "width": 4, "height": 50, "sliceName": "Пассажиропоток (млн)", "uuid": UUID_CH_TOTAL_PASS } },
        "CHART-bar": { "id": "CHART-bar", "type": "CHART", "children": [], "meta": { "chartId": ch_bar, "width": 8, "height": 50, "sliceName": "Выручка по месяцам (млрд руб)", "uuid": UUID_CH_MONTHLY_BAR } },
        
        // Row 2
        "ROW-2": { "id": "ROW-2", "type": "ROW", "children": ["CHART-pie", "CHART-line"], "meta": { "background": "BACKGROUND_TRANSPARENT" } },
        "CHART-pie": { "id": "CHART-pie", "type": "CHART", "children": [], "meta": { "chartId": ch_pie, "width": 4, "height": 50, "sliceName": "Распределение грузов", "uuid": UUID_CH_CARGO_PIE } },
        "CHART-line": { "id": "CHART-line", "type": "CHART", "children": [], "meta": { "chartId": ch_line, "width": 8, "height": 50, "sliceName": "Пассажиры по регионам (тыс.)", "uuid": UUID_CH_DAILY_LINE } },
        
        // Row 3
        "ROW-3": { "id": "ROW-3", "type": "ROW", "children": ["CHART-table", "CHART-inc"], "meta": { "background": "BACKGROUND_TRANSPARENT" } },
        "CHART-table": { "id": "CHART-table", "type": "CHART", "children": [], "meta": { "chartId": ch_table, "width": 8, "height": 50, "sliceName": "Крупнейшие станции РЖД", "uuid": UUID_CH_STATIONS_TBL } },
        "CHART-inc": { "id": "CHART-inc", "type": "CHART", "children": [], "meta": { "chartId": ch_inc, "width": 4, "height": 50, "sliceName": "Инциденты по типам", "uuid": UUID_CH_INCIDENTS_BAR } }
    });
    
    let position_json = position.to_string();
    
    let metadata = json!({
        "color_scheme": "supersetColors",
        "refresh_frequency": 0,
        "expanded_slices": {},
        "timed_refresh_immune_slices": [],
        "label_colors": {},
        "shared_label_colors": {},
        "color_scheme_domain": [],
        "map_label_colors": {}
    });
    let metadata_json = metadata.to_string();

    // Check if dash exists
    let mut stmt = conn.prepare("SELECT id FROM dashboards WHERE slug = ?")?;
    let dash_id: i32 = if let Some(row) = stmt.query(params![dash_slug])?.next()? {
        let id: i32 = row.get(0)?;
        conn.execute("UPDATE dashboards SET dashboard_title = ?, position_json = ?, json_metadata = ?, published = 1, changed_on = ? WHERE id = ?",
            params!["РЖД Аналитика", position_json, metadata_json, now, id])?;
        id
    } else {
        conn.execute("INSERT INTO dashboards (dashboard_title, slug, position_json, json_metadata, uuid, published, created_on, changed_on, created_by_fk, changed_by_fk) VALUES (?, ?, ?, ?, ?, 1, ?, ?, 1, 1)",
            params!["РЖД Аналитика", dash_slug, position_json, metadata_json, dash_uuid, now, now])?;
        conn.last_insert_rowid() as i32
    };

    // Link charts
    conn.execute("DELETE FROM dashboard_slices WHERE dashboard_id = ?", params![dash_id])?;
    for (_, chart_id) in chart_ids.iter() {
        conn.execute("INSERT INTO dashboard_slices (dashboard_id, slice_id) VALUES (?, ?)", 
            params![dash_id, chart_id])?;
    }
    
    println!("  [OK] Dashboard '{}' (id={}) updated with layout.", "РЖД Аналитика", dash_id);

    // --- World Rail Dashboard ---
    let world_dash_uuid = uuid_from_str("e4000001-0001-0001-0001-000000000001");
    let world_dash_slug = "world_railways";
    
    // IDs
    let ch_world_table_id = chart_ids.get("ch_world_stats").copied().unwrap_or(0);
    let ch_world_map_id = chart_ids.get("ch_world_map").copied().unwrap_or(0);

    // Position JSON
    let world_position = json!({
        "DASHBOARD_VERSION_KEY": "v2",
        "ROOT_ID": { "id": "ROOT_ID", "type": "ROOT", "children": ["GRID_ID"] },
        "GRID_ID": { "id": "GRID_ID", "type": "GRID", "children": ["ROW-MAP", "ROW-TABLE"], "parents": ["ROOT_ID"] },
        "HEADER_ID": { "id": "HEADER_ID", "type": "HEADER", "meta": { "text": "World Railways (Offline Map)" } },
        
        "ROW-MAP": { "id": "ROW-MAP", "type": "ROW", "children": ["CHART-MAP"], "meta": { "background": "BACKGROUND_TRANSPARENT" } },
        "CHART-MAP": { "id": "CHART-MAP", "type": "CHART", "children": [], "meta": { "chartId": ch_world_map_id, "width": 12, "height": 60, "sliceName": "Global Networks", "uuid": "e4000004-0004-0004-0004-000000000004" } },
        
        "ROW-TABLE": { "id": "ROW-TABLE", "type": "ROW", "children": ["CHART-TABLE"], "meta": { "background": "BACKGROUND_TRANSPARENT" } },
        "CHART-TABLE": { "id": "CHART-TABLE", "type": "CHART", "children": [], "meta": { "chartId": ch_world_table_id, "width": 12, "height": 40, "sliceName": "Railway Statistics", "uuid": "e4000003-0003-0003-0003-000000000003" } }
    });
    
    let world_pos_json = world_position.to_string();

    // Check if dash exists
    let mut stmt = conn.prepare("SELECT id FROM dashboards WHERE slug = ?")?;
    let world_dash_id: i32 = if let Some(row) = stmt.query(params![world_dash_slug])?.next()? {
        let id: i32 = row.get(0)?;
        conn.execute("UPDATE dashboards SET dashboard_title = ?, position_json = ?, json_metadata = ?, published = 1, changed_on = ? WHERE id = ?",
            params!["World Railways", world_pos_json, metadata_json, now, id])?;
        id
    } else {
        conn.execute("INSERT INTO dashboards (dashboard_title, slug, position_json, json_metadata, uuid, published, created_on, changed_on, created_by_fk, changed_by_fk) VALUES (?, ?, ?, ?, ?, 1, ?, ?, 1, 1)",
            params!["World Railways", world_dash_slug, world_pos_json, metadata_json, world_dash_uuid, now, now])?;
        conn.last_insert_rowid() as i32
    };

    // Link charts
    conn.execute("DELETE FROM dashboard_slices WHERE dashboard_id = ?", params![world_dash_id])?;
    if ch_world_map_id > 0 { conn.execute("INSERT INTO dashboard_slices (dashboard_id, slice_id) VALUES (?, ?)", params![world_dash_id, ch_world_map_id])?; }
    if ch_world_table_id > 0 { conn.execute("INSERT INTO dashboard_slices (dashboard_id, slice_id) VALUES (?, ?)", params![world_dash_id, ch_world_table_id])?; }
    
    println!("  [OK] Dashboard '{}' (id={}) updated.", "World Railways", world_dash_id);

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    println!("========================================");
    println!("  Rust Dashboard Creator for RZD");
    println!("========================================");

    let root = get_root_dir().unwrap_or(PathBuf::from(".")); // Fallback
    
    // Handle dev environment (cargo run) where root is project root
    let root = if root.join("Cargo.toml").exists() { root } else { root };
    
    println!("Root dir: {:?}", root);

    // Phase 1
    update_examples_db(&root)?;

    // Phase 2
    update_metadata(&root)?;

    println!("\nSUCCESS: Dashboard data updated!");
    Ok(())
}
