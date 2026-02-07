use libsql::{Builder, Connection, Value};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager};
use std::sync::Arc;

const TURSO_DATABASE_URL: &str = "libsql://dappler-alexiko.aws-eu-west-1.turso.io";
const TURSO_AUTH_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicnciLCJpYXQiOjE3NzA0NTIwODAsImlkIjoiYWUxZjNiNWUtZGRjMy00OTgyLWFiMGMtMDg0NTc4NjYxMWRhIiwicmlkIjoiZjAwNDJhMGItMGM1MC00MDgxLWE4YzQtM2RlNGI1YmNjOWM5In0.WQ3brd558pKMzaTjtQS_T5QNYE60yxUUhoJcjvtwuMKWnad08FM7PQy4-vP6Uqy3aLdhhmWXD2O2_JNLmh1pAA";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub task_id: i64,
    pub task_name: String,
    pub task_time_start: String,
    pub task_time_end: String,
    pub task_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTask {
    pub task_name: String,
    pub task_time_start: String,
    pub task_time_end: String,
    pub task_color: String,
}

#[derive(Clone)]
pub struct DbConnection {
    conn: Arc<Connection>,
}

async fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            task_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_name VARCHAR(255) NOT NULL,
            task_time_start TIME,
            task_time_end TIME,
            task_color CHAR(7) NOT NULL
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn times_overlap(start1: &str, end1: &str, start2: &str, end2: &str) -> bool {
    start1 < end2 && start2 < end1
}

#[tauri::command]
async fn get_all_tasks(state: State<'_, DbConnection>) -> Result<Vec<Task>, String> {
    let mut rows = state.conn
        .query("SELECT task_id, task_name, task_time_start, task_time_end, task_color FROM tasks ORDER BY task_time_start", ())
        .await
        .map_err(|e| e.to_string())?;

    let mut tasks = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        tasks.push(Task {
            task_id: row.get(0).map_err(|e| e.to_string())?,
            task_name: row.get(1).map_err(|e| e.to_string())?,
            task_time_start: row.get(2).map_err(|e| e.to_string())?,
            task_time_end: row.get(3).map_err(|e| e.to_string())?,
            task_color: row.get(4).map_err(|e| e.to_string())?,
        });
    }
    Ok(tasks)
}

#[tauri::command]
async fn create_task(
    state: State<'_, DbConnection>,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
) -> Result<Task, String> {
    let mut rows = state.conn
        .query("SELECT task_time_start, task_time_end FROM tasks", ())
        .await
        .map_err(|e| e.to_string())?;
    
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let start: String = row.get(0).map_err(|e| e.to_string())?;
        let end: String = row.get(1).map_err(|e| e.to_string())?;
        if times_overlap(&task_time_start, &task_time_end, &start, &end) {
            return Err("Task overlaps with an existing task".to_string());
        }
    }

    let params: Vec<Value> = vec![
        task_name.clone().into(),
        task_time_start.clone().into(),
        task_time_end.clone().into(),
        task_color.clone().into(),
    ];
    state.conn.execute(
        "INSERT INTO tasks (task_name, task_time_start, task_time_end, task_color) VALUES (?, ?, ?, ?)",
        params,
    )
    .await
    .map_err(|e| e.to_string())?;

    let params_select: Vec<Value> = vec![
        task_name.into(),
        task_time_start.into(),
    ];
    let mut rows = state.conn
        .query(
            "SELECT task_id, task_name, task_time_start, task_time_end, task_color FROM tasks WHERE task_name = ? AND task_time_start = ? ORDER BY task_id DESC LIMIT 1",
            params_select, 
        )
        .await
        .map_err(|e| e.to_string())?;
        
    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
         Ok(Task {
            task_id: row.get(0).map_err(|e| e.to_string())?,
            task_name: row.get(1).map_err(|e| e.to_string())?,
            task_time_start: row.get(2).map_err(|e| e.to_string())?,
            task_time_end: row.get(3).map_err(|e| e.to_string())?,
            task_color: row.get(4).map_err(|e| e.to_string())?,
        })
    } else {
        Err("Failed to retrieve created task".to_string())
    }
}

#[tauri::command]
async fn update_task(
    state: State<'_, DbConnection>,
    id: i64,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
) -> Result<Task, String> {
     let mut rows = state.conn
        .query("SELECT task_id, task_time_start, task_time_end FROM tasks", ())
        .await
        .map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let other_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        if other_id == id { continue; }
        
        let start: String = row.get(1).map_err(|e| e.to_string())?;
        let end: String = row.get(2).map_err(|e| e.to_string())?;
        
        if times_overlap(&task_time_start, &task_time_end, &start, &end) {
            return Err("Task overlaps with an existing task".to_string());
        }
    }

    let params: Vec<Value> = vec![
        task_name.clone().into(),
        task_time_start.clone().into(),
        task_time_end.clone().into(),
        task_color.clone().into(),
        id.into(),
    ];
    state.conn.execute(
        "UPDATE tasks SET task_name = ?, task_time_start = ?, task_time_end = ?, task_color = ? WHERE task_id = ?",
        params,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(Task {
        task_id: id,
        task_name,
        task_time_start,
        task_time_end,
        task_color,
    })
}

