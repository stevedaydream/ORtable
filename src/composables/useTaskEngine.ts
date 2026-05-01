import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  DeptRule,
  ExtraComplianceResult,
  Staff,
  StaffComplianceResult,
  SurgeryTask,
  TaskWithScore,
} from "../types";

const scoredTasks = ref<TaskWithScore[]>([]);
const loading = ref(false);

export function useTaskEngine() {
  async function loadTasks(
    tasks: SurgeryTask[],
    deptRules: DeptRule[] = [],
    availableRooms: string[] = []
  ): Promise<void> {
    loading.value = true;
    try {
      scoredTasks.value = await invoke<TaskWithScore[]>("get_tasks_with_scores", {
        tasks,
        deptRules,
        availableRooms,
      });
    } finally {
      loading.value = false;
    }
  }

  async function checkStaffCompliance(
    staffList: Staff[],
    estimatedExtraEnd: number
  ): Promise<StaffComplianceResult[]> {
    return invoke<StaffComplianceResult[]>("batch_check_extra_compliance", {
      staffList,
      estimatedExtraEnd,
    });
  }

  async function checkSingleCompliance(
    todayShiftStart: number,
    nextDayShiftStart: number,
    estimatedExtraEnd: number
  ): Promise<ExtraComplianceResult> {
    return invoke<ExtraComplianceResult>("check_extra_compliance", {
      todayShiftStart,
      nextDayShiftStart,
      estimatedExtraEnd,
    });
  }

  return {
    scoredTasks,
    loading,
    loadTasks,
    checkStaffCompliance,
    checkSingleCompliance,
  };
}
