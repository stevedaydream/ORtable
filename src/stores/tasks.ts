import { defineStore } from "pinia";
import { ref } from "vue";
import type { SurgeryTask, TaskWithScore } from "../types";
import { useTasksDb } from "../composables/useDatabase";
import { useTaskEngine } from "../composables/useTaskEngine";

export const useTasksStore = defineStore("tasks", () => {
  const tasks = ref<SurgeryTask[]>([]);
  const scoredTasks = ref<TaskWithScore[]>([]);
  const db = useTasksDb();
  const engine = useTaskEngine();

  async function load() {
    tasks.value = await db.getAll();
    await refreshScores();
  }

  async function refreshScores(availableRooms: string[] = []) {
    await engine.loadTasks(tasks.value, [], availableRooms);
    scoredTasks.value = engine.scoredTasks.value;
  }

  async function add(task: SurgeryTask) {
    const created = await db.create(task);
    tasks.value.push(created);
    await refreshScores();
    return created;
  }

  async function edit(task: SurgeryTask) {
    const updated = await db.update(task);
    const idx = tasks.value.findIndex((t) => t.id === task.id);
    if (idx !== -1) tasks.value[idx] = updated;
    await refreshScores();
    return updated;
  }

  async function remove(id: number) {
    await db.remove(id);
    tasks.value = tasks.value.filter((t) => t.id !== id);
    await refreshScores();
  }

  return { tasks, scoredTasks, load, refreshScores, add, edit, remove };
});
