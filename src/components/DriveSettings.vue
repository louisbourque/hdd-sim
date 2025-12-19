<template>
  <h2 class="text-2xl font-semibold mb-8 text-gray-900 dark:text-gray-100">
    {{ drive.model }} Settings
  </h2>
  <div
    class="max-w-[600px] rounded-lg bg-gray-50 dark:bg-[#3a3a3a] transition-all duration-200 ease-out shadow-sm dark:shadow-[0_1px_2px_rgba(0,0,0,0.2)]"
  >
    <div
      class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
    >
      <span
        class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
        >Enabled</span
      >
      <div class="flex items-center gap-3">
        <div
          :class="['toggle-switch', { active: drive.config.enabled }]"
          @click="toggleEnabled"
        ></div>
      </div>
    </div>

    <div
      class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
    >
      <span
        class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
        >Read</span
      >
      <div class="flex items-center gap-3">
        <div
          :class="[
            'toggle-switch',
            { active: drive.config.read },
            { disabled: !drive.config.enabled },
          ]"
          @click="drive.config.enabled && toggleRead()"
        ></div>
      </div>
    </div>

    <div
      class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
    >
      <span
        class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
        >Write</span
      >
      <div class="flex items-center gap-3">
        <div
          :class="[
            'toggle-switch',
            { active: drive.config.write },
            { disabled: !drive.config.enabled },
          ]"
          @click="drive.config.enabled && toggleWrite()"
        ></div>
      </div>
    </div>

    <div
      class="flex items-center justify-between p-4 mb-2 max-w-[560px] last:mb-0"
    >
      <span
        class="text-sm font-medium text-gray-900 dark:text-gray-100 min-w-[80px]"
        >Volume</span
      >
      <div class="flex items-center gap-3 flex-1 max-w-[300px]">
        <input
          type="range"
          min="0"
          max="100"
          :value="drive.config.volume"
          :disabled="!drive.config.enabled"
          class="slider"
          @input="updateVolume"
        />
        <span
          class="min-w-[50px] text-right text-sm font-medium text-gray-600 dark:text-gray-400"
          >{{ drive.config.volume }}</span
        >
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
