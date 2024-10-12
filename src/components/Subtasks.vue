<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useTasks } from './useTasks';
import TaskList from './TaskList.vue';

// 定义任务类型
interface Task {
  text: string;
  completed: boolean;
  ordered: boolean;
  subtasks: Task[];
}


// 获取路由
const route = useRoute();
const router = useRouter();

// 使用任务数据（包含假数据）
const { tasks } = useTasks();


// Get taskPath from route params and split it into an array
const taskPathParam = route.params.taskPath as string || '';
const taskPath = taskPathParam.split('/').map(Number).filter(n => !isNaN(n));

// Function to get the task by path
const getTaskByPath = (path: number[]) => {
  let currentTasks = tasks.value;
  let currentTask: Task | null = null;
  for (const index of path) {
    currentTask = currentTasks[index];
    if (!currentTask) return null;
    currentTasks = currentTask.subtasks;
  }
  return currentTask;
};

const currentTask = ref<Task | null>(getTaskByPath(taskPath));

const newSubtask = ref('');

// 添加子任务
const addSubtask = () => {
  if (newSubtask.value.trim() !== '' && currentTask.value) {
    currentTask.value.subtasks.push({
      text: newSubtask.value.trim(),
      completed: false,
      subtasks: [],
      ordered: false,
    });
    newSubtask.value = '';
  }
};

// 切换子任务的完成状态
const toggleSubtask = (index: number) => {
  if (currentTask.value) {
    currentTask.value.subtasks[index].completed = !currentTask.value.subtasks[index].completed;
  }
};

// 导航到子任务
const navigateToSubtasks = (subtaskIndex: number) => {
  const newTaskPath = [...taskPath, subtaskIndex].join('/');
  router.push({
    name: 'Subtasks',
    params: { taskPath: newTaskPath },
  });
};

// 切换排序
const toggleOrdered = () => {
  if (currentTask.value) {
    currentTask.value.ordered = !currentTask.value.ordered;
  }
};

// 生成层次结构
const hierarchy = computed(() => {
  const hierarchyTasks = [];
  let path: string[] = [];
  let currentTasks = tasks.value;
  for (const index of taskPath) {
    const task = currentTasks[index];
    path.push(index.toString());
    hierarchyTasks.push({
      text: task.text,
      path: [...path],
    });
    currentTasks = task.subtasks;
  }
  return hierarchyTasks;
});

// 返回主界面
const returnToMain = () => {
  router.push({ name: 'TaskTracker' });
};

// 导航到指定任务
const navigateToTask = (path: string[]) => {
  router.push({ name: 'Subtasks', params: { taskPath: path.join('/') } });
};
</script>

<template>
  <div class="task-container">
    <div class="hierarchy">
      <span v-for="(item, index) in hierarchy" :key="index">
        <span @click="navigateToTask(item.path)">{{ item.text }}</span>
        <span v-if="index < hierarchy.length - 1"> &gt; </span>
      </span>
    </div>
    <div class="controls">
      <button @click="returnToMain">返回主界面</button>
      <!-- 检查 currentTask 是否为 null，显示对应文本 -->
      <button v-if="currentTask" @click="toggleOrdered">
        {{ currentTask.ordered ? '切换为无序' : '切换为有序' }}
      </button>
      <!-- 如果 currentTask 是 null，可以显示一个占位文本 -->
      <button v-else disabled>加载中...</button>
    </div>
    <!-- 检查 currentTask 是否为 null，防止访问错误 -->
    <TaskList
      v-if="currentTask"
      :tasks="currentTask.subtasks"
      :taskPath="taskPath"
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
/* Your styles here */
/* Add styles for ordered list */
ul {
  list-style-type: none;
  padding: 0;
}

ul.ordered {
  list-style-type: decimal;
  padding-left: 20px;
}
</style>