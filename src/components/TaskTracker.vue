<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import TaskList from './TaskList.vue';

interface Task {
  id: number;
  text: string;
  completed: boolean;
  ordered: boolean;
  subtasks: Task[];
}

const tasks = ref<Task[]>([]);
const newTask = ref('');
const router = useRouter();

const loadTasks = async () => {
  try {
    const result = await invoke<Task[]>('get_active_tasks'); // 调用后端的 get_active_tasks 命令
    tasks.value = result; // 将结果绑定到 tasks 列表
  } catch (error) {
    console.error('Failed to load tasks:', error);
  }
};

const addTask = async () => {
  if (newTask.value.trim() !== '') {
    try {
      const result = await invoke<number>('add_task', {
        text: newTask.value.trim(),
        ordered: false,
      });
      tasks.value.push({
        id: result, // Rust 返回的任务 ID
        text: newTask.value.trim(),
        completed: false,
        subtasks: [],
        ordered: true,
      });
      newTask.value = '';
    } catch (error) {
      console.error('Failed to add task:', error);
    }
  }
};

const toggleTask = async (index: number) => {
  if (tasks.value[index]) {
    if (tasks.value[index].completed) {
      await invoke('uncomplete_task', { id: tasks.value[index].id });
      tasks.value[index].completed = false;
    }
    else {
      await invoke('complete_task', { id: tasks.value[index].id });
      tasks.value[index].completed = true;
    }
  }
  loadTasks();
};

const navigateToSubtasks = (index: number) => {
  router.push({
    name: 'Subtasks',
    params: { taskId: index },
  });
};

onMounted(() => {
  loadTasks();
});
</script>

<template>
  <form class="task-container" @submit.prevent="addTask">
    <div class="task-list">
      <TaskList :tasks="tasks" :onToggleTask="toggleTask" :onNavigateToSubtasks="navigateToSubtasks" />
    </div>
    <div class="task-input">
      <input v-model="newTask" type="text" placeholder="添加新的任务..." />
      <button type="submit">添加任务</button>
    </div>
  </form>
</template>

<style scoped>
.app-background {
  background-color: #1e1e1e;
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  border: none;
}

.task-input button:hover {
  background-color: #2a5b75;
}

.task-list {
  list-style: none;
  padding: 0;
}
</style>