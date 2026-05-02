import { defineStore } from "pinia";
import { ref } from "vue";
import type { DeptRule } from "../types";
import { useDeptRulesDb } from "../composables/useDatabase";

export const useDeptRulesStore = defineStore("deptRules", () => {
  const rules = ref<DeptRule[]>([]);
  const db = useDeptRulesDb();

  async function load() {
    rules.value = await db.getAll();
  }

  async function upsert(rule: DeptRule) {
    const updated = await db.upsert(rule);
    const idx = rules.value.findIndex((r) => r.dept === rule.dept);
    if (idx !== -1) {
      rules.value[idx] = updated;
    } else {
      rules.value.push(updated);
    }
    return updated;
  }

  async function remove(dept: string) {
    await db.remove(dept);
    rules.value = rules.value.filter((r) => r.dept !== dept);
  }

  return { rules, load, upsert, remove };
});
