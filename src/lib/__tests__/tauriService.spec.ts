import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  createTauriService,
  setTauriService,
  resetTauriService,
  type TauriService,
} from "../tauriService";
import type { Drive, DriveConfig, Config } from "../tauri";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("tauriService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetTauriService();
  });

  describe("DefaultTauriService", () => {
    it("getDrives returns drive array", async () => {
      const mockDrives: Drive[] = [
        {
          name: "sda",
          model: "Test Drive",
          size_display: "1.0 TB",
          config: {
            enabled: false,
            read: false,
            write: false,
            volume: 50,
          },
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockDrives);

      const service = createTauriService();
      const result = await service.getDrives();

      expect(invoke).toHaveBeenCalledWith("get_drives");
      expect(result).toEqual(mockDrives);
    });

    it("loadConfig returns config object", async () => {
      const mockConfig: Config = {
        active: true,
        poll_interval_ms: 100,
        drives: {},
      };

      vi.mocked(invoke).mockResolvedValue(mockConfig);

      const service = createTauriService();
      const result = await service.loadConfig();

      expect(invoke).toHaveBeenCalledWith("load_config_command");
      expect(result).toEqual(mockConfig);
    });

    it("saveConfig calls invoke with correct params", async () => {
      const mockConfig: Config = {
        active: true,
        poll_interval_ms: 100,
        drives: {},
      };

      vi.mocked(invoke).mockResolvedValue(undefined);

      const service = createTauriService();
      await service.saveConfig(mockConfig);

      expect(invoke).toHaveBeenCalledWith("save_config_command", {
        config: mockConfig,
      });
    });

    it("updateDriveConfig calls invoke with correct params", async () => {
      const driveConfig: DriveConfig = {
        enabled: true,
        read: true,
        write: false,
        volume: 75,
      };

      vi.mocked(invoke).mockResolvedValue(undefined);

      const service = createTauriService();
      await service.updateDriveConfig("sda", driveConfig);

      expect(invoke).toHaveBeenCalledWith("update_drive_config", {
        deviceName: "sda",
        driveConfig,
      });
    });

    it("updateGlobalConfig calls invoke with active param", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const service = createTauriService();
      await service.updateGlobalConfig(true, undefined);

      expect(invoke).toHaveBeenCalledWith("update_global_config", {
        active: true,
        pollIntervalMs: undefined,
      });
    });

    it("updateGlobalConfig calls invoke with pollIntervalMs param", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const service = createTauriService();
      await service.updateGlobalConfig(undefined, 200);

      expect(invoke).toHaveBeenCalledWith("update_global_config", {
        active: undefined,
        pollIntervalMs: 200,
      });
    });

    it("updateGlobalConfig calls invoke with both params", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const service = createTauriService();
      await service.updateGlobalConfig(false, 300);

      expect(invoke).toHaveBeenCalledWith("update_global_config", {
        active: false,
        pollIntervalMs: 300,
      });
    });

    it("handles errors appropriately", async () => {
      const error = new Error("Tauri invoke failed");
      vi.mocked(invoke).mockRejectedValue(error);

      const service = createTauriService();

      await expect(service.getDrives()).rejects.toThrow("Tauri invoke failed");
    });
  });

  describe("Service management", () => {
    it("createTauriService returns singleton instance", () => {
      const service1 = createTauriService();
      const service2 = createTauriService();

      expect(service1).toBe(service2);
    });

    it("setTauriService allows custom service injection", () => {
      const mockService: TauriService = {
        getDrives: vi.fn(),
        loadConfig: vi.fn(),
        saveConfig: vi.fn(),
        updateDriveConfig: vi.fn(),
        updateGlobalConfig: vi.fn(),
      };

      setTauriService(mockService);
      const service = createTauriService();

      expect(service).toBe(mockService);
    });

    it("resetTauriService clears service instance", () => {
      const service1 = createTauriService();
      resetTauriService();
      const service2 = createTauriService();

      expect(service1).not.toBe(service2);
    });
  });
});

