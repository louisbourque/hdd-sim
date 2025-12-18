<template>
  <h2 class="settings-title">{{ drive.model }} Settings</h2>
  <div class="settings-section">
    <div class="setting-row">
      <span class="setting-label">Enabled</span>
      <div class="setting-control">
        <div
          :class="['toggle-switch', { active: drive.config.enabled }]"
          @click="toggleEnabled"
        ></div>
      </div>
    </div>

    <div class="setting-row">
      <span class="setting-label">Read</span>
      <div class="setting-control">
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

    <div class="setting-row">
      <span class="setting-label">Write</span>
      <div class="setting-control">
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

    <div class="setting-row">
      <span class="setting-label">Volume</span>
      <div class="setting-control slider-container">
        <input
          type="range"
          min="0"
          max="100"
          :value="drive.config.volume"
          :disabled="!drive.config.enabled"
          class="slider"
          @input="updateVolume"
        />
        <span class="slider-value">{{ drive.config.volume }}</span>
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
