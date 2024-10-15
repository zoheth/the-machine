<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import TaskList from './TaskList.vue';

const props = defineProps<{
  taskId: number;
}>();

interface Task {
  id: number;
  text: string;
  completed: boolean;
  ordered: boolean;
  subtasks: Task[];
}

const router = useRouter();

let currentId = props.taskId;


const currentTask = ref<Task | null>(null);
const hierarchy = ref<{ text: string; id: number }[]>([]);
const newSubtask = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

const loadParentTasks = async () => {
  try {
    const result = await invoke<{ text: string; id: number }[]>('get_parent_tasks', { id: currentId });
    console.log(result);
    hierarchy.value = result.reverse();
  } catch (error) {
    console.error('Failed to load parent tasks:', error);
  }
};

const loadTask = async () => {
  try {
    const result = await invoke<Task>('get_task', { id: currentId });
    console.log(result);
    currentTask.value = result;

    const subtasks = await invoke<Task[]>('get_subtasks', { id: currentId });
    console.log("Subtasks received from backend:", result);
    currentTask.value.subtasks = subtasks;

  } catch (error) {
    console.error('Failed to load subtasks:', error);
  }
};

const addSubtask = async () => {
  if (newSubtask.value.trim() !== '' && currentTask.value) {
    try {
      const result = await invoke<number>('add_subtask', {
        parentId: currentTask.value.id,
        text: newSubtask.value.trim(),
      });
      currentTask.value.subtasks.push({
        id: result,
        text: newSubtask.value.trim(),
        completed: false,
        ordered: true,
        subtasks: [],
      });
      newSubtask.value = '';
    } catch (error) {
      console.error('Failed to add subtask:', error);
    }
  }
  if (inputRef.value) {
    inputRef.value.blur();
  }
};

const toggleSubtask = async (index: number) => {
  if (currentTask.value) {
    if (currentTask.value.subtasks[index].completed) {
      await invoke('uncomplete_task', { id: currentTask.value.subtasks[index].id });
      currentTask.value.subtasks[index].completed = false;
    }
    else {
      await invoke('complete_task', { id: currentTask.value.subtasks[index].id });
      currentTask.value.subtasks[index].completed = true;
    }
    loadTask();
  }
};

const toggleOrdered = async () => {
  if (currentTask.value) {
    await invoke('toggle_ordered', { id: currentTask.value.id });
    currentTask.value.ordered = !currentTask.value.ordered;
  }
};

const navigateToSubtasks = (index: number) => {
  console.log('Navigating to:', index)
  currentId = index;
  loadParentTasks();
  loadTask();
};

const returnToMain = () => {
  router.push({ name: 'TaskTracker' });
};

onMounted(() => {
  loadParentTasks();
  loadTask();
});

</script>

<template>
  <div class="page-header">
    <img @click="returnToMain" class="back-button icon-button" src="../assets/menu-button.png" alt="返回主界面" />
    <div class="hierarchy">
      <span v-for="(item, index) in hierarchy" :key="item.id" class="hierarchy-item">
        <span @click="navigateToSubtasks(item.id)" class="clickable">
          {{ item.text }}
        </span>
        <span v-if="index < hierarchy.length - 1" class="hierarchy-separator"> &gt; </span>
      </span>
    </div>
  </div>

  <form class="task-container" @submit.prevent="addSubtask">
    <div class="task-header">
      <h3>{{ currentTask ? currentTask.text : '任务详情' }}</h3>
      <div class="toggle-icons">
        <img v-if="currentTask?.ordered" src="../assets/ordered_list.png" @click="toggleOrdered" alt="有序"
          class="toggle-icon icon-button" />
        <img v-else src="../assets/unordered_list.png" @click="toggleOrdered" alt="无序"
          class="toggle-icon icon-button" />
      </div>
    </div>
    <TaskList v-if="currentTask" :tasks="currentTask.subtasks" :parentId="currentTask.id" :ordered="currentTask.ordered"
      :onToggleTask="toggleSubtask" :onNavigateToSubtasks="navigateToSubtasks" />
    <div class="task-input">
      <input ref="inputRef" v-model="newSubtask" type="text" placeholder="添加新的子任务..." />
    </div>
  </form>
</template>

<style scoped>
.page-header {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  background-color: #2b2b2b;
  border-bottom: 1px solid #444;
  z-index: 10;
  box-sizing: border-box;
}

.icon-button {
  width: 28px;
  height: 28px;
  cursor: pointer;
  margin-right: 16px;
  opacity: 0.6;
  transition: opacity 0.3s;
  filter: invert(0);
}

.toggle-icon {
  opacity: 0.3;
}

.icon-button:hover {
  opacity: 0.9;
}

.hierarchy {
  display: flex;
  align-items: center;
  color: #e1e1e1;
  font-size: 14px;
}

.hierarchy-item {
  display: flex;
  align-items: center;
  cursor: pointer;
  transition: color 0.3s ease;
}

.hierarchy-item:hover {
  color: #ffffff;
}

.hierarchy-separator {
  color: #666666;
  margin: 0 8px;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background-color: #2b2b2b;
  color: #e1e1e1;
  border-bottom: 1px solid #444;
  border-radius: 8px;
  margin-bottom: 16px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.task-header h3 {
  font-size: 18px;
  font-weight: bold;
}

@media (prefers-color-scheme: dark) {
  .icon-button {
    filter: invert(1);
  }

  .page-header {
    background-color: #1e1e1e;
    border-bottom: 1px solid #333;
  }

  .task-input input {
    border: 1px solid #555;
    background-color: #2e2e2e;
    color: #e1e1e1;
  }

  .task-header {
    background-color: #1e1e1e;
    border-bottom: 1px solid #333;
  }
}
</style>
