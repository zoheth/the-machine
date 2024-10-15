<template>
    <div v-if="isVisible" class="custom-context-menu"
        :style="{ top: menuPosition.y + 'px', left: menuPosition.x + 'px' }" ref="menuRef">
        <ul>
            <li v-for="(item, index) in props.menuItems" :key="index" @click="onMenuClick(item.action)">
                {{ item.label }}
            </li>
        </ul>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick, defineProps } from 'vue';

type MenuItem = {
    label: string;
    action: () => void;
};

const props = defineProps<{ menuItems: MenuItem[] }>();

const isVisible = ref(false);
const menuPosition = ref({ x: 0, y: 0 });
const menuRef = ref<HTMLElement | null>(null);

const showMenu = (event: MouseEvent) => {
    event.preventDefault();
    menuPosition.value = { x: event.clientX, y: event.clientY };
    isVisible.value = true;
    nextTick(() => {
        const menu = menuRef.value;
        if (menu) {
            const { innerWidth, innerHeight } = window;
            if (menuPosition.value.x + menu.offsetWidth > innerWidth) {
                menuPosition.value.x = innerWidth - menu.offsetWidth;
            }
            if (menuPosition.value.y + menu.offsetHeight > innerHeight) {
                menuPosition.value.y = innerHeight - menu.offsetHeight;
            }
        }
    });
};

const hideMenu = () => {
    isVisible.value = false;
};

const onMenuClick = (action: () => void) => {
    action();
    hideMenu();
};

const handleClickOutside = (event: MouseEvent) => {
    if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
        hideMenu();
    }
};

onMounted(() => {
    document.addEventListener('contextmenu', showMenu);
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('scroll', hideMenu, true);
});

onBeforeUnmount(() => {
    document.removeEventListener('contextmenu', showMenu);
    document.removeEventListener('click', handleClickOutside);
    document.removeEventListener('scroll', hideMenu, true);
});
</script>

<style scoped>
.custom-context-menu {
    position: absolute;
    z-index: 1000;
    background: #fff;
    border: 1px solid #ddd;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.15);
    border-radius: 4px;
    padding: 8px 0;
    min-width: 150px;
}

.custom-context-menu ul {
    list-style: none;
    margin: 0;
    padding: 0;
}

.custom-context-menu li {
    padding: 8px 16px;
    cursor: pointer;
    transition: background 0.2s ease;
}

.custom-context-menu li:hover {
    background: #f0f0f0;
}
</style>