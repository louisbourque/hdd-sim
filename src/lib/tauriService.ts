import { invoke } from "@tauri-apps/api/core"
import type { Drive, DriveConfig, Config } from "./tauri"

export interface TauriService {
  getDrives(): Promise<Drive[]>
  loadConfig(): Promise<Config>
  saveConfig(config: Config): Promise<void>
  updateDriveConfig(deviceName: string, driveConfig: DriveConfig): Promise<void>
  updateGlobalConfig(active?: boolean, pollIntervalMs?: number, theme?: string): Promise<void>
}

class DefaultTauriService implements TauriService {
  async getDrives(): Promise<Drive[]> {
    return await invoke("get_drives")
  }

  async loadConfig(): Promise<Config> {
    return await invoke("load_config_command")
  }

  async saveConfig(config: Config): Promise<void> {
    return await invoke("save_config_command", { config })
  }

  async updateDriveConfig(
    deviceName: string,
    driveConfig: DriveConfig
  ): Promise<void> {
    return await invoke("update_drive_config", {
      deviceName,
      driveConfig,
    })
  }

  async updateGlobalConfig(
    active?: boolean,
    pollIntervalMs?: number,
    theme?: string
  ): Promise<void> {
    return await invoke("update_global_config", {
      active,
      pollIntervalMs,
      theme,
    })
  }
}

let serviceInstance: TauriService | null = null

export function createTauriService(): TauriService {
  if (!serviceInstance) {
    serviceInstance = new DefaultTauriService()
  }
  return serviceInstance
}

export function setTauriService(service: TauriService): void {
  serviceInstance = service
}

export function resetTauriService(): void {
  serviceInstance = null
}
