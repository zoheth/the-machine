use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: usize,
    text: String,
    completed: bool,
    ordered: bool,
    subtasks: Vec<usize>,  // IDs of subtasks
    parent: Option<usize>, // ID of the parent task
}

impl Task {
    fn new(id: usize, text: String, ordered: bool) -> Self {
        Task {
            id,
            text,
            completed: false,
            ordered: true,
            subtasks: Vec::new(),
            parent: None,
        }
    }
}

struct TaskManager {
    tasks: Mutex<HashMap<usize, Arc<Mutex<Task>>>>,
    root_tasks: Mutex<Vec<usize>>,
    next_id: Mutex<usize>,
}

impl TaskManager {
    fn new() -> Self {
        TaskManager {
            tasks: Mutex::new(HashMap::new()),
            root_tasks: Mutex::new(Vec::new()),
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

        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, task);
        }

        {
            let mut root_tasks = self.root_tasks.lock().unwrap();
            root_tasks.push(id);
        }
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
                .ok_or(format!("Task with id: {} not found", parent_id))?
                .clone()
        };

        {
            let mut subtask_lock = subtask.lock().unwrap();
            subtask_lock.parent = Some(parent_id);
        }

        {
            let mut parent_task_lock = parent_task.lock().unwrap();
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
            tasks
                .get(&id)
                .ok_or(format!("Task with id: {} not found", id))?
                .clone()
        };
        task.lock().unwrap().completed = true;
        Ok(())
    }

    fn uncomplete_task(&self, id: usize) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(&id)
            .ok_or(format!("Task with id: {} not found", id))?;
        let mut task_lock = task.lock().unwrap();
        task_lock.completed = false;
        Ok(())
    }

    fn toggle_ordered(&self, id: usize) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(&id)
            .ok_or(format!("Task with id: {} not found", id))?;
        let mut task_lock = task.lock().unwrap();
        task_lock.ordered = !task_lock.ordered;
        Ok(())
    }

    // Method to adjust the order of subtasks
    fn reorder_subtasks(&self, parent_id: usize, new_order: Vec<usize>) -> Result<(), String> {
        let tasks_map = self.tasks.lock().unwrap();
        let parent_task_arc = tasks_map
            .get(&parent_id)
            .ok_or(format!("Parent task with id: {} not found", parent_id))?
            .clone();

        let mut parent_task_lock = parent_task_arc.lock().unwrap();

        // Validate that new_order contains the same subtasks
        let current_subtasks_set: HashSet<_> = parent_task_lock.subtasks.iter().cloned().collect();
        let new_subtasks_set: HashSet<_> = new_order.iter().cloned().collect();
        if current_subtasks_set != new_subtasks_set {
            return Err("New order must contain the same subtasks".to_string());
        }

        // Update the subtask order
        parent_task_lock.subtasks = new_order.clone();
        let ordered = parent_task_lock.ordered;
        drop(parent_task_lock);

        Ok(())
    }

    fn get_active_tasks(&self) -> Vec<Task> {
        // 克隆任务映射，避免持有锁
        let tasks_map = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .iter()
                .map(|(&id, task_arc)| {
                    let task_lock = task_arc.lock().unwrap();
                    (id, task_lock.clone())
                })
                .collect::<HashMap<usize, Task>>()
        };

        // 获取根任务的ID列表
        let root_task_ids = {
            let root_tasks = self.root_tasks.lock().unwrap();
            root_tasks.clone()
        };

        let mut active_tasks = Vec::new();

        // 从根任务开始遍历
        for root_task_id in root_task_ids {
            if let Some(root_task) = tasks_map.get(&root_task_id) {
                self.collect_active_tasks(root_task, &tasks_map, &mut active_tasks);
            }
        }

        active_tasks
    }

    fn collect_active_tasks(
        &self,
        task: &Task,
        tasks_map: &HashMap<usize, Task>,
        active_tasks: &mut Vec<Task>,
    ) {
        if task.completed {
            return;
        }

        if task.subtasks.is_empty() {
            // 没有子任务，任务为活跃任务
            active_tasks.push(task.clone());
            return;
        }

        let mut all_subtasks_completed = true;

        if task.ordered {
            // 有序任务，只考虑第一个未完成的子任务
            for &subtask_id in &task.subtasks {
                if let Some(subtask) = tasks_map.get(&subtask_id) {
                    if !subtask.completed {
                        self.collect_active_tasks(subtask, tasks_map, active_tasks);
                        all_subtasks_completed = false;
                        break;
                    }
                }
            }
        } else {
            // 无序任务，遍历所有未完成的子任务
            for &subtask_id in &task.subtasks {
                if let Some(subtask) = tasks_map.get(&subtask_id) {
                    if !subtask.completed {
                        self.collect_active_tasks(subtask, tasks_map, active_tasks);
                        all_subtasks_completed = false;
                    }
                }
            }
        }

        // 如果所有子任务都已完成，当前任务为活跃任务
        if all_subtasks_completed {
            active_tasks.push(task.clone());
        }
    }

    fn get_subtasks(&self, id: usize) -> Result<Vec<Task>, String> {
        let task = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .get(&id)
                .ok_or(format!("Task with id: {} not found", id))?
                .clone()
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
        Ok(subtasks)
    }

    fn get_parent_tasks(&self, task_id: usize) -> Result<Vec<Task>, String> {
        let mut hierarchy = Vec::new();
        let mut current_task_id = Some(task_id);

        while let Some(id) = current_task_id {
            let task = {
                let tasks = self.tasks.lock().unwrap();
                tasks
                    .get(&id)
                    .ok_or(format!("Task with id: {} not found", id))?
                    .clone()
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
    use std::collections::HashSet;

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
    fn test_get_active_tasks_complex() {
        let manager = TaskManager::new();

        // Create main tasks
        let task_a = manager.add_task("Task A".to_string(), true); // Ordered
        let task_b = manager.add_task("Task B".to_string(), false); // Unordered
        let task_c = manager.add_task("Task C".to_string(), true); // Ordered

        // Add subtasks to Task A
        let task_a1 = manager.add_subtask(task_a, "Task A1".to_string()).unwrap();
        let task_a2 = manager.add_subtask(task_a, "Task A2".to_string()).unwrap();
        let task_a3 = manager.add_subtask(task_a, "Task A3".to_string()).unwrap();

        // Add subtasks to Task B
        let task_b1 = manager.add_subtask(task_b, "Task B1".to_string()).unwrap();
        let task_b2 = manager.add_subtask(task_b, "Task B2".to_string()).unwrap();

        // Add subtasks to Task C
        let task_c1 = manager.add_subtask(task_c, "Task C1".to_string()).unwrap();
        let task_c2 = manager.add_subtask(task_c, "Task C2".to_string()).unwrap();

        // Add dependencies
        // Task A3 depends on Task B2
        {
            let tasks = manager.tasks.lock().unwrap();
            let task_a3_arc = tasks.get(&task_a3).unwrap().clone();
            let mut task_a3_lock = task_a3_arc.lock().unwrap();
            task_a3_lock.predecessors.push(task_b2);
        }

        // Task B2 depends on Task C
        {
            let tasks = manager.tasks.lock().unwrap();
            let task_b2_arc = tasks.get(&task_b2).unwrap().clone();
            let mut task_b2_lock = task_b2_arc.lock().unwrap();
            task_b2_lock.predecessors.push(task_c);
        }

        // Check initial active tasks
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        // Expected active tasks: Task A1, Task B1, Task C1
        let expected_active = vec![task_a1, task_b1, task_c1];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();

        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task A1
        manager.complete_task(task_a1).unwrap();

        // Now, Task A2 should become active
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a2, task_b1, task_c1];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task B1
        manager.complete_task(task_b1).unwrap();

        // No change in active tasks yet since B2 depends on C
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task C1
        manager.complete_task(task_c1).unwrap();

        // Task C2 becomes active
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a2, task_b2, task_c2];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task C2
        manager.complete_task(task_c2).unwrap();

        // Task B2's dependency on Task C is satisfied
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a2, task_b2];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task B2
        manager.complete_task(task_b2).unwrap();

        // Task A2 remains active
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a2];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task A2
        manager.complete_task(task_a2).unwrap();

        // Task A3 depends on B2 (which is completed), so it becomes active
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a3];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Toggle Task A to unordered
        manager.toggle_ordered(task_a).unwrap();

        // Now, Task A3 should have no internal predecessors due to order
        // Since its explicit predecessor B2 is completed, Task A3 remains active
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Reorder subtasks of Task A
        manager
            .reorder_subtasks(task_a, vec![task_a3, task_a1, task_a2])
            .unwrap();

        // Complete Task A3
        manager.complete_task(task_a3).unwrap();

        // Since Task A is unordered, other subtasks remain incomplete but are not active
        // Because Task A1 and Task A2 were already completed
        let active_tasks = manager.get_active_tasks();
        assert!(active_tasks.is_empty());

        // Uncomplete Task A1
        manager.uncomplete_task(task_a1).unwrap();

        // Now, Task A1 should be active again
        let active_tasks = manager.get_active_tasks();
        let active_task_ids: HashSet<usize> = active_tasks.iter().map(|t| t.id).collect();

        let expected_active = vec![task_a1];
        let expected_active_set: HashSet<usize> = expected_active.into_iter().collect();
        assert_eq!(active_task_ids, expected_active_set);

        // Complete Task A1
        manager.complete_task(task_a1).unwrap();

        // All tasks should now be completed
        let active_tasks = manager.get_active_tasks();
        assert!(active_tasks.is_empty());
    }

    #[test]
    fn test_get_parent_tasks() {
        let manager = TaskManager::new();
        let parent_id = manager.add_task("Parent Task".to_string(), true);
        let subtask_id = manager
            .add_subtask(parent_id, "Subtask".to_string())
            .unwrap();
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
async fn uncomplete_task(id: usize, task_manager: State<'_, TaskManager>) -> Result<(), String> {
    task_manager.uncomplete_task(id)
}

#[tauri::command]
async fn toggle_ordered(id: usize, task_manager: State<'_, TaskManager>) -> Result<(), String> {
    task_manager.toggle_ordered(id)
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
            uncomplete_task,
            toggle_ordered,
            get_active_tasks,
            get_subtasks,
            get_parent_tasks,
            get_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
