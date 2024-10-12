import { createRouter, createWebHistory } from 'vue-router';
import TaskTracker from '../components/TaskTracker.vue'; // Main Task Page
import Subtasks from '../components/Subtasks.vue'; // Subtasks Page

const routes = [
  {
    path: '/',
    name: 'TaskTracker',
    component: TaskTracker,
  },
  {
    path: '/tasks/:taskPath(.*)*',
    name: 'Subtasks',
    component: Subtasks,
    props: true,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;