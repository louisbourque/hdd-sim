import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import GlobalSettings from "../GlobalSettings.vue";
import type { Config } from "../../lib/tauri";
import * as tauri from "../../lib/tauri";

vi.mock("../../lib/tauri", () => ({
  updateGlobalConfig: vi.fn(),
}));

describe("GlobalSettings", () => {
  const mockConfig: Config = {
    active: true,
    poll_interval_ms: 100,
    drives: {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders play/pause button with correct state", () => {
    const wrapper = mount(GlobalSettings, {
      props: {
        config: mockConfig,
      },
    });

    const button = wrapper.find(".play-pause-button");
    expect(button.exists()).toBe(true);
    expect(button.text()).toBe("⏸");
    expect(button.classes()).not.toContain("paused");
  });

  it("shows play button when paused", () => {
    const pausedConfig: Config = {
      ...mockConfig,
      active: false,
    };

    const wrapper = mount(GlobalSettings, {
      props: {
        config: pausedConfig,
      },
    });

    const button = wrapper.find(".play-pause-button");
    expect(button.text()).toBe("▶");
    expect(button.classes()).toContain("paused");
  });

  it("toggles active state and emits update", async () => {
    const updateGlobalConfigSpy = vi.spyOn(tauri, "updateGlobalConfig");
    updateGlobalConfigSpy.mockResolvedValue(undefined);

    const wrapper = mount(GlobalSettings, {
      props: {
        config: mockConfig,
      },
    });

    const button = wrapper.find(".play-pause-button");
    await button.trigger("click");

    expect(updateGlobalConfigSpy).toHaveBeenCalledWith(false, undefined);
    expect(wrapper.emitted("update")).toBeTruthy();
    expect(wrapper.emitted("update")?.[0]).toEqual([{ active: false }]);
  });

  it("updates interval slider and emits update", async () => {
    const updateGlobalConfigSpy = vi.spyOn(tauri, "updateGlobalConfig");
    updateGlobalConfigSpy.mockResolvedValue(undefined);

    const wrapper = mount(GlobalSettings, {
      props: {
        config: mockConfig,
      },
    });

    const slider = wrapper.find('input[type="range"]');
    await slider.setValue(200);
    await slider.trigger("input");

    expect(updateGlobalConfigSpy).toHaveBeenCalledWith(undefined, 200);
    expect(wrapper.emitted("update")).toBeTruthy();
    expect(wrapper.emitted("update")?.[0]).toEqual([{ poll_interval_ms: 200 }]);
  });

  it("displays correct interval value", () => {
    const configWithInterval: Config = {
      ...mockConfig,
      poll_interval_ms: 250,
    };

    const wrapper = mount(GlobalSettings, {
      props: {
        config: configWithInterval,
      },
    });

    expect(wrapper.text()).toContain("250ms");
    const slider = wrapper.find('input[type="range"]');
    expect(slider.element.value).toBe("250");
  });

  it("handles update errors gracefully", async () => {
    const updateGlobalConfigSpy = vi.spyOn(tauri, "updateGlobalConfig");
    updateGlobalConfigSpy.mockRejectedValue(new Error("Update failed"));
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const wrapper = mount(GlobalSettings, {
      props: {
        config: mockConfig,
      },
    });

    const button = wrapper.find(".play-pause-button");
    await button.trigger("click");

    expect(updateGlobalConfigSpy).toHaveBeenCalled();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "Failed to update active state:",
      expect.any(Error)
    );

    consoleErrorSpy.mockRestore();
  });
});

