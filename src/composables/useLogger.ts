import { ref } from "vue";

interface LogEntry {
  id: number;
  time: string;
  level: "error" | "warn" | "info";
  message: string;
  detail?: string;
}

const MAX_LOGS = 300;
const logs = ref<LogEntry[]>([]);
let idCounter = 0;
let initialized = false;

function addLog(level: LogEntry["level"], message: string, detail?: string) {
  if (logs.value.length >= MAX_LOGS) logs.value.shift();
  logs.value.push({
    id: ++idCounter,
    time: new Date().toTimeString().slice(0, 8),
    level,
    message,
    detail,
  });
}

function initLogger() {
  if (initialized) return;
  initialized = true;

  const origError = console.error.bind(console);
  const origWarn = console.warn.bind(console);

  console.error = (...args) => {
    addLog("error", args.map(String).join(" "));
    origError(...args);
  };
  console.warn = (...args) => {
    addLog("warn", args.map(String).join(" "));
    origWarn(...args);
  };

  window.onerror = (msg, _src, _line, _col, err) => {
    addLog("error", String(msg), err?.stack);
    return false;
  };

  window.addEventListener("unhandledrejection", (e) => {
    addLog("error", `Unhandled: ${e.reason}`, e.reason?.stack);
  });

  const origFetch = window.fetch.bind(window);
  window.fetch = async (...args) => {
    try {
      const res = await origFetch(...args);
      if (res.status >= 400) {
        addLog("error", `fetch ${res.status}: ${args[0]}`);
      }
      return res;
    } catch (err) {
      addLog("error", `fetch error: ${args[0]}`, String(err));
      throw err;
    }
  };
}

async function exportLogs() {
  try {
    const { writeTextFile, BaseDirectory } = await import("@tauri-apps/plugin-fs");
    const filename = `or-triage-log-${Date.now()}.txt`;
    const content = logs.value
      .map((l) => `[${l.time}] ${l.level.toUpperCase()} ${l.message}${l.detail ? "\n" + l.detail : ""}`)
      .join("\n");
    await writeTextFile(filename, content, { baseDir: BaseDirectory.Desktop });
  } catch (e) {
    addLog("error", `exportLogs failed: ${e}`);
  }
}

function clearLogs() {
  logs.value = [];
}

export function useLogger() {
  return { logs, addLog, initLogger, exportLogs, clearLogs };
}
