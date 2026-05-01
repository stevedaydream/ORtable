import { defineStore } from "pinia";
import { ref } from "vue";
import type { Room } from "../types";
import { useRoomsDb } from "../composables/useDatabase";

export const useRoomsStore = defineStore("rooms", () => {
  const rooms = ref<Room[]>([]);
  const db = useRoomsDb();

  async function load() {
    rooms.value = await db.getAll();
  }

  async function add(room: Room) {
    const created = await db.create(room);
    rooms.value.push(created);
    return created;
  }

  async function edit(room: Room) {
    const updated = await db.update(room);
    const idx = rooms.value.findIndex((r) => r.id === room.id);
    if (idx !== -1) rooms.value[idx] = updated;
    return updated;
  }

  async function remove(id: number) {
    await db.remove(id);
    rooms.value = rooms.value.filter((r) => r.id !== id);
  }

  return { rooms, load, add, edit, remove };
});
