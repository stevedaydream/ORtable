import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Staff } from "../types";
import { useStaffDb } from "../composables/useDatabase";

export const useStaffStore = defineStore("staff", () => {
  const staffList = ref<Staff[]>([]);
  const db = useStaffDb();

  const onCallStaff = computed(() => staffList.value.filter((s) => s.is_on_call));
  const extraVolunteers = computed(() => staffList.value.filter((s) => s.is_volunteer_extra));

  async function load() {
    staffList.value = await db.getAll();
  }

  async function add(staff: Staff) {
    const created = await db.create(staff);
    staffList.value.push(created);
    return created;
  }

  async function edit(staff: Staff) {
    const updated = await db.update(staff);
    const idx = staffList.value.findIndex((s) => s.id === staff.id);
    if (idx !== -1) staffList.value[idx] = updated;
    return updated;
  }

  async function remove(id: number) {
    await db.remove(id);
    staffList.value = staffList.value.filter((s) => s.id !== id);
  }

  return { staffList, onCallStaff, extraVolunteers, load, add, edit, remove };
});
