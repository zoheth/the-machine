use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Write};
use serde_json;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: usize,
    text: String,
    completed: bool,
    ordered: bool,
    subtasks: Vec<usize>,     // IDs of subtasks
    predecessors: Vec<usize>, // IDs of predecessors
    parent: Option<usize>,    // ID of the parent task
}

impl Task {
    fn new(id: usize, text: String, ordered: bool) -> Self {
        Task {
            id,
            text,
            completed: false,
            ordered,
            subtasks: Vec::new(),
            predecessors: Vec::new(),
            parent: None, 
        }
    }
}

struct TaskManager {
    tasks: Mutex<HashMap<usize, Arc<Mutex<Task>>>>,
    next_id: Mutex<usize>,
}

impl TaskManager {
    fn new() -> Self {
        TaskManager {
            tasks: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    fn generate_id(&self) -> usize {
        let mut id = self.next_id.lock().unwrap();
        let current_id = *id;
        *id += 1;
        current_id
    }

    fn add_task(&self, text: String, ordered: bool) -> usize {
        let id = self.generate_id();
        let task = Arc::new(Mutex::new(Task::new(id, text, ordered)));
        self.tasks.lock().unwrap().insert(id, task);
        id
    }

    fn add_subtask(&self, parent_id: usize, text: String) -> Result<usize, String> {
        let id = self.generate_id();
        let subtask = Arc::new(Mutex::new(Task::new(id, text.clone(), true)));

        // Acquire the parent task without holding the lock on self.tasks
        let parent_task = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .get(&parent_id)
                .ok_or(format!("Task with id: {} not found", id))?
                .clone()
        };

        // Set the ordered flag to match the parent
        {
            let mut subtask_lock = subtask.lock().unwrap();
            let parent_task_lock = parent_task.lock().unwrap();
            subtask_lock.ordered = parent_task_lock.ordered;
            subtask_lock.parent = Some(parent_id);
        }

        // Handle ordering and predecessors
        {
            let mut parent_task_lock = parent_task.lock().unwrap();
            if parent_task_lock.ordered {
                if let Some(&last_subtask_id) = parent_task_lock.subtasks.last() {
                    subtask.lock().unwrap().predecessors.push(last_subtask_id);
                }
            }
            parent_task_lock.subtasks.push(id);
        }

        // Insert the subtask into the task map
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, subtask);
        }

