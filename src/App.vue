<template>
  <div class="main-container">
    <DriveList
      :drives="drives"
      :selected-index="selectedDriveIndex"
      @select="handleDriveSelect"
    />
    <div class="content-area">
      <div class="settings-pane">
        <EmptyState v-if="selectedDriveIndex === undefined" />
        <DriveSettings
          v-else
          :drive="drives[selectedDriveIndex]"
          @update="handleDriveUpdate"
        />
      </div>
      <GlobalSettings :config="config" @update="handleGlobalUpdate" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import type { Drive, Config } from "./lib/tauri"
import DriveList from "./components/DriveList.vue"
import DriveSettings from "./components/DriveSettings.vue"
import GlobalSettings from "./components/GlobalSettings.vue"
import EmptyState from "./components/EmptyState.vue"
import { useDrives } from "./composables/useDrives"
import { useConfig } from "./composables/useConfig"

const { drives, loadDrives, updateDrive } = useDrives()
const { config, loadConfig, updateConfig } = useConfig()
const selectedDriveIndex = ref<number | undefined>(undefined)

async function loadData(): Promise<void> {
  try {
    await Promise.all([loadDrives(), loadConfig()])
  } catch (error) {
    console.error("Failed to load data:", error)
  }
}

function handleDriveSelect(index: number | undefined): void {
  selectedDriveIndex.value = index
}

function handleDriveUpdate(updatedDrive: Drive): void {
  updateDrive(updatedDrive)
}

function handleGlobalUpdate(updatedConfig: Partial<Config>): void {
  updateConfig(updatedConfig)
}

onMounted(() => {
  loadData()
})
</script>
