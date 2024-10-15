#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::task_manager::TaskManager;
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