        Ok(id)
    }

    fn complete_task(&self, id: usize) -> Result<(), String> {
        let task = {
            let tasks = self.tasks.lock().unwrap();
            tasks.get(&id).ok_or(format!("Task with id: {} not found", id))?.clone()
        };
        task.lock().unwrap().completed = true;
        Ok(())
    }

    fn get_active_tasks(&self) -> Vec<Task> {
        // Clone the tasks map to avoid holding the lock during iteration
        let tasks_map = {
            let tasks = self.tasks.lock().unwrap();
            tasks.clone()
        };

        tasks_map
            .values()
            .filter_map(|task| {
                let task_lock = task.lock().unwrap();
                if self.is_task_active(&task_lock, &tasks_map) {
                    Some(task_lock.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_task_active(&self, task: &Task, tasks_map: &HashMap<usize, Arc<Mutex<Task>>>) -> bool {
        if task.completed {
            return false;
        }

        // Collect all IDs of predecessors and subtasks
        let mut related_task_ids = task.predecessors.clone();
        related_task_ids.extend(&task.subtasks);

        // Sort the IDs to ensure consistent lock order
        related_task_ids.sort_unstable();

        // Check if all predecessors and subtasks are completed
        let all_related_tasks_completed = related_task_ids.iter().all(|&tid| {
            if let Some(t) = tasks_map.get(&tid) {
                let t_lock = t.lock().unwrap();
                t_lock.completed
            } else {
                false
            }
        });

        all_related_tasks_completed
    }

    fn get_subtasks(&self, id: usize) -> Result<Vec<Task>, String> {
        let task = {
            let tasks = self.tasks.lock().unwrap();
            tasks.get(&id).ok_or(format!("Task with id: {} not found", id))?.clone()
        };

        let subtasks_ids = {
            let task_lock = task.lock().unwrap();
            task_lock.subtasks.clone()
        };

        let tasks_map = {
            let tasks = self.tasks.lock().unwrap();
            tasks.clone()
        };

        let subtasks: Vec<Task> = subtasks_ids
            .iter()
            .filter_map(|&sid| tasks_map.get(&sid))
            .map(|t| t.lock().unwrap().clone())
            .collect();

        for subtask in &subtasks {
            println!("Subtask ID: {}", subtask.id);
        }

        Ok(subtasks)
    }

    fn get_parent_tasks(&self, task_id: usize) -> Result<Vec<Task>, String> {
        let mut hierarchy = Vec::new();
        let mut current_task_id = Some(task_id);

        while let Some(id) = current_task_id {
            let task = {
                let tasks = self.tasks.lock().unwrap();
                tasks.get(&id).ok_or(format!("Task with id: {} not found", id))?.clone()
            };

            let task_lock = task.lock().unwrap();
            hierarchy.push(task_lock.clone());

            current_task_id = task_lock.parent;
        }

        Ok(hierarchy)
    }

    fn get_task(&self, id: usize) -> Option<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).map(|t| t.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_task() {
        let manager = TaskManager::new();
        let task_id = manager.add_task("Test Task".to_string(), true);
        let task = manager.get_task(task_id).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.text, "Test Task");
        assert!(task.subtasks.is_empty());
    }

    #[test]
    fn test_add_subtasks_and_predecessors() {
        let manager = TaskManager::new();
        let parent_id = manager.add_task("Parent Task".to_string(), true);

        let subtask1_id = manager
            .add_subtask(parent_id, "Subtask 1".to_string())
            .unwrap();
        let subtask2_id = manager
            .add_subtask(parent_id, "Subtask 2".to_string())
            .unwrap();

        let subtask1 = manager.get_task(subtask1_id).unwrap();
        let subtask2 = manager.get_task(subtask2_id).unwrap();

        assert!(subtask1.predecessors.is_empty());
        assert_eq!(subtask2.predecessors, vec![subtask1_id]);

        let parent_task = manager.get_task(parent_id).unwrap();
        assert_eq!(parent_task.subtasks, vec![subtask1_id, subtask2_id]);
    }

    #[test]
    fn test_get_active_tasks() {
        let manager = TaskManager::new();
        let task_id = manager.add_task("Task".to_string(), true);
        let subtask_id = manager.add_subtask(task_id, "Subtask".to_string()).unwrap();

        let active_tasks = manager.get_active_tasks();
        // The subtask should be active at this point
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].id, subtask_id);

        manager.complete_task(subtask_id).unwrap();

        let active_tasks = manager.get_active_tasks();
        // Now, the parent task should be active
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].id, task_id);
    }

    #[test]
    fn test_get_parent_tasks() {
        let manager = TaskManager::new();
        let parent_id = manager.add_task("Parent Task".to_string(), true);
        let subtask_id = manager.add_subtask(parent_id, "Subtask".to_string()).unwrap();
        let hierarchy = manager.get_parent_tasks(subtask_id).unwrap();
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0].text, "Subtask");
        assert_eq!(hierarchy[1].text, "Parent Task");
    }
}

/// Initializes the task manager as a Tauri state.
fn init_task_manager() -> TaskManager {
    TaskManager::new()
}

#[tauri::command]
async fn add_task(
    text: String,
    ordered: bool,
    task_manager: State<'_, TaskManager>,
) -> Result<usize, String> {
    Ok(task_manager.add_task(text, ordered))
}

#[tauri::command]
async fn add_subtask(
    parent_id: usize,
    text: String,
    task_manager: State<'_, TaskManager>,
) -> Result<usize, String> {
    task_manager.add_subtask(parent_id, text)
}

#[tauri::command]
async fn complete_task(id: usize, task_manager: State<'_, TaskManager>) -> Result<(), String> {
    task_manager.complete_task(id)
}

#[tauri::command]
async fn get_active_tasks(task_manager: State<'_, TaskManager>) -> Result<Vec<Task>, String> {
    Ok(task_manager.get_active_tasks())
}

#[tauri::command]
async fn get_subtasks(
    id: usize,
    task_manager: State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    task_manager.get_subtasks(id)
}

#[tauri::command]
async fn get_parent_tasks(
    id: usize,
    task_manager: State<'_, TaskManager>,
) -> Result<Vec<Task>, String> {
    task_manager.get_parent_tasks(id)
}

#[tauri::command]
async fn get_task(id: usize, task_manager: State<'_, TaskManager>) -> Result<Task, String> {
    match task_manager.get_task(id) {
        Some(task) => Ok(task),
        None => Err(format!("Task with id {} not found", id)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(init_task_manager())
        .invoke_handler(tauri::generate_handler![
            add_task,
            add_subtask,
            complete_task,
            get_active_tasks,
            get_subtasks,
            get_parent_tasks,
            get_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
