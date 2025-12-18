import { describe, it, expect, vi } from "vitest"
import { mount } from "@vue/test-utils"
import DriveList from "../DriveList.vue"
import type { Drive } from "../../lib/tauri"

describe("DriveList", () => {
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
    {
      name: "sdb",
      model: "Test Drive 2",
      size_display: "500.0 GB",
      config: {
        enabled: true,
        read: true,
        write: false,
        volume: 75,
      },
    },
  ]

  it("renders list of drives with correct display names", () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: mockDrives,
        selectedIndex: undefined,
      },
    })

    expect(wrapper.text()).toContain("1.0 TB Disk")
    expect(wrapper.text()).toContain("Test Drive 1")
    expect(wrapper.text()).toContain("500.0 GB Disk")
    expect(wrapper.text()).toContain("Test Drive 2")
  })

  it("highlights selected drive", () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: mockDrives,
        selectedIndex: 1,
      },
    })

    const driveItems = wrapper.findAll(".drive-item")
    expect(driveItems[0].classes()).not.toContain("selected")
    expect(driveItems[1].classes()).toContain("selected")
  })

  it("emits select event when drive clicked", async () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: mockDrives,
        selectedIndex: undefined,
      },
    })

    const driveItems = wrapper.findAll(".drive-item")
    await driveItems[0].trigger("click")

    expect(wrapper.emitted("select")).toBeTruthy()
    expect(wrapper.emitted("select")?.[0]).toEqual([0])
  })

  it("shows empty state when no drives", () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: [],
        selectedIndex: undefined,
      },
    })

    expect(wrapper.text()).toContain("No hard drives found")
    expect(wrapper.find(".drive-item").exists()).toBe(true)
  })

  it("handles click outside to deselect", async () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: mockDrives,
        selectedIndex: 0,
      },
    })

    const sidebar = wrapper.find(".sidebar")
    await sidebar.trigger("click")

    expect(wrapper.emitted("select")).toBeTruthy()
    expect(wrapper.emitted("select")?.[0]).toEqual([undefined])
  })

  it("prevents deselection when clicking on drive list", async () => {
    const wrapper = mount(DriveList, {
      props: {
        drives: mockDrives,
        selectedIndex: 0,
      },
    })

    const driveList = wrapper.find(".drive-list")
    await driveList.trigger("click.stop")

    expect(wrapper.emitted("select")).toBeFalsy()
  })
})
