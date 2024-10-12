<script setup lang="ts">
import { defineProps, ref } from 'vue';

const props = defineProps<{
  tasks: { text: string; completed: boolean }[];
  taskPath: number[];
  onToggleTask: (index: number) => void;
  onNavigateToSubtasks: (index: number) => void;
}>();

const activeTaskIndex = ref<number | null>(null);
const pressProgress = ref(0);
const isShowingProgress = ref(false);

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
</script>

<template>
  <ul>
    <li
      v-for="(task, index) in tasks"
      :key="index"
      :class="{ completed: task.completed }"
    >
      <div
        class="task-item"
        @mousedown="startLongPress(index)"
        @touchstart="startLongPress(index)"
        @mouseup="cancelLongPress"
        @mouseleave="cancelLongPress"
        @touchend="cancelLongPress"
        @dblclick="handleDoubleClick(index)"
      >
        <span>{{ task.text }}</span>
        <span v-if="task.completed" class="status">已完成</span>

        <div class="progress-bar" v-if="isShowingProgress && activeTaskIndex === index">
          <div class="progress" :style="{ width: pressProgress + '%' }"></div>
        </div>
      </div>
    </li>
  </ul>
</template>

<style scoped>
.task-item {
  display: flex;
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
</style>
