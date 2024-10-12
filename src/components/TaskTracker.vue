<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useTasks } from './useTasks';
import TaskList from './TaskList.vue';


const { tasks } = useTasks();
const newTask = ref('');
const router = useRouter();

const addTask = () => {
  if (newTask.value.trim() !== '') {
    tasks.value.push({
      text: newTask.value.trim(),
      completed: false,
      subtasks: [],
      ordered: false,
    });
    newTask.value = '';
  }
};

const toggleTask = (index: number) => {
  tasks.value[index].completed = !tasks.value[index].completed;
};

const navigateToSubtasks = (index: number) => {
  router.push({
    name: 'Subtasks',
    params: { taskPath: index.toString() },
  });
};
</script>

<template>
  <form class="task-container" @submit.prevent="addTask">
    <div class="task-list">
      <TaskList
        :tasks="tasks"
        :taskPath="[]"
        :onToggleTask="toggleTask"
        :onNavigateToSubtasks="navigateToSubtasks"
      />
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