<script setup lang="ts">
import { ref, nextTick } from 'vue';
import draggable from 'vuedraggable';
import { invoke } from '@tauri-apps/api/core';
import ContextMenu from '@imengyu/vue3-context-menu'

const props = defineProps<{
  tasks: { id: number; text: string; completed: boolean }[];
  parentId?: number;
  ordered: boolean;
  onToggleTask: (index: number) => void;
  onNavigateToSubtasks: (index: number) => void;
}>();

interface Task {
  id: number;
  text: string;
  completed: boolean;
  ordered: boolean;
  subtasks: Task[];
}

const activeTaskIndex = ref<number | null>(null);
const pressProgress = ref(0);
const isShowingProgress = ref(false);
const editingIndex = ref<number | null>(null);
const newText = ref<string>('');

const longPressState = ref({
  timeout: null as ReturnType<typeof setTimeout> | null,
  progressInterval: null as ReturnType<typeof setInterval> | null,
});

function startLongPress(index: number) {
  if (editingIndex.value !== null) return;  // Prevent long press during editing
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

function submitEdit(index: number) {
  const task = props.tasks[index];
  if (newText.value.trim() !== '') {
    invoke('update_task', { id: task.id, text: newText.value.trim() })
      .then(() => {
        editingIndex.value = null;
        task.text = newText.value.trim();
      })
      .catch(err => console.error('Failed to update task:', err));
  } else {
    editingIndex.value = null;
  }
}

function handleEditBlur(index: number) {
  submitEdit(index);
}

const onContextMenu = (index: number, e: any) => {
  e.preventDefault();
  ContextMenu.showContextMenu({
    theme: 'my-theme',
    x: e.x,
    y: e.y,
    items: [
      {
        label: "编辑",
        onClick: () => {
          editingIndex.value = index;
          newText.value = props.tasks[index].text;
          nextTick(() => {
            const input = document.querySelector<HTMLInputElement>('input.editing');
            if (input) {
              input.focus();
              input.select();
            }
          });
        }
      },
      {
        label: "删除",
        onClick: () => {
          setTimeout(() => {
            confirmDeleteTask(index);  // 延迟执行删除操作
          }, 100); 
        }
      },
    ]
  });
};

function confirmDeleteTask(index: number) {
  ContextMenu.closeContextMenu();
  const task = props.tasks[index];

  invoke<Task[]>('get_subtasks', { id: task.id })
    .then((subtasks) => {
      let message = `将要删除任务: "${task.text}"`;

      if (subtasks.length > 0) {
        message += `，并且包含以下子任务:\n`;
        subtasks.forEach(subtask => {
          message += `- ${subtask.text}\n`;
        });
      }

      message += '是否确认删除？';

      if (confirm(message)) {
        invoke<number>('remove_task', { id: task.id })
          .then(() => {
            props.tasks.splice(index, 1);
          })
          .catch((err) => console.error('Failed to remove task:', err));
      }
    })
    .catch((err) => console.error('Failed to get subtasks:', err));
}

function onReorder(_event: any) {
  if (editingIndex.value !== null) return;  // Prevent reordering during editing
  const newOrder = props.tasks.map(task => task.id);
  console.log('New order:', newOrder);
  if (props.parentId !== undefined) {
    invoke('reorder_subtasks', { parentId: props.parentId, newOrder: newOrder })
      .catch(err => console.error('Failed to reorder tasks:', err));
  }
}

</script>

<template>
  <component :is="ordered ? 'ol' : 'ul'">
    <draggable :list="props.tasks" @end="onReorder" item-key="" ghost-class="ghost" :force-fallback="true"
      chosen-class="chosenClass" animation="300">
      <template #item="{ element: task, index }">
        <li :key="index" :class="{ completed: task.completed }">
            <input  v-if="editingIndex === index" type="text" v-model="newText" @blur="handleEditBlur(index)" @keyup.enter="submitEdit(index)" class="editing" />

          <div v-else class="task-item" @contextmenu="onContextMenu(index, $event)" @mousedown="startLongPress(index)"
            @touchstart="startLongPress(index)" @mouseup="cancelLongPress" @mouseleave="cancelLongPress"
            @touchend="cancelLongPress" @dblclick="handleDoubleClick(task.id)" >
            <span>{{ task.text }}</span>
            <span v-if="task.completed" class="status">已完成</span>

            <div class="progress-bar" v-if="isShowingProgress && activeTaskIndex === index">
              <div class="progress" :style="{ width: pressProgress + '%' }"></div>
            </div>
          </div>
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
}

.editing {
  box-sizing: border-box;
  width: 100%;
  display: flex;
  position: relative;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  margin-bottom: 10px;
  border-radius: 4px;
  background-color: #444;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
  cursor: pointer;
  transition: background-color 0.3s ease;
  color: #f9f9f9;
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