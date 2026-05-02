import { defineStore } from "pinia";
import { ref } from "vue";
import type { StaffAssignment } from "../types";
import { useStaffAssignmentsDb } from "../composables/useDatabase";

export const useAssignmentsStore = defineStore("assignments", () => {
  const assignments = ref<StaffAssignment[]>([]);
  const db = useStaffAssignmentsDb();

  async function load(date: string) {
    assignments.value = await db.getByDate(date);
  }

  async function add(assignment: StaffAssignment) {
    const created = await db.addOne(assignment);
    assignments.value.push(created);
    return created;
  }

  async function remove(id: number) {
    await db.removeOne(id);
    assignments.value = assignments.value.filter((a) => a.id !== id);
  }

  return { assignments, load, add, remove };
});
