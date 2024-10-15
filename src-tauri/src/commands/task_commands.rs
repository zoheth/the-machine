use std::sync::Arc;

use crate::core::task_manager::{Task, TaskManager};
use tauri::State;

#[tauri::command]
pub async fn add_task(
    text: String,
    ordered: bool,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<usize, String> {
    Ok(task_manager.add_task(text, ordered))
}

#[tauri::command]
pub async fn add_subtask(
    parent_id: usize,
    text: String,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<usize, String> {
    task_manager.add_subtask(parent_id, text)
}

#[tauri::command]
pub async fn complete_task(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<(), String> {
    task_manager.complete_task(id)
}

#[tauri::command]
pub async fn uncomplete_task(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<(), String> {
    task_manager.uncomplete_task(id)
}

#[tauri::command]
pub async fn toggle_ordered(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<(), String> {
    task_manager.toggle_ordered(id)
}

#[tauri::command]
pub async fn get_active_tasks(
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<Vec<Task>, String> {
    Ok(task_manager.get_active_tasks())
}

#[tauri::command]
pub async fn get_subtasks(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<Vec<Task>, String> {
    task_manager.get_subtasks(id)
}

#[tauri::command]
pub async fn get_parent_tasks(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<Vec<Task>, String> {
    task_manager.get_parent_tasks(id)
}

#[tauri::command]
pub async fn get_task(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<Task, String> {
    match task_manager.get_task(id) {
        Some(task) => Ok(task),
        None => Err(format!("Task with id {} not found", id)),
    }
}

#[tauri::command]
pub async fn reorder_subtasks(
    parent_id: usize,
    new_order: Vec<usize>,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<(), String> {
    task_manager.reorder_subtasks(parent_id, new_order)
}

#[tauri::command]
pub async fn remove_task(
    id: usize,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<usize, String> {
    task_manager.remove_task_recursive(id)
}

#[tauri::command]
pub async fn update_task(
    id: usize,
    text: String,
    task_manager: State<'_, Arc<TaskManager>>,
) -> Result<(), String> {
    task_manager.update_task_text(id, text)
}
