<template>
  <div
    class="p-5 px-8 bg-white dark:bg-[#2d2d2d] rounded-xl flex items-start gap-6 shrink-0 shadow-md dark:shadow-[0_2px_8px_rgba(0,0,0,0.3),0_1px_2px_rgba(0,0,0,0.2)]"
  >
    <button
      :class="[
              'w-12 h-12 rounded-full border-none text-white cursor-pointer flex items-center justify-center transition-all duration-300 ease-out shadow-[0_4px_12px_rgba(33,150,243,0.3),0_2px_4px_rgba(0,0,0,0.1)] hover:-translate-y-0.5 hover:shadow-[0_6px_16px_rgba(33,150,243,0.4),0_3px_6px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_8px_rgba(33,150,243,0.3),0_1px_2px_rgba(0,0,0,0.1)]',
        !config.active
          ? 'bg-linear-to-br from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 shadow-[0_4px_12px_rgba(76,175,80,0.3),0_2px_4px_rgba(0,0,0,0.1)] hover:shadow-[0_6px_16px_rgba(76,175,80,0.4),0_3px_6px_rgba(0,0,0,0.15)] text-2xl'
          : 'bg-linear-to-br from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-4xl'
      ]""
      @click="toggleActive"
      :title="config.active ? 'Pause' : 'Play'"
    >
      {{ config.active ? "⏸" : "▶" }}
    </button>
    <div class="flex flex-col gap-4 flex-1">
      <div class="flex items-center justify-between flex-1">
        <span
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Interval</span
        >
        <div class="flex items-center gap-3 flex-1 max-w-[300px]">
          <input
            type="range"
            min="50"
            max="1000"
            step="10"
            :value="config.poll_interval_ms"
            class="slider"
            @input="updateInterval"
          />
          <span
            class="min-w-[50px] text-right text-sm font-medium text-gray-600 dark:text-gray-400"
            >{{ config.poll_interval_ms }}ms</span
          >
        </div>
      </div>

      <div class="flex items-center justify-between flex-0">
        <span
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Theme</span
        >
        <div class="flex items-center gap-3">
          <select
            :value="config.theme || 'system'"
            class="px-3 py-2 rounded-md border border-gray-300 dark:border-[#4a4a4a] text-gray-900 text-sm cursor-pointer transition-all duration-200 ease-out outline-none hover:border-blue-500 dark:hover:border-blue-300 focus:border-blue-500 dark:focus:border-blue-300 focus:shadow-[0_0_0_3px_rgba(33,150,243,0.1)] dark:focus:shadow-[0_0_0_3px_rgba(100,181,246,0.2)]"
            @change="updateTheme"
          >
            <option value="system">System Default</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Config } from "../lib/tauri"
import { updateGlobalConfig } from "../lib/tauri"

const props = defineProps<{
  config: Config
}>()

const emit = defineEmits<{
  update: [config: Partial<Config>]
}>()

async function toggleActive(): Promise<void> {
  try {
    const newActive = !props.config.active
    await updateGlobalConfig(newActive, undefined, undefined)
    emit("update", { active: newActive })
  } catch (error) {
    console.error("Failed to update active state:", error)
  }
}

async function updateInterval(event: Event): Promise<void> {
  try {
    const target = event.target as HTMLInputElement
    const newInterval = parseInt(target.value)
    await updateGlobalConfig(undefined, newInterval, undefined)
    emit("update", { poll_interval_ms: newInterval })
  } catch (error) {
    console.error("Failed to update interval:", error)
  }
}

async function updateTheme(event: Event): Promise<void> {
  try {
    const target = event.target as HTMLSelectElement
    const newTheme = target.value
    await updateGlobalConfig(undefined, undefined, newTheme)
    emit("update", { theme: newTheme })
  } catch (error) {
    console.error("Failed to update theme:", error)
  }
}
</script>
