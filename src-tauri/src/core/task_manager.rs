use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: usize,
    text: String,
    completed: bool,
    ordered: bool,
    subtasks: Vec<usize>,
    parent: Option<usize>,
}

impl Task {
    fn new(id: usize, text: String, ordered: bool) -> Self {
        Task {
            id,
            text,
            completed: false,
            ordered,
            subtasks: Vec::new(),
            parent: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TaskManagerData {
    tasks: Vec<Task>,
    root_tasks: Vec<usize>,
    next_id: usize,
}

pub struct TaskManager {
    pub tasks: Mutex<HashMap<usize, Arc<Mutex<Task>>>>,
    root_tasks: Mutex<Vec<usize>>,
    next_id: Mutex<usize>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: Mutex::new(HashMap::new()),
            root_tasks: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap();
        let root_tasks = self.root_tasks.lock().unwrap();
        let next_id = *self.next_id.lock().unwrap();

        let task_data: Vec<Task> = tasks
            .values()
            .map(|task_arc| task_arc.lock().unwrap().clone())
            .collect();

        let data = TaskManagerData {
            tasks: task_data,
            root_tasks: root_tasks.clone(),
            next_id,
        };

        let file = File::create(file_path).map_err(|e| format!("Failed to create file: {}", e))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &data)
            .map_err(|e| format!("Failed to write data to file: {}", e))?;

        Ok(())
    }

    pub fn load_from_file(&self, file_path: &str) -> Result<(), String> {
        let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        let data: TaskManagerData = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to read data from file: {}", e))?;

        let mut tasks_map = self.tasks.lock().unwrap();
        let mut root_task_ids = self.root_tasks.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        tasks_map.clear();
        root_task_ids.clear();

        for task in data.tasks {
            let task_id = task.id;
            let task_arc = Arc::new(Mutex::new(task));
            tasks_map.insert(task_id, task_arc);
        }

        *root_task_ids = data.root_tasks;
        *next_id = data.next_id;

        Ok(())
    }

    fn generate_id(&self) -> usize {
        let mut id = self.next_id.lock().unwrap();
        let current_id = *id;
        *id += 1;
        current_id
    }

    pub fn add_task(&self, text: String, ordered: bool) -> usize {
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

    pub fn add_subtask(&self, parent_id: usize, text: String) -> Result<usize, String> {
        let id = self.generate_id();
        let subtask = Arc::new(Mutex::new(Task::new(id, text.clone(), true)));

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

        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, subtask);
        }

        Ok(id)
    }

    pub fn update_task_text(&self, id: usize, text: String) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(&id)
            .ok_or(format!("Task with id: {} not found", id))?;
        let mut task_lock = task.lock().unwrap();
        task_lock.text = text;
        Ok(())
    }

    pub fn complete_task(&self, id: usize) -> Result<(), String> {
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

    pub fn uncomplete_task(&self, id: usize) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(&id)
            .ok_or(format!("Task with id: {} not found", id))?;
        let mut task_lock = task.lock().unwrap();
        task_lock.completed = false;
        Ok(())
    }

    pub fn toggle_ordered(&self, id: usize) -> Result<(), String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(&id)
            .ok_or(format!("Task with id: {} not found", id))?;
        let mut task_lock = task.lock().unwrap();
        task_lock.ordered = !task_lock.ordered;
        Ok(())
    }

    // Method to adjust the order of subtasks
    pub fn reorder_subtasks(&self, parent_id: usize, new_order: Vec<usize>) -> Result<(), String> {
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
        drop(parent_task_lock);

        Ok(())
    }

    pub fn get_active_tasks(&self) -> Vec<Task> {
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

        let root_task_ids = {
            let root_tasks = self.root_tasks.lock().unwrap();
            root_tasks.clone()
        };

        let mut active_tasks = Vec::new();

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
            active_tasks.push(task.clone());
            return;
        }

        let mut all_subtasks_completed = true;

        if task.ordered {
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
            for &subtask_id in &task.subtasks {
                if let Some(subtask) = tasks_map.get(&subtask_id) {
                    if !subtask.completed {
                        self.collect_active_tasks(subtask, tasks_map, active_tasks);
                        all_subtasks_completed = false;
                    }
                }
            }
        }

        if all_subtasks_completed {
            active_tasks.push(task.clone());
        }
    }

    pub fn remove_task_recursive(&self, task_id: usize) -> Result<usize, String> {
        let task_arc = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .get(&task_id)
                .ok_or(format!("Task with id: {} not found", task_id))?
                .clone()
        };

        let subtasks = {
            let task_lock = task_arc.lock().unwrap();
            task_lock.subtasks.clone()
        };

        let mut delete_count = 1;

        for subtask_id in subtasks {
            delete_count += self.remove_task_recursive(subtask_id)?;
        }

        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.remove(&task_id);
        }

        {
            let mut root_tasks = self.root_tasks.lock().unwrap();
            if let Some(pos) = root_tasks.iter().position(|&id| id == task_id) {
                root_tasks.remove(pos);
            }
        }

        Ok(delete_count)
    }

    pub fn get_subtasks_recursive(&self, id: usize, max_count: usize) -> Result<Vec<Task>, String> {
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

        let mut subtasks: Vec<Task> = Vec::new();
        let mut subtasks_to_process = subtasks_ids;
        let mut processed_count = 0;

        while let Some(subtask_id) = subtasks_to_process.pop() {
            if let Some(subtask) = tasks_map.get(&subtask_id) {
                subtasks.push(subtask.lock().unwrap().clone());
                processed_count += 1;

                if processed_count >= max_count {
                    break;
                }

                let subtask_lock = subtask.lock().unwrap();
                subtasks_to_process.extend(subtask_lock.subtasks.iter().cloned());
            }
        }

        Ok(subtasks)
    }

    pub fn get_subtasks(&self, id: usize) -> Result<Vec<Task>, String> {
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

    pub fn get_parent_tasks(&self, task_id: usize) -> Result<Vec<Task>, String> {
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

    pub fn get_task(&self, id: usize) -> Option<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).map(|t| t.lock().unwrap().clone())
    }
}
