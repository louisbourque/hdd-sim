import { describe, it, expect, vi, beforeEach } from "vitest"
import { mount } from "@vue/test-utils"
import App from "../../App.vue"
import DriveList from "../DriveList.vue"
import DriveSettings from "../DriveSettings.vue"
import GlobalSettings from "../GlobalSettings.vue"
import EmptyState from "../EmptyState.vue"
import type { Drive, Config } from "../../lib/tauri"
import { useDrives } from "../../composables/useDrives"
import { useConfig } from "../../composables/useConfig"
import { ref } from "vue"

vi.mock("../../composables/useDrives")
vi.mock("../../composables/useConfig")

describe("App", () => {
  const mockDrives: Drive[] = [
    {
      name: "sda",
      model: "Test Drive 1",
      size_display: "1.0 TB",
      config: {
        enabled: false,
        read: false,
        write: false,
        volume: 50,
      },
    },
  ]

  const mockConfig: Config = {
    active: true,
    poll_interval_ms: 100,
    drives: {},
    theme: "system",
  }

  beforeEach(() => {
    vi.clearAllMocks()
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: vi.fn().mockImplementation((query) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    })
  })

  it("loads drives and config on mount", async () => {
    const mockLoadDrives = vi.fn().mockResolvedValue(undefined)
    const mockLoadConfig = vi.fn().mockResolvedValue(undefined)

    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: mockLoadDrives,
      updateDrive: vi.fn(),
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: mockLoadConfig,
      updateConfig: vi.fn(),
    })

    mount(App)

    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(mockLoadDrives).toHaveBeenCalled()
    expect(mockLoadConfig).toHaveBeenCalled()
  })

  it("handles drive selection", async () => {
    const mockUpdateDrive = vi.fn()

    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: vi.fn(),
      updateDrive: mockUpdateDrive,
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: vi.fn(),
      updateConfig: vi.fn(),
    })

    const wrapper = mount(App)

    const driveList = wrapper.findComponent(DriveList)
    await driveList.vm.$emit("select", 0)
    await wrapper.vm.$nextTick()

    expect(wrapper.findComponent(EmptyState).exists()).toBe(false)
    expect(wrapper.findComponent(DriveSettings).exists()).toBe(true)
  })

  it("updates drive when DriveSettings emits", async () => {
    const mockUpdateDrive = vi.fn()
    const updatedDrive: Drive = {
      ...mockDrives[0],
      config: {
        ...mockDrives[0].config,
        enabled: true,
      },
    }

    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: vi.fn(),
      updateDrive: mockUpdateDrive,
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: vi.fn(),
      updateConfig: vi.fn(),
    })

    const wrapper = mount(App)

    const driveList = wrapper.findComponent(DriveList)
    await driveList.vm.$emit("select", 0)
    await wrapper.vm.$nextTick()

    const driveSettings = wrapper.findComponent(DriveSettings)
    await driveSettings.vm.$emit("update", updatedDrive)

    expect(mockUpdateDrive).toHaveBeenCalledWith(updatedDrive)
  })

  it("updates config when GlobalSettings emits", async () => {
    const mockUpdateConfig = vi.fn()

    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: vi.fn(),
      updateDrive: vi.fn(),
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: vi.fn(),
      updateConfig: mockUpdateConfig,
    })

    const wrapper = mount(App)

    const globalSettings = wrapper.findComponent(GlobalSettings)
    await globalSettings.vm.$emit("update", { active: false })

    expect(mockUpdateConfig).toHaveBeenCalledWith({ active: false })
  })

  it("shows EmptyState when no drive selected", () => {
    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: vi.fn(),
      updateDrive: vi.fn(),
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: vi.fn(),
      updateConfig: vi.fn(),
    })

    const wrapper = mount(App)

    expect(wrapper.findComponent(EmptyState).exists()).toBe(true)
    expect(wrapper.findComponent(DriveSettings).exists()).toBe(false)
  })

  it("shows DriveSettings when drive selected", async () => {
    vi.mocked(useDrives).mockReturnValue({
      drives: ref(mockDrives),
      loading: ref(false),
      error: ref(null),
      loadDrives: vi.fn(),
      updateDrive: vi.fn(),
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: vi.fn(),
      updateConfig: vi.fn(),
    })

    const wrapper = mount(App)

    const driveList = wrapper.findComponent(DriveList)
    await driveList.vm.$emit("select", 0)
    await wrapper.vm.$nextTick()

    expect(wrapper.findComponent(EmptyState).exists()).toBe(false)
    expect(wrapper.findComponent(DriveSettings).exists()).toBe(true)
  })

  it("handles loading errors gracefully", async () => {
    const mockLoadDrives = vi.fn().mockRejectedValue(new Error("Load failed"))
    const mockLoadConfig = vi.fn().mockResolvedValue(undefined)
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {})

    vi.mocked(useDrives).mockReturnValue({
      drives: ref([]),
      loading: ref(false),
      error: ref(new Error("Load failed")),
      loadDrives: mockLoadDrives,
      updateDrive: vi.fn(),
    })

    vi.mocked(useConfig).mockReturnValue({
      config: ref(mockConfig),
      loading: ref(false),
      error: ref(null),
      loadConfig: mockLoadConfig,
      updateConfig: vi.fn(),
    })

    mount(App)

    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(mockLoadDrives).toHaveBeenCalled()
    expect(mockLoadConfig).toHaveBeenCalled()

    consoleErrorSpy.mockRestore()
  })
})
