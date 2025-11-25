import { invoke } from "@tauri-apps/api/core";

export interface DriveConfig {
  enabled: boolean;
  read: boolean;
  write: boolean;
  volume: number;
}

export interface Drive {
  name: string;
  model: string;
  size_display: string;
  config: DriveConfig;
}

export interface Config {
  drives: Record<string, DriveConfig>;
  active: boolean;
  poll_interval_ms: number;
}

export async function getDrives(): Promise<Drive[]> {
  return await invoke("get_drives");
}

export async function loadConfig(): Promise<Config> {
  return await invoke("load_config_command");
}

export async function saveConfig(config: Config): Promise<void> {
  return await invoke("save_config_command", { config });
}

export async function updateDriveConfig(
  deviceName: string,
  driveConfig: DriveConfig
): Promise<void> {
  return await invoke("update_drive_config", {
    deviceName,
    driveConfig,
  });
}

export async function updateGlobalConfig(
  active?: boolean,
  pollIntervalMs?: number
): Promise<void> {
  return await invoke("update_global_config", {
    active,
    pollIntervalMs,
  });
}