#[tauri::command]
async fn delete_task(state: State<'_, DbConnection>, id: i64) -> Result<(), String> {
    let params: Vec<Value> = vec![id.into()];
    state.conn.execute("DELETE FROM tasks WHERE task_id = ?", params)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_all_tasks(state: State<'_, DbConnection>) -> Result<(), String> {
    state.conn.execute("DELETE FROM tasks", ())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn time_to_minutes(t: &str) -> i32 {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() != 2 { return 0; }
    let h: i32 = parts[0].parse().unwrap_or(0);
    let m: i32 = parts[1].parse().unwrap_or(0);
    h * 60 + m
}

fn minutes_to_time(m: i32) -> String {
    let h = (m / 60) % 24; // Simple wrap for 24h
    let m = m % 60;
    format!("{:02}:{:02}", h, m)
}

#[tauri::command]
async fn update_task_with_shift(
    state: State<'_, DbConnection>,
    id: i64,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
    original_end: String,
) -> Result<Task, String> {
    let old_end_mins = time_to_minutes(&original_end);
    let new_end_mins = time_to_minutes(&task_time_end);
    
    let diff = new_end_mins - old_end_mins;
    
    let delta = if diff > 720 {
        diff - 1440
    } else if diff < -720 {
        diff + 1440
    } else {
        diff
    };
    
    if delta == 0 {
        return update_task(state, id, task_name, task_time_start, task_time_end, task_color).await;
    }
    
    let params: Vec<Value> = vec![id.into()];
    let mut rows = state.conn
        .query("SELECT task_id, task_time_start, task_time_end FROM tasks WHERE task_id != ?", params)
        .await
        .map_err(|e| e.to_string())?;
        
    let mut tasks_to_shift = Vec::new();
    let offset_cutoff = 6 * 60;
    
    let mut ref_end_abs = old_end_mins;
    if ref_end_abs < offset_cutoff { ref_end_abs += 1440; }

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let t_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let t_start: String = row.get(1).map_err(|e| e.to_string())?;
        let t_end: String = row.get(2).map_err(|e| e.to_string())?;
        
        let mut t_start_mins = time_to_minutes(&t_start);
        if t_start_mins < offset_cutoff { t_start_mins += 1440; }
        
        if t_start_mins >= ref_end_abs {
            tasks_to_shift.push((t_id, t_start, t_end));
        }
    }

    for (t_id, t_start, t_end) in tasks_to_shift {
        let s_mins = time_to_minutes(&t_start);
        let e_mins = time_to_minutes(&t_end);
        
        let new_s = minutes_to_time(s_mins + delta);
        let new_e = minutes_to_time(e_mins + delta);
        
        let params: Vec<Value> = vec![new_s.into(), new_e.into(), t_id.into()];
        state.conn.execute(
            "UPDATE tasks SET task_time_start = ?, task_time_end = ? WHERE task_id = ?",
             params,
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    
    let params: Vec<Value> = vec![
        task_name.clone().into(),
        task_time_start.clone().into(),
        task_time_end.clone().into(),
        task_color.clone().into(),
        id.into(),
    ];
    state.conn.execute(
        "UPDATE tasks SET task_name = ?, task_time_start = ?, task_time_end = ?, task_color = ? WHERE task_id = ?",
        params,
    )
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(Task {
        task_id: id,
        task_name,
        task_time_start,
        task_time_end,
        task_color,
    })
}

#[tauri::command]
async fn check_overlap(
    state: State<'_, DbConnection>,
    task_time_start: String,
    task_time_end: String,
    exclude_id: Option<i64>,
) -> Result<bool, String> {
    let mut rows = state.conn
        .query("SELECT task_id, task_time_start, task_time_end FROM tasks", ())
        .await
        .map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let t_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        if let Some(eid) = exclude_id {
            if t_id == eid { continue; }
        }
        
        let start: String = row.get(1).map_err(|e| e.to_string())?;
        let end: String = row.get(2).map_err(|e| e.to_string())?;
        
        if times_overlap(&task_time_start, &task_time_end, &start, &end) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db = Builder::new_remote(TURSO_DATABASE_URL.to_string(), TURSO_AUTH_TOKEN.to_string())
                    .build()
                    .await
                    .expect("Failed to build db");
                
                let conn = db.connect().expect("Failed to connect");
                
                init_db(&conn).await.expect("Failed to init db");
                
                handle.manage(DbConnection { conn: Arc::new(conn) });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_tasks,
            create_task,
            update_task,
            delete_task,
            delete_all_tasks,
            check_overlap,
            update_task_with_shift
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
