use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

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

pub struct DbConnection(pub Mutex<Connection>);

fn init_db(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            task_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_name VARCHAR(255) NOT NULL,
            task_time_start TIME,
            task_time_end TIME,
            task_color CHAR(7) NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn times_overlap(start1: &str, end1: &str, start2: &str, end2: &str) -> bool {
    // Times are in HH:MM format
    // Two ranges overlap if start1 < end2 AND start2 < end1
    start1 < end2 && start2 < end1
}

#[tauri::command]
fn get_all_tasks(db: State<DbConnection>) -> Result<Vec<Task>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT task_id, task_name, task_time_start, task_time_end, task_color FROM tasks ORDER BY task_time_start")
        .map_err(|e| e.to_string())?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(Task {
                task_id: row.get(0)?,
                task_name: row.get(1)?,
                task_time_start: row.get(2)?,
                task_time_end: row.get(3)?,
                task_color: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tasks)
}

#[tauri::command]
fn create_task(
    db: State<DbConnection>,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
) -> Result<Task, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check for overlaps
    let mut stmt = conn
        .prepare("SELECT task_time_start, task_time_end FROM tasks")
        .map_err(|e| e.to_string())?;

    let existing_times: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (start, end) in &existing_times {
        if times_overlap(&task_time_start, &task_time_end, start, end) {
            return Err("Task overlaps with an existing task".to_string());
        }
    }

    conn.execute(
        "INSERT INTO tasks (task_name, task_time_start, task_time_end, task_color) VALUES (?1, ?2, ?3, ?4)",
        (&task_name, &task_time_start, &task_time_end, &task_color),
    )
    .map_err(|e| e.to_string())?;

    let task_id = conn.last_insert_rowid();

    Ok(Task {
        task_id,
        task_name,
        task_time_start,
        task_time_end,
        task_color,
    })
}

#[tauri::command]
fn update_task(
    db: State<DbConnection>,
    id: i64,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
) -> Result<Task, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check for overlaps (excluding current task)
    let mut stmt = conn
        .prepare("SELECT task_time_start, task_time_end FROM tasks WHERE task_id != ?1")
        .map_err(|e| e.to_string())?;

    let existing_times: Vec<(String, String)> = stmt
        .query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (start, end) in &existing_times {
        if times_overlap(&task_time_start, &task_time_end, start, end) {
            return Err("Task overlaps with an existing task".to_string());
        }
    }

    conn.execute(
        "UPDATE tasks SET task_name = ?1, task_time_start = ?2, task_time_end = ?3, task_color = ?4 WHERE task_id = ?5",
        (&task_name, &task_time_start, &task_time_end, &task_color, &id),
    )
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
fn delete_task(db: State<DbConnection>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE task_id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_all_tasks(db: State<DbConnection>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Helper to convert "HH:MM" to minutes from 00:00
fn time_to_minutes(time: &str) -> i32 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return 0;
    }
    let h: i32 = parts[0].parse().unwrap_or(0);
    let m: i32 = parts[1].parse().unwrap_or(0);
    h * 60 + m
}

// Helper to convert minutes to "HH:MM"
fn minutes_to_time(minutes: i32) -> String {
    let m = minutes.rem_euclid(24 * 60); // Handle wrap around
    let h = m / 60;
    let min = m % 60;
    format!("{:02}:{:02}", h, min)
}

#[tauri::command]
fn update_task_with_shift(
    db: State<DbConnection>,
    id: i64,
    task_name: String,
    task_time_start: String,
    task_time_end: String,
    task_color: String,
    original_end: String,
) -> Result<Task, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // calculate delta
    let old_end_mins = time_to_minutes(&original_end);
    let new_end_mins = time_to_minutes(&task_time_end);

    // Handle crossing midnight for delta calculation if needed
    // Assuming small shifts, but if new_end < old_end it might be next day?
    // User requirement: "end time span has increased".
    // We assume explicit increase.
    // If old=23:50, new=00:20 (next day). new_end_mins=20. old=1430.
    // delta = 20 - 1430 = -1410. Incorrect.
    // We need "logical" comparison.

    let diff = new_end_mins - old_end_mins;
    // Normalize to -12h to +12h range to interpret direction correctly
    let delta_mins = if diff > 12 * 60 {
        diff - 24 * 60
    } else if diff < -12 * 60 {
        diff + 24 * 60
    } else {
        diff
    };

    // Begin transaction
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    // 1. Update current task
    conn.execute(
        "UPDATE tasks SET task_name = ?1, task_time_start = ?2, task_time_end = ?3, task_color = ?4 WHERE task_id = ?5",
        (&task_name, &task_time_start, &task_time_end, &task_color, id),
    )
    .map_err(|e| e.to_string())?;

    // 2. Shift subsequent tasks
    // Find all tasks starting >= original_end (string comparison might fail for midnight wrap)
    // We'll fetch all, filter logically, update.
    let mut stmt = conn
        .prepare("SELECT task_id, task_time_start, task_time_end FROM tasks WHERE task_id != ?1")
        .map_err(|e| e.to_string())?;

    let tasks_to_shift: Vec<(i64, String, String)> = stmt
        .query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (t_id, t_start, t_end) in tasks_to_shift {
        // Logic check: does this task start at or after original_end?
        // Again, handle wrap around logic if app supports consistent < 6 AM is next day logic.
        // We'll use the same logic as Frontend: < 6 => +24h.

        let start_offset = 6 * 60; // 06:00 in minutes

        let mut t_start_m = time_to_minutes(&t_start);
        if t_start_m < start_offset {
            t_start_m += 24 * 60;
        }

        let mut ref_end_m = old_end_mins;
        if ref_end_m < start_offset {
            ref_end_m += 24 * 60;
        }

        if t_start_m >= ref_end_m {
            // Shift this task
            let new_s = minutes_to_time(time_to_minutes(&t_start) + delta_mins);
            let new_e = minutes_to_time(time_to_minutes(&t_end) + delta_mins);

            conn.execute(
                "UPDATE tasks SET task_time_start = ?1, task_time_end = ?2 WHERE task_id = ?3",
                (&new_s, &new_e, t_id),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(Task {
        task_id: id,
        task_name,
        task_time_start,
        task_time_end,
        task_color,
    })
}

#[tauri::command]
fn check_overlap(
    db: State<DbConnection>,
    task_time_start: String,
    task_time_end: String,
    exclude_id: Option<i64>,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let query = match exclude_id {
        Some(_) => "SELECT task_time_start, task_time_end FROM tasks WHERE task_id != ?1",
        None => "SELECT task_time_start, task_time_end FROM tasks",
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let existing_times: Vec<(String, String)> = match exclude_id {
        Some(id) => stmt
            .query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        None => stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };

    for (start, end) in &existing_times {
        if times_overlap(&task_time_start, &task_time_end, start, end) {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Get the database path relative to the executable
    let db_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("../../../db"))
        .unwrap_or_else(|| std::path::PathBuf::from("db"));

    let conn = Connection::open(&db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DbConnection(Mutex::new(conn)))
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
