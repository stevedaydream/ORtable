import { invoke } from "@tauri-apps/api/core";
import type { DeptRule, Room, RoomScheduleEntry, Staff, SurgeryTask } from "../types";

// ── Surgery Tasks ─────────────────────────────────────────────────────────────

export function useTasksDb() {
  const getAll = () => invoke<SurgeryTask[]>("get_all_tasks");
  const create = (task: SurgeryTask) => invoke<SurgeryTask>("create_task", { task });
  const update = (task: SurgeryTask) => invoke<SurgeryTask>("update_task", { task });
  const remove = (id: number) => invoke<void>("delete_task", { id });
  return { getAll, create, update, remove };
}

// ── Staff ─────────────────────────────────────────────────────────────────────

export function useStaffDb() {
  const getAll = () => invoke<Staff[]>("get_all_staff");
  const create = (staff: Staff) => invoke<Staff>("create_staff", { staff });
  const update = (staff: Staff) => invoke<Staff>("update_staff", { staff });
  const remove = (id: number) => invoke<void>("delete_staff", { id });
  return { getAll, create, update, remove };
}

// ── Dept Rules ────────────────────────────────────────────────────────────────

export function useDeptRulesDb() {
  const getAll = () => invoke<DeptRule[]>("get_dept_rules");
  const upsert = (rule: DeptRule) => invoke<DeptRule>("upsert_dept_rule", { rule });
  const remove = (dept: string) => invoke<void>("delete_dept_rule", { dept });
  return { getAll, upsert, remove };
}

// ── Rooms ─────────────────────────────────────────────────────────────────────

export function useRoomsDb() {
  const getAll = () => invoke<Room[]>("get_all_rooms");
  const create = (room: Room) => invoke<Room>("create_room", { room });
  const update = (room: Room) => invoke<Room>("update_room", { room });
  const remove = (id: number) => invoke<void>("delete_room", { id });
  return { getAll, create, update, remove };
}

// ── Room Shifts ───────────────────────────────────────────────────────────────

export function useRoomShiftsDb() {
  const getByDate = (date: string) =>
    invoke<RoomScheduleEntry[]>("get_room_shifts_by_date", { date });
  const replaceByMonth = (month: string, entries: RoomScheduleEntry[]) =>
    invoke<void>("replace_room_shifts_by_month", { month, entries });
  return { getByDate, replaceByMonth };
}

// ── Settings ──────────────────────────────────────────────────────────────────

export function useSettings() {
  const getGasUrl = () => invoke<string | null>("get_gas_url");
  const setGasUrl = (url: string) => invoke<void>("set_gas_url", { url });
  return { getGasUrl, setGasUrl };
}
