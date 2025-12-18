import { describe, it, expect, vi, beforeEach } from "vitest"
import { mount } from "@vue/test-utils"
import DriveSettings from "../DriveSettings.vue"
import type { Drive } from "../../lib/tauri"
import * as tauri from "../../lib/tauri"

vi.mock("../../lib/tauri", () => ({
  updateDriveConfig: vi.fn(),
}))

describe("DriveSettings", () => {
  const mockDrive: Drive = {
    name: "sda",
    model: "Test Drive",
    size_display: "1.0 TB",
    config: {
      enabled: true,
      read: true,
      write: false,
      volume: 50,
    },
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it("renders drive model in title", () => {
    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    expect(wrapper.find(".settings-title").text()).toBe("Test Drive Settings")
  })

  it("toggles enabled state and emits update", async () => {
    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    const enabledToggle = wrapper.findAll(".toggle-switch")[0]
    await enabledToggle.trigger("click")

    expect(updateDriveConfigSpy).toHaveBeenCalledWith("sda", {
      ...mockDrive.config,
      enabled: false,
    })
    expect(wrapper.emitted("update")).toBeTruthy()
    expect(wrapper.emitted("update")?.[0]?.[0]).toMatchObject({
      name: "sda",
      config: {
        ...mockDrive.config,
        enabled: false,
      },
    })
  })

  it("toggles read when enabled", async () => {
    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    const readToggle = wrapper.findAll(".toggle-switch")[1]
    await readToggle.trigger("click")

    expect(updateDriveConfigSpy).toHaveBeenCalledWith("sda", {
      ...mockDrive.config,
      read: false,
    })
  })

  it("prevents read toggle when disabled", async () => {
    const disabledDrive: Drive = {
      ...mockDrive,
      config: {
        ...mockDrive.config,
        enabled: false,
      },
    }

    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: disabledDrive,
      },
    })

    const readToggle = wrapper.findAll(".toggle-switch")[1]
    await readToggle.trigger("click")

    expect(updateDriveConfigSpy).not.toHaveBeenCalled()
  })

  it("toggles write when enabled", async () => {
    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    const writeToggle = wrapper.findAll(".toggle-switch")[2]
    await writeToggle.trigger("click")

    expect(updateDriveConfigSpy).toHaveBeenCalledWith("sda", {
      ...mockDrive.config,
      write: true,
    })
  })

  it("prevents write toggle when disabled", async () => {
    const disabledDrive: Drive = {
      ...mockDrive,
      config: {
        ...mockDrive.config,
        enabled: false,
      },
    }

    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: disabledDrive,
      },
    })

    const writeToggle = wrapper.findAll(".toggle-switch")[2]
    await writeToggle.trigger("click")

    expect(updateDriveConfigSpy).not.toHaveBeenCalled()
  })

  it("updates volume slider and emits update", async () => {
    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockResolvedValue(undefined)

    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    const slider = wrapper.find('input[type="range"]')
    await slider.setValue(75)
    await slider.trigger("input")

    expect(updateDriveConfigSpy).toHaveBeenCalledWith("sda", {
      ...mockDrive.config,
      volume: 75,
    })
    expect(wrapper.emitted("update")).toBeTruthy()
  })

  it("disables controls when drive disabled", () => {
    const disabledDrive: Drive = {
      ...mockDrive,
      config: {
        ...mockDrive.config,
        enabled: false,
      },
    }

    const wrapper = mount(DriveSettings, {
      props: {
        drive: disabledDrive,
      },
    })

    const slider = wrapper.find('input[type="range"]')
    expect(slider.attributes("disabled")).toBeDefined()

    const readToggle = wrapper.findAll(".toggle-switch")[1]
    expect(readToggle.classes()).toContain("disabled")

    const writeToggle = wrapper.findAll(".toggle-switch")[2]
    expect(writeToggle.classes()).toContain("disabled")
  })

  it("displays correct volume value", () => {
    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    expect(wrapper.text()).toContain("50")
    const slider = wrapper.find('input[type="range"]')
    expect((slider.element as HTMLInputElement).value).toBe("50")
  })

  it("handles update errors gracefully", async () => {
    const updateDriveConfigSpy = vi.spyOn(tauri, "updateDriveConfig")
    updateDriveConfigSpy.mockRejectedValue(new Error("Update failed"))
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {})

    const wrapper = mount(DriveSettings, {
      props: {
        drive: mockDrive,
      },
    })

    const enabledToggle = wrapper.findAll(".toggle-switch")[0]
    await enabledToggle.trigger("click")

    expect(updateDriveConfigSpy).toHaveBeenCalled()
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "Failed to update drive config:",
      expect.any(Error)
    )

    consoleErrorSpy.mockRestore()
  })
})
