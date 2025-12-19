<template>
  <div>
    <h2
      id="drive-settings-heading"
      class="text-2xl font-semibold mb-8 text-gray-900 dark:text-gray-100"
    >
      {{ drive.model }} Settings
    </h2>
    <div
      class="max-w-[600px] rounded-lg bg-gray-50 dark:bg-[#3a3a3a] transition-all duration-200 ease-out shadow-sm dark:shadow-[0_1px_2px_rgba(0,0,0,0.2)]"
      role="group"
      aria-labelledby="drive-settings-heading"
    >
      <div
        class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
      >
        <label
          for="enabled-toggle"
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Enabled</label
        >
        <div class="flex items-center gap-3">
          <button
            id="enabled-toggle"
            type="button"
            role="switch"
            :aria-checked="drive.config.enabled"
            :aria-label="`Enable ${drive.model}`"
            :class="['toggle-switch', { active: drive.config.enabled }]"
            @click="toggleEnabled"
            @keydown.enter.prevent="toggleEnabled"
            @keydown.space.prevent="toggleEnabled"
          ></button>
        </div>
      </div>

      <div
        class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
      >
        <label
          for="read-toggle"
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Read</label
        >
        <div class="flex items-center gap-3">
          <button
            id="read-toggle"
            type="button"
            role="switch"
            :aria-checked="drive.config.read"
            :aria-label="`Enable read operations for ${drive.model}`"
            :aria-disabled="!drive.config.enabled"
            :class="[
              'toggle-switch',
              { active: drive.config.read },
              { disabled: !drive.config.enabled },
            ]"
            @click="drive.config.enabled && toggleRead()"
            @keydown.enter.prevent="drive.config.enabled && toggleRead()"
            @keydown.space.prevent="drive.config.enabled && toggleRead()"
            :tabindex="drive.config.enabled ? 0 : -1"
          ></button>
        </div>
      </div>

      <div
        class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
      >
        <label
          for="write-toggle"
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Write</label
        >
        <div class="flex items-center gap-3">
          <button
            id="write-toggle"
            type="button"
            role="switch"
            :aria-checked="drive.config.write"
            :aria-label="`Enable write operations for ${drive.model}`"
            :aria-disabled="!drive.config.enabled"
            :class="[
              'toggle-switch',
              { active: drive.config.write },
              { disabled: !drive.config.enabled },
            ]"
            @click="drive.config.enabled && toggleWrite()"
            @keydown.enter.prevent="drive.config.enabled && toggleWrite()"
            @keydown.space.prevent="drive.config.enabled && toggleWrite()"
            :tabindex="drive.config.enabled ? 0 : -1"
          ></button>
        </div>
      </div>

      <div
        class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
      >
        <label
          for="volume-slider"
          class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
          >Volume</label
        >
        <div class="flex items-center gap-3 flex-1 max-w-[300px]">
          <input
            id="volume-slider"
            type="range"
            min="0"
            max="100"
            :value="drive.config.volume"
            :disabled="!drive.config.enabled"
            class="slider"
            @input="updateVolume"
            :aria-valuemin="0"
            :aria-valuemax="100"
            :aria-valuenow="drive.config.volume"
            :aria-label="`Volume level for ${drive.model}`"
          />
          <span
            class="min-w-[50px] text-right text-sm font-medium text-gray-600 dark:text-gray-400"
            aria-live="polite"
            aria-atomic="true"
            >{{ drive.config.volume }}</span
          >
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Drive, DriveConfig } from "../lib/tauri"
import { updateDriveConfig } from "../lib/tauri"

const props = defineProps<{
  drive: Drive
}>()

const emit = defineEmits<{
  update: [drive: Drive]
}>()

async function saveDriveConfig(newConfig: DriveConfig): Promise<void> {
  try {
    await updateDriveConfig(props.drive.name, newConfig)
    emit("update", {
      ...props.drive,
      config: newConfig,
    })
  } catch (error) {
    console.error("Failed to update drive config:", error)
  }
}

function toggleEnabled(): void {
  const newConfig = {
    ...props.drive.config,
    enabled: !props.drive.config.enabled,
  }
  saveDriveConfig(newConfig)
}

function toggleRead(): void {
  if (props.drive.config.enabled) {
    const newConfig = {
      ...props.drive.config,
      read: !props.drive.config.read,
    }
    saveDriveConfig(newConfig)
  }
}

function toggleWrite(): void {
  if (props.drive.config.enabled) {
    const newConfig = {
      ...props.drive.config,
      write: !props.drive.config.write,
    }
    saveDriveConfig(newConfig)
  }
}

function updateVolume(event: Event): void {
  if (props.drive.config.enabled) {
    const target = event.target as HTMLInputElement
    const newConfig = {
      ...props.drive.config,
      volume: parseInt(target.value),
    }
    saveDriveConfig(newConfig)
  }
}
</script>
