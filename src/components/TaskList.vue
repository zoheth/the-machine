<script setup lang="ts">
import { ref, watch } from 'vue';
import draggable from 'vuedraggable';
import { invoke } from '@tauri-apps/api/core';
import ContextMenu from './ContextMenu.vue';

const props = defineProps<{
  tasks: { id: number; text: string; completed: boolean }[];
  parentId?: number;
  ordered: boolean;
  onToggleTask: (index: number) => void;
  onNavigateToSubtasks: (index: number) => void;
}>();

const activeTaskIndex = ref<number | null>(null);
const pressProgress = ref(0);
const isShowingProgress = ref(false);
const editingIndex = ref<number | null>(null);
const newText = ref<string>('');
let singleClickTimer: ReturnType<typeof setTimeout> | null = null;

const selectedTaskIndex = ref<number | null>(null);

const longPressState = ref({
  timeout: null as ReturnType<typeof setTimeout> | null,
  progressInterval: null as ReturnType<typeof setInterval> | null,
});

function startLongPress(index: number) {
  activeTaskIndex.value = index;
  const task = props.tasks[index];
  pressProgress.value = task.completed ? 100 : 0;
  isShowingProgress.value = false;

  longPressState.value.timeout = setTimeout(() => {
    isShowingProgress.value = true;

    longPressState.value.progressInterval = setInterval(() => {
      updateProgress(task);
    }, 100);

    longPressState.value.timeout = setTimeout(() => {
      props.onToggleTask(index);
      resetProgress();
    }, 1000);
  }, 300);
}

function updateProgress(task: { completed: boolean }) {
  if (task.completed) {
    pressProgress.value = Math.max(0, pressProgress.value - 10);
  } else {
    pressProgress.value = Math.min(100, pressProgress.value + 10);
  }

  if (pressProgress.value === 0 || pressProgress.value === 100) {
    clearProgressInterval();
  }
}

function cancelLongPress() {
  clearTimeouts();
  clearProgressInterval();
  resetProgress();
}

function handleDoubleClick(index: number) {
  props.onNavigateToSubtasks(index);
}

function resetProgress() {
  activeTaskIndex.value = null;
  isShowingProgress.value = false;
  pressProgress.value = 0;
}

function clearTimeouts() {
  if (longPressState.value.timeout) {
    clearTimeout(longPressState.value.timeout);
    longPressState.value.timeout = null;
  }
}

function clearProgressInterval() {
  if (longPressState.value.progressInterval) {
    clearInterval(longPressState.value.progressInterval);
    longPressState.value.progressInterval = null;
  }
}

function handleSingleClick(index: number) {
  if (singleClickTimer) clearTimeout(singleClickTimer);
  singleClickTimer = setTimeout(() => {
    editingIndex.value = index;
    newText.value = props.tasks[index].text;
  }, 250);
}

function submitEdit(index: number) {
  const task = props.tasks[index];
  if (newText.value.trim() !== '') {
    invoke('update_task', { id: task.id, text: newText.value.trim() })
      .then(() => {
        editingIndex.value = null;
        task.text = newText.value.trim();
      })
      .catch(err => console.error('Failed to update task:', err));
  }
}

function handleRightClick(index: number, event: MouseEvent) {
  event.preventDefault();
  selectedTaskIndex.value = index;
}

const contextMenuItems = ref([
  { label: '删除任务', action: confirmDeleteTask }
]);

function confirmDeleteTask() {
  if (selectedTaskIndex.value !== null) {
    const task = props.tasks[selectedTaskIndex.value];

    invoke<number>('remove_task', { id: task.id })
      .then((deletedTaskCount) => {
        if (confirm(`将要删除 ${deletedTaskCount} 个任务，是否确认？`)) {
          props.tasks.splice(selectedTaskIndex.value!, 1);
        }
      })
      .catch((err) => console.error('Failed to remove task:', err));
  }
}

const tasksRef = ref([...props.tasks]); // 本地的可变 tasks

// 监听 props.tasks 的变化
watch(
  () => props.tasks,
  (newTasks) => {
    tasksRef.value = [...newTasks];
  },
  { immediate: true } // 立即执行一次以确保同步
);

function onReorder(event: any) {
  const newOrder = tasksRef.value.map(task => task.id);
  console.log('New order:', newOrder);
  if (props.parentId !== undefined) {
    invoke('reorder_subtasks', { parentId: props.parentId, newOrder: newOrder })
      .catch(err => console.error('Failed to reorder tasks:', err));
  }
}
</script>

<template>
  <component :is="ordered ? 'ol' : 'ul'">
    <draggable v-model="tasksRef" @end="onReorder" :disabled="props.parentId === undefined" item-key="task.id"
      :animation="200">
      <template #item="{ element: task, index }">
        <li :key="index" :class="{ completed: task.completed }" :draggable="props.parentId !== undefined">

          <div class="task-item" v-if="editingIndex === 99999999999">
            <input v-model="newText" @blur="submitEdit(index)" @keyup.enter="submitEdit(index)" />
          </div>

          <div class="task-item" v-else @mousedown="startLongPress(index)" @touchstart="startLongPress(index)"
            @mouseup="cancelLongPress" @mouseleave="cancelLongPress" @touchend="cancelLongPress"
            @dblclick="handleDoubleClick(task.id)" @contextmenu="handleRightClick(index, $event)"
            @click="handleSingleClick(index)">
            <span>{{ task.text }}</span>
            <span v-if="task.completed" class="status">已完成</span>

            <div class="progress-bar" v-if="isShowingProgress && activeTaskIndex === index">
              <div class="progress" :style="{ width: pressProgress + '%' }"></div>
            </div>
          </div>

          <ContextMenu :menuItems="contextMenuItems" />
        </li>
      </template>
    </draggable>

  </component>
</template>

<style scoped>
.task-item {
  display: flex;
  position: relative;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  margin-bottom: 10px;
  border-radius: 4px;
  background-color: #3c3c3c;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
  cursor: pointer;
  transition: background-color 0.3s ease;
  color: #f9f9f9;
  position: relative;
}

.task-item:hover {
  background-color: #4a4a4a;
}

.completed {
  text-decoration: line-through;
  color: #888;
}

.status {
  font-size: 0.9em;
  color: #3a7ca5;
  font-weight: bold;
}

.progress-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 4px;
  width: 100%;
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
}

.progress {
  height: 100%;
  background-color: #3a7ca5;
  transition: width 0.1s linear;
}

.context-menu {
  position: absolute;
  background-color: white;
  border: 1px solid #ccc;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
  z-index: 1000;
}

.context-menu ul {
  list-style-type: none;
  margin: 0;
  padding: 0;
}

.context-menu ul li {
  padding: 10px;
  cursor: pointer;
}

.context-menu ul li:hover {
  background-color: #f0f0f0;
}
</style>
