<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core'; 
import TaskList from './TaskList.vue';

interface Task {
  id: number;
  text: string;
  completed: boolean;
  ordered: boolean;
  subtasks: Task[];
}

const route = useRoute();
const router = useRouter();

let currentId = Number(route.params.taskId);;


const currentTask = ref<Task | null>(null);
const hierarchy = ref<{ text: string; id: number }[]>([]);
const newSubtask = ref('');

const loadParentTasks = async () => {
  try {
    const result = await invoke<{ text: string; id: number }[]>('get_parent_tasks', { id: currentId });
    console.log(result); 
    hierarchy.value = result.reverse();
  } catch (error) {
    console.error('Failed to load parent tasks:', error);
  }
};

const loadSubtasks = async () => {
  try {
    const result = await invoke<Task[]>('get_subtasks', { id: currentId });
    console.log("Subtasks received from backend:", result);
    currentTask.value = {
      id: Number(currentId),
      text: hierarchy.value[hierarchy.value.length - 1]?.text || '',
      completed: false,
      ordered: false,
      subtasks: result,
    };
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
      currentTask.value.subtasks = [
        ...currentTask.value.subtasks,
        {
          id: result,
          text: newSubtask.value.trim(),
          completed: false,
          subtasks: [],
          ordered: false,
        },
      ];
      newSubtask.value = '';
    } catch (error) {
      console.error('Failed to add subtask:', error);
    }
  }
};

const toggleSubtask = (index: number) => {
  if (currentTask.value) {
    currentTask.value.subtasks[index].completed = !currentTask.value.subtasks[index].completed;
  }
};

const toggleOrdered = () => {
  if (currentTask.value) {
    currentTask.value.ordered = !currentTask.value.ordered;
  }
};

const navigateToSubtasks = (index: number) => {
  console.log('Navigating to:', index)
  currentId = index;
  loadParentTasks();
  loadSubtasks();
};

const returnToMain = () => {
  router.push({ name: 'TaskTracker' });
};

onMounted(() => {
  loadParentTasks();
  loadSubtasks();
});

</script>

<template>
  <div class="task-container">
    <div class="hierarchy">
      <span v-for="(item, index) in hierarchy" :key="item.id" class="hierarchy-item">
        <span @click="navigateToSubtasks(item.id)" class="clickable">
          {{ item.text }}
        </span>
        <span v-if="index < hierarchy.length - 1" class="hierarchy-separator"> &gt; </span>
      </span>
    </div>
    <div class="controls">
      <button @click="returnToMain">返回主界面</button>
      <button v-if="currentTask" @click="toggleOrdered">
        {{ currentTask.ordered ? '切换为无序' : '切换为有序' }}
      </button>
      <button v-else disabled>加载中...</button>
    </div>
    <TaskList
      v-if="currentTask"
      :tasks="currentTask.subtasks"
      :onToggleTask="toggleSubtask"
      :onNavigateToSubtasks="navigateToSubtasks"
    />
    <div class="task-input">
      <input v-model="newSubtask" type="text" placeholder="添加新的子任务..." />
      <button @click="addSubtask">添加子任务</button>
    </div>
  </div>
</template>

<style scoped>
ul {
  list-style-type: none;
  padding: 0;
}

ul.ordered {
  list-style-type: decimal;
  padding-left: 20px;
}

.hierarchy {
  display: flex;
  align-items: center;
  padding: 8px 16px;         /* 内边距 */
  border-radius: 4px;        /* 圆角 */
}

.hierarchy-item {
  display: flex;
  align-items: center;
}

.hierarchy-separator {
  color: #666666;             /* 分隔符颜色 */
  margin: 0 8px;              /* 分隔符左右的间距 */
  font-size: 14px;
}

.hierarchy-item {
  color: #e1e1e1;            /* 字体颜色 */
  font-size: 14px;
  cursor: pointer;
  transition: color 0.3s ease; /* 颜色过渡效果 */
}

.hierarchy-item:hover {
  color: #ffffff;  /* 悬停时的颜色 */
  background-color: #454545; /* 悬停时的背景颜色 */
}

.hierarchy-item:active {
  color: #003d80; /* 点击时的颜色 */
}

span {
  margin-right: 5px;
}
</style>