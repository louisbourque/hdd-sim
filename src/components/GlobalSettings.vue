<template>
  <div class="global-settings">
    <button
      :class="['play-pause-button', { paused: !config.active }]"
      @click="toggleActive"
      :title="config.active ? 'Pause' : 'Play'"
    >
      {{ config.active ? "⏸" : "▶" }}
    </button>

    <div class="settings-column">
      <div class="setting-row" style="flex: 1; border: none; padding: 0">
        <span class="setting-label">Interval</span>
        <div class="setting-control slider-container">
          <input
            type="range"
            min="50"
            max="1000"
            step="10"
            :value="config.poll_interval_ms"
            class="slider"
            @input="updateInterval"
          />
          <span class="slider-value">{{ config.poll_interval_ms }}ms</span>
        </div>
      </div>

      <div class="setting-row" style="flex: 0; border: none; padding: 0">
        <span class="setting-label">Theme</span>
        <div class="setting-control">
          <select
            :value="config.theme || 'system'"
            class="theme-select"
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
