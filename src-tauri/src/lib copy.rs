use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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
    // Cache for all tasks' predecessors
    dependency_cache: Mutex<HashMap<usize, HashSet<usize>>>,
    // Dirty flag to indicate if cache needs updating
    cache_dirty: Mutex<bool>,
}

impl TaskManager {
    fn new() -> Self {
        TaskManager {
            tasks: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
            dependency_cache: Mutex::new(HashMap::new()),
            cache_dirty: Mutex::new(true),
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
        // Mark cache as dirty since tasks have changed
        *self.cache_dirty.lock().unwrap() = true;
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
            // Inherit parent's predecessors
            {
                let parent_predecessors = &parent_task_lock.predecessors;
                subtask
                    .lock()
                    .unwrap()
                    .predecessors
                    .extend(parent_predecessors);
            }
            parent_task_lock.subtasks.push(id);
        }

        // Insert the subtask into the task map
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(id, subtask);
        }

        // Mark cache as dirty since tasks have changed
        *self.cache_dirty.lock().unwrap() = true;

        Ok(id)
    }

    // Helper method to set up predecessors among ordered subtasks
    fn set_ordered_subtasks(&self, subtasks: &Vec<usize>) -> Result<(), String> {
        let tasks_map = self.tasks.lock().unwrap();

        // Collect parent mapping for each subtask
        let subtask_parents: HashMap<usize, Option<usize>> = subtasks
            .iter()
            .map(|&subtask_id| {
                let subtask_arc = tasks_map.get(&subtask_id).unwrap();
                let subtask_lock = subtask_arc.lock().unwrap();
                (subtask_id, subtask_lock.parent)
            })
            .collect();

        for i in 0..subtasks.len() {
            let current_task_id = subtasks[i];
            let current_task_arc = tasks_map.get(&current_task_id).unwrap().clone();
            let mut current_task_lock = current_task_arc.lock().unwrap();

            // Capture current_parent outside the closure to avoid borrowing current_task_lock inside
            let current_parent = current_task_lock.parent;

            // Clone the predecessors
            let existing_predecessors = current_task_lock.predecessors.clone();

            // Filter predecessors
            let mut new_predecessors: Vec<usize> = existing_predecessors
                .into_iter()
                .filter(|&pid| {
                    // Remove predecessors that are subtasks of the same parent
                    let pred_parent = subtask_parents.get(&pid).cloned().flatten();
                    pred_parent != current_parent
                })
                .collect();

            // For all but the first subtask, set the predecessor
            if i > 0 {
                let predecessor_id = subtasks[i - 1];
                new_predecessors.push(predecessor_id);
            }

            current_task_lock.predecessors = new_predecessors;
        }

        Ok(())
    }

    // Helper method to remove predecessors among subtasks
    fn remove_subtasks_predecessors(&self, subtasks: &Vec<usize>) -> Result<(), String> {
        let tasks_map = self.tasks.lock().unwrap();

        // Collect parent mapping for each subtask
        let subtask_parents: HashMap<usize, Option<usize>> = subtasks
            .iter()
            .map(|&subtask_id| {
                let subtask_arc = tasks_map.get(&subtask_id).unwrap();
                let subtask_lock = subtask_arc.lock().unwrap();
                (subtask_id, subtask_lock.parent)
            })
            .collect();

        for &subtask_id in subtasks {
            let subtask_arc = tasks_map.get(&subtask_id).unwrap().clone();
            let mut subtask_lock = subtask_arc.lock().unwrap();

            // Capture current_parent outside the closure
            let current_parent = subtask_lock.parent;

            // Clone the predecessors
            let existing_predecessors = subtask_lock.predecessors.clone();

            // Remove predecessors that are subtasks of the same parent
            let new_predecessors: Vec<usize> = existing_predecessors
                .into_iter()
                .filter(|&pid| {
                    let pred_parent = subtask_parents.get(&pid).cloned().flatten();
                    pred_parent != current_parent
                })
                .collect();

            subtask_lock.predecessors = new_predecessors;
        }

        Ok(())
    }

    // Modified complete_task method
    fn complete_task(&self, id: usize) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap();
        let task_arc = tasks
            .get(&id)
            .ok_or(format!("Task with id: {} not found", id))?
            .clone();

        let mut task_lock = task_arc.lock().unwrap();
        task_lock.completed = true;
        drop(task_lock);

        // Mark cache as dirty since task completion may affect active tasks
        *self.cache_dirty.lock().unwrap() = true;

        Ok(())
    }

    // Modified uncomplete_task method
    fn uncomplete_task(&self, id: usize) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap();
        let task_arc = tasks
            .get(&id)
            .ok_or(format!("Task with id: {} not found", id))?
            .clone();

        let mut task_lock = task_arc.lock().unwrap();
        task_lock.completed = false;
        drop(task_lock);

        // Mark cache as dirty since uncompletion may affect dependencies
        *self.cache_dirty.lock().unwrap() = true;

        Ok(())
    }

    // Modified toggle_ordered method
    fn toggle_ordered(&self, id: usize) -> Result<(), String> {
        let tasks_map = self.tasks.lock().unwrap();
        let task_arc = tasks_map
            .get(&id)
            .ok_or(format!("Task with id: {} not found", id))?
            .clone();

        let mut task_lock = task_arc.lock().unwrap();
        let new_ordered = !task_lock.ordered;
        task_lock.ordered = new_ordered;
        let subtasks = task_lock.subtasks.clone();
        drop(task_lock);

        // Update dependencies among subtasks based on the new ordered flag
        if new_ordered {
            // For ordered tasks, set up predecessors among subtasks
            self.set_ordered_subtasks(&subtasks)?;
        } else {
            // For unordered tasks, remove predecessors among subtasks
            self.remove_subtasks_predecessors(&subtasks)?;
        }

        // Mark cache as dirty since dependencies have changed
        *self.cache_dirty.lock().unwrap() = true;

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

        // Update the predecessors among subtasks based on the parent's ordered flag
        if ordered {
            self.set_ordered_subtasks(&new_order)?;
        }

        // Mark cache as dirty since dependencies have changed
        *self.cache_dirty.lock().unwrap() = true;

        Ok(())
    }

    fn build_dependency_cache(&self) {
        let mut cache_dirty = self.cache_dirty.lock().unwrap();
        if !*cache_dirty {
            return;
        }

        let tasks_map = self.tasks.lock().unwrap();
        let mut dependency_cache = self.dependency_cache.lock().unwrap();

        dependency_cache.clear();

        for &task_id in tasks_map.keys() {
            let mut visited = HashSet::new();
            let all_preds = self.collect_all_predecessors(
                task_id,
                &tasks_map,
                &mut visited,
                &mut dependency_cache,
            );
            dependency_cache.insert(task_id, all_preds);
        }

        *cache_dirty = false;
    }

    fn collect_all_predecessors(
        &self,
        task_id: usize,
        tasks_map: &HashMap<usize, Arc<Mutex<Task>>>,
        visited: &mut HashSet<usize>,
        dependency_cache: &mut HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize> {
        if let Some(cached_preds) = dependency_cache.get(&task_id) {
            return cached_preds.clone();
        }

        if !visited.insert(task_id) {
            // Already visited; avoid cycles
            return HashSet::new();
        }

        let task_arc = tasks_map.get(&task_id).unwrap();
        // Clone necessary data while holding the lock
        let (predecessors, parent) = {
            let task_lock = task_arc.lock().unwrap();
            (task_lock.predecessors.clone(), task_lock.parent)
        }; // Release lock here

        let mut all_preds = HashSet::new();

        // Add explicit predecessors
        for &pred_id in &predecessors {
            all_preds.insert(pred_id);
            let preds_of_pred =
                self.collect_all_predecessors(pred_id, tasks_map, visited, dependency_cache);
            all_preds.extend(preds_of_pred);
        }

        // If task has a parent, add parent as predecessor
        if let Some(parent_id) = parent {
            all_preds.insert(parent_id);
            let preds_of_parent =
                self.collect_all_predecessors(parent_id, tasks_map, visited, dependency_cache);
            all_preds.extend(preds_of_parent);

            // Clone necessary data from parent
            let (parent_ordered, parent_subtasks) = {
                let parent_task_arc = tasks_map.get(&parent_id).unwrap();
                let parent_task_lock = parent_task_arc.lock().unwrap();
                (parent_task_lock.ordered, parent_task_lock.subtasks.clone())
            }; // Release parent lock here

            // For ordered parent, add previous subtasks
            if parent_ordered {
                if let Some(pos) = parent_subtasks.iter().position(|&id| id == task_id) {
                    if pos > 0 {
                        for i in 0..pos {
                            let prev_subtask_id = parent_subtasks[i];
                            all_preds.insert(prev_subtask_id);
                            let preds_of_prev_subtask = self.collect_all_predecessors(
                                prev_subtask_id,
                                tasks_map,
                                visited,
                                dependency_cache,
                            );
                            all_preds.extend(preds_of_prev_subtask);
                        }
                    }
                }
            }
        }

        dependency_cache.insert(task_id, all_preds.clone());
        all_preds
    }

    fn is_task_active(&self, task_id: usize) -> bool {
        self.build_dependency_cache();

        let tasks_map = self.tasks.lock().unwrap();

        // Clone task completion status while holding the lock
        let task_completed = {
            let task_arc = tasks_map.get(&task_id).unwrap();
            let task_lock = task_arc.lock().unwrap();
            task_lock.completed
        };

        if task_completed {
            return false;
        }

        let dependency_cache = self.dependency_cache.lock().unwrap();
        let all_preds = dependency_cache.get(&task_id).unwrap().clone(); // Clone to release lock early

        drop(tasks_map); // Release the tasks_map lock early

        for pred_id in all_preds {
            let tasks_map = self.tasks.lock().unwrap();
            let pred_task_arc = tasks_map.get(&pred_id).unwrap();
            // Clone completion status while holding the lock
            let pred_completed = {
                let pred_task_lock = pred_task_arc.lock().unwrap();
                pred_task_lock.completed
            };
            if !pred_completed {
                return false;
            }
        }

        // Task is active
        true
    }

    fn get_active_tasks(&self) -> Vec<Task> {
        self.build_dependency_cache();

        let tasks_map = self.tasks.lock().unwrap();
        let task_ids: Vec<usize> = tasks_map.keys().cloned().collect();
        drop(tasks_map); // Release lock early

        let mut active_tasks = Vec::new();

        for task_id in task_ids {
            if self.is_task_active(task_id) {
                let tasks_map = self.tasks.lock().unwrap();
                let task_arc = tasks_map.get(&task_id).unwrap();
                let task_lock = task_arc.lock().unwrap();
                active_tasks.push(task_lock.clone());
            }
        }

        active_tasks
    }
}
