import { createRouter, createWebHistory } from 'vue-router';
import { RouteLocationNormalized } from 'vue-router';
import TaskTracker from '../components/TaskTracker.vue';
import Subtasks from '../components/Subtasks.vue';

const routes = [
  {
    path: '/',
    name: 'TaskTracker',
    component: TaskTracker,
  },
  {
    path: '/tasks/:taskId',
    name: 'Subtasks',
    component: Subtasks,
    props: (route: RouteLocationNormalized) => ({ taskId: Number(route.params.taskId) }),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;