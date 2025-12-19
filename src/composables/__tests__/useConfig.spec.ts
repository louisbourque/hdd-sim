import { describe, it, expect, vi, beforeEach } from "vitest";
import { useConfig } from "../useConfig";
import type { TauriService } from "../../lib/tauriService";
import type { Config } from "../../lib/tauri";

describe("useConfig", () => {
  let mockService: TauriService;

  beforeEach(() => {
    mockService = {
      getDrives: vi.fn(),
      loadConfig: vi.fn(),
      saveConfig: vi.fn(),
      updateDriveConfig: vi.fn(),
      updateGlobalConfig: vi.fn(),
    };
  });

  it("initializes with default config", () => {
    const { config, loading, error } = useConfig(mockService);

    expect(config.value).toEqual({
      active: true,
      poll_interval_ms: 100,
      drives: {},
      theme: "system",
    });
    expect(loading.value).toBe(false);
    expect(error.value).toBeNull();
  });

  it("loads config from service", async () => {
    const mockConfig: Config = {
      active: false,
      poll_interval_ms: 200,
      drives: {
        sda: {
          enabled: true,
          read: true,
          write: false,
          volume: 75,
        },
      },
      theme: "system",
    };

    vi.mocked(mockService.loadConfig).mockResolvedValue(mockConfig);

    const { config, loadConfig, loading } = useConfig(mockService);

    await loadConfig();

    expect(mockService.loadConfig).toHaveBeenCalled();
    expect(config.value).toEqual(mockConfig);
    expect(loading.value).toBe(false);
  });

  it("sets loading state during load", async () => {
    let resolvePromise: (value: Config) => void;
    const promise = new Promise<Config>((resolve) => {
      resolvePromise = resolve;
    });

    vi.mocked(mockService.loadConfig).mockReturnValue(promise);

    const { loading, loadConfig } = useConfig(mockService);

    const loadPromise = loadConfig();

    expect(loading.value).toBe(true);

    resolvePromise!({
      active: true,
      poll_interval_ms: 100,
      drives: {},
      theme: "system",
    });
    await loadPromise;

    expect(loading.value).toBe(false);
  });

  it("updates config values", () => {
    const { config, updateConfig } = useConfig(mockService);

    updateConfig({ active: false });

    expect(config.value.active).toBe(false);
    expect(config.value.poll_interval_ms).toBe(100);
  });

  it("merges partial config updates", () => {
    const initialConfig: Config = {
      active: true,
      poll_interval_ms: 100,
      drives: {
        sda: {
          enabled: false,
          read: false,
          write: false,
          volume: 50,
        },
      },
      theme: "system",
    };

    const { config, updateConfig } = useConfig(mockService);
    config.value = { ...initialConfig };

    updateConfig({ poll_interval_ms: 250 });

    expect(config.value.active).toBe(true);
    expect(config.value.poll_interval_ms).toBe(250);
    expect(config.value.drives).toEqual(initialConfig.drives);
  });

  it("handles service errors and falls back to default", async () => {
    const serviceError = new Error("Service error");
    vi.mocked(mockService.loadConfig).mockRejectedValue(serviceError);
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { error, loadConfig, config } = useConfig(mockService);

    await loadConfig();

    expect(error.value).toBeInstanceOf(Error);
    expect(error.value?.message).toBe("Service error");
    expect(config.value).toEqual({
      active: true,
      poll_interval_ms: 100,
      drives: {},
      theme: "system",
    });
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "Failed to load config:",
      serviceError
    );

    consoleErrorSpy.mockRestore();
  });

  it("handles non-Error exceptions", async () => {
    vi.mocked(mockService.loadConfig).mockRejectedValue("String error");
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { error, loadConfig, config } = useConfig(mockService);

    await loadConfig();

    expect(error.value).toBeInstanceOf(Error);
    expect(error.value?.message).toBe("String error");
    expect(config.value).toEqual({
      active: true,
      poll_interval_ms: 100,
      drives: {},
      theme: "system",
    });

    consoleErrorSpy.mockRestore();
  });
});

