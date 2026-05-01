import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SyncTimestamps } from "../types";

export type SyncStatus = "idle" | "pushing" | "pulling" | "ok" | "error";

const status = ref<SyncStatus>("idle");
const message = ref("");
const timestamps = ref<SyncTimestamps>({ last_push_at: null, last_pull_at: null });

export function useSync() {
  async function push(): Promise<string> {
    status.value = "pushing";
    message.value = "";
    try {
      const msg = await invoke<string>("sync_push");
      status.value = "ok";
      message.value = msg;
      await refreshTimestamps();
      return msg;
    } catch (e) {
      status.value = "error";
      message.value = String(e);
      console.error(`[sync] push 失敗: ${e}`);
      throw e;
    }
  }

  async function pull(): Promise<string> {
    status.value = "pulling";
    message.value = "";
    try {
      const msg = await invoke<string>("sync_pull");
      status.value = "ok";
      message.value = msg;
      await refreshTimestamps();
      return msg;
    } catch (e) {
      status.value = "error";
      message.value = String(e);
      console.error(`[sync] pull 失敗: ${e}`);
      throw e;
    }
  }

  async function refreshTimestamps(): Promise<void> {
    try {
      timestamps.value = await invoke<SyncTimestamps>("get_sync_timestamps");
    } catch (_) {}
  }

  function formatTs(ts: number | null): string {
    if (!ts) return "從未";
    return new Date(ts * 1000).toLocaleString("zh-TW", {
      month: "2-digit", day: "2-digit",
      hour: "2-digit", minute: "2-digit",
    });
  }

  return { status, message, timestamps, push, pull, refreshTimestamps, formatTs };
}
