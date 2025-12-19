import { createTauriService } from "./tauriService";

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
  theme: string;
}

const service = createTauriService();

export async function getDrives(): Promise<Drive[]> {
  return await service.getDrives();
}

export async function loadConfig(): Promise<Config> {
  return await service.loadConfig();
}

export async function saveConfig(config: Config): Promise<void> {
  return await service.saveConfig(config);
}

export async function updateDriveConfig(
  deviceName: string,
  driveConfig: DriveConfig
): Promise<void> {
  return await service.updateDriveConfig(deviceName, driveConfig);
}

export async function updateGlobalConfig(
  active?: boolean,
  pollIntervalMs?: number,
  theme?: string
): Promise<void> {
  return await service.updateGlobalConfig(active, pollIntervalMs, theme);
}
