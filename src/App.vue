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
import { getDrives, loadConfig, type Drive, type Config } from "./lib/tauri"
import DriveList from "./components/DriveList.vue"
import DriveSettings from "./components/DriveSettings.vue"
import GlobalSettings from "./components/GlobalSettings.vue"
import EmptyState from "./components/EmptyState.vue"

const drives = ref<Drive[]>([])
const selectedDriveIndex = ref<number | undefined>(undefined)
const config = ref<Config>({ active: true, poll_interval_ms: 100, drives: {} })

async function loadData(): Promise<void> {
  try {
    const [drivesData, configData] = await Promise.all([
      getDrives(),
      loadConfig(),
    ])
    drives.value = drivesData
    config.value = configData
  } catch (error) {
    console.error("Failed to load data:", error)
  }
}

function handleDriveSelect(index: number): void {
  selectedDriveIndex.value =
    index === selectedDriveIndex.value ? undefined : index
}

function handleDriveUpdate(updatedDrive: Drive): void {
  const index = drives.value.findIndex((d) => d.name === updatedDrive.name)
  if (index !== -1) {
    drives.value[index] = updatedDrive
  }
}

function handleGlobalUpdate(updatedConfig: Partial<Config>): void {
  config.value = { ...config.value, ...updatedConfig }
}

onMounted(() => {
  loadData()
})
</script>
