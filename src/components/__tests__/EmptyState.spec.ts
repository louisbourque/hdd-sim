import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import EmptyState from "../EmptyState.vue";

describe("EmptyState", () => {
  it("renders empty state message", () => {
    const wrapper = mount(EmptyState);

    expect(wrapper.text()).toContain("No Device Selected");
    expect(wrapper.text()).toContain("Select a device to manage.");
  });

  it("renders empty state icon", () => {
    const wrapper = mount(EmptyState);

    expect(wrapper.find(".empty-state-icon").exists()).toBe(true);
    expect(wrapper.find(".empty-state-icon").text()).toBe("💾");
  });

  it("has correct CSS classes", () => {
    const wrapper = mount(EmptyState);

    expect(wrapper.find(".empty-state").exists()).toBe(true);
    expect(wrapper.find(".empty-state-title").exists()).toBe(true);
    expect(wrapper.find(".empty-state-subtitle").exists()).toBe(true);
  });
});

