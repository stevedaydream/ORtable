import { ref } from "vue";

export type UpdateStatus = "idle" | "checking" | "available" | "downloading" | "up-to-date" | "error";

const status = ref<UpdateStatus>("idle");
const updateVersion = ref<string | null>(null);

export function useUpdater() {
  async function checkForUpdate() {
    status.value = "checking";
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update?.available) {
        status.value = "available";
        updateVersion.value = update.version;
        return update;
      }
      status.value = "up-to-date";
      return null;
    } catch (_) {
      // Updater not configured or no network — silently ignore
      status.value = "idle";
      return null;
    }
  }

  async function installUpdate() {
    status.value = "downloading";
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update?.available) {
        await update.downloadAndInstall();
      }
    } catch (e) {
      status.value = "error";
      throw e;
    }
  }

  return { status, updateVersion, checkForUpdate, installUpdate };
}
