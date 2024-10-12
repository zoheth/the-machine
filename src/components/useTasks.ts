import { ref } from 'vue';

export function useTasks() {
    const tasks = ref([
        {
            text: '学习 Rust',
            completed: false,
            ordered: false,
            subtasks: [
                {
                    text: '阅读 Rust 官方文档',
                    completed: false,
                    ordered: false,
                    subtasks: [],
                },
                {
                    text: '编写第一个 Rust 程序',
                    completed: false,
                    ordered: false,
                    subtasks: [],
                },
            ],
        },
        {
            text: '完成 Tauri 项目',
            completed: false,
            ordered: false,
            subtasks: [
                {
                    text: '设计用户界面',
                    completed: true,
                    ordered: false,
                    subtasks: [],
                },
                {
                    text: '实现主要功能',
                    completed: false,
                    ordered: false,
                    subtasks: [
                        {
                            text: '集成文件系统访问',
                            completed: false,
                            ordered: false,
                            subtasks: [],
                        },
                        {
                            text: '打包应用',
                            completed: false,
                            ordered: false,
                            subtasks: [],
                        },
                    ],
                },
            ],
        },
    ]);

    return { tasks };
}