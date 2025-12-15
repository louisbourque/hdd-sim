import { describe, it, expect, vi, beforeEach } from "vitest";
import { useDrives } from "../useDrives";
import type { TauriService } from "../../lib/tauriService";
import type { Drive } from "../../lib/tauri";

describe("useDrives", () => {
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

  it("initializes with empty drives", () => {
    const { drives, loading, error } = useDrives(mockService);

    expect(drives.value).toEqual([]);
    expect(loading.value).toBe(false);
    expect(error.value).toBeNull();
  });

  it("loads drives from service", async () => {
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

    vi.mocked(mockService.getDrives).mockResolvedValue(mockDrives);

    const { drives, loadDrives, loading } = useDrives(mockService);

    await loadDrives();

    expect(mockService.getDrives).toHaveBeenCalled();
    expect(drives.value).toEqual(mockDrives);
    expect(loading.value).toBe(false);
  });

  it("sets loading state during load", async () => {
    let resolvePromise: (value: Drive[]) => void;
    const promise = new Promise<Drive[]>((resolve) => {
      resolvePromise = resolve;
    });

    vi.mocked(mockService.getDrives).mockReturnValue(promise);

    const { loading, loadDrives } = useDrives(mockService);

    const loadPromise = loadDrives();

    expect(loading.value).toBe(true);

    resolvePromise!([]);
    await loadPromise;

    expect(loading.value).toBe(false);
  });

  it("updates drive in array", () => {
    const initialDrives: Drive[] = [
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

    const { drives, updateDrive } = useDrives(mockService);
    drives.value = [...initialDrives];

    const updatedDrive: Drive = {
      ...initialDrives[0],
      config: {
        ...initialDrives[0].config,
        enabled: true,
      },
    };

    updateDrive(updatedDrive);

    expect(drives.value[0]).toEqual(updatedDrive);
  });

  it("does not update if drive not found", () => {
    const initialDrives: Drive[] = [
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

    const { drives, updateDrive } = useDrives(mockService);
    drives.value = [...initialDrives];

    const updatedDrive: Drive = {
      name: "sdb",
      model: "Other Drive",
      size_display: "500.0 GB",
      config: {
        enabled: true,
        read: false,
        write: false,
        volume: 50,
      },
    };

    updateDrive(updatedDrive);

    expect(drives.value).toEqual(initialDrives);
  });

  it("handles service errors", async () => {
    const serviceError = new Error("Service error");
    vi.mocked(mockService.getDrives).mockRejectedValue(serviceError);
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { error, loadDrives, drives } = useDrives(mockService);

    await loadDrives();

    expect(error.value).toBeInstanceOf(Error);
    expect(error.value?.message).toBe("Service error");
    expect(drives.value).toEqual([]);
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "Failed to load drives:",
      serviceError
    );

    consoleErrorSpy.mockRestore();
  });

  it("handles non-Error exceptions", async () => {
    vi.mocked(mockService.getDrives).mockRejectedValue("String error");
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { error, loadDrives } = useDrives(mockService);

    await loadDrives();

    expect(error.value).toBeInstanceOf(Error);
    expect(error.value?.message).toBe("String error");

    consoleErrorSpy.mockRestore();
  });
});

