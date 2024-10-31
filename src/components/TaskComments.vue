<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  taskId: number;
}>();

interface Comment {
  id: number;
  text: string;
  updatedAt: string;
}

const comments = ref<Comment[]>([]);
const newComment = ref('');

// Load comments from the backend
const loadComments = async () => {
  try {
    const result = await invoke<Comment[]>('get_comments', { taskId: props.taskId });
    comments.value = result;
  } catch (error) {
    console.error('Failed to load comments:', error);
  }
};

// Add a new comment and push it to the comments array
const addComment = async () => {
  if (newComment.value.trim() !== '') {
    try {
      const result = await invoke<number>('add_comment', {
        taskId: props.taskId,
        text: newComment.value.trim(),
      });
      comments.value.push({
        id: result,
        text: newComment.value.trim(),
        updatedAt: new Date().toISOString(), // Set to current time
      });
      newComment.value = '';
    } catch (error) {
      console.error('Failed to add comment:', error);
    }
  }
};

// Format the comments for display
const formattedComments = computed(() =>
  comments.value.map((comment) => ({
    ...comment,
    updatedAt: new Date(comment.updatedAt).toLocaleString(), // Format the display of updatedAt
  }))
);

onMounted(() => {
  loadComments();
});
</script>

<template>
  <div class="comments-section">
    <div v-for="comment in formattedComments" :key="comment.id" class="comment-item">
      <p class="comment-text">{{ comment.text }}</p>
      <span class="comment-timestamp">{{ comment.updatedAt }}</span>
    </div>
    <div class="comment-input">
      <input
        v-model="newComment"
        type="text"
        placeholder="添加新的注释..."
        @keyup.enter="addComment"
      />
    </div>
  </div>
</template>

<style scoped>
.comments-section {
  padding: 8px;
  background-color: transparent;
  border-top: none;
  border-radius: 8px;
  margin-top: 16px;
}

.comment-item {
  margin-bottom: 8px;
}

.comment-text {
  color: #e1e1e1;
  font-size: 14px;
  margin: 0;
}

.comment-timestamp {
  font-size: 12px;
  color: #666666;
}

.comment-input {
  display: flex;
  margin-top: 8px;
}

.comment-input input {
  flex: 1;
  padding: 8px;
  border: none;
  background-color: transparent;
  color: #e1e1e1;
  border-bottom: 1px solid #555;
  outline: none;
}
</style>
