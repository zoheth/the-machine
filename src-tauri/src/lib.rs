pub mod commands;
pub mod core;

use commands::task_commands::*;
use core::task_manager::TaskManager;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::async_runtime;
use tokio::time::sleep;

fn get_data_file_path() -> PathBuf {
    let app_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    app_dir.join("task_manager_data.json")
}

/// Initializes the task manager as a Tauri state.
fn init_task_manager() -> Arc<TaskManager> {
    let task_manager = Arc::new(TaskManager::new());

    let file_path = get_data_file_path();
    if let Err(e) = task_manager.load_from_file(file_path.to_str().unwrap()) {
        println!("Failed to load data: {}", e);
    }
    task_manager
}

fn start_auto_save(task_manager: Arc<TaskManager>, interval: Duration) {
    async_runtime::spawn(async move {
        loop {
            sleep(interval).await;
            let file_path = get_data_file_path();

            if let Err(e) = task_manager.save_to_file(file_path.to_str().unwrap()) {
                println!("Auto-save failed: {}", e);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_manager = init_task_manager();
    let task_manager_clone = Arc::clone(&task_manager);
    start_auto_save(Arc::clone(&task_manager), Duration::from_secs(300));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(task_manager)
        .invoke_handler(tauri::generate_handler![
            commands::task_commands::add_task,
            add_subtask,
            complete_task,
            uncomplete_task,
            toggle_ordered,
            get_active_tasks,
            get_subtasks,
            get_parent_tasks,
            get_task,
            reorder_subtasks,
            remove_task,
            update_task
        ])
        .on_window_event(move |_, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let file_path = get_data_file_path();
                if let Err(e) = task_manager_clone.save_to_file(file_path.to_str().unwrap()) {
                    println!("Failed to save data on window close: {}", e);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
