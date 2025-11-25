<template>
  <div class="main-container">
    <DriveList
      :drives="drives"
      :selected-index="selectedDriveIndex"
      @select="handleDriveSelect"
    />
    <div class="content-area">
      <div class="settings-pane">
        <EmptyState v-if="selectedDriveIndex === null" />
        <DriveSettings
          v-else
          :drive="drives[selectedDriveIndex]"
          @update="handleDriveUpdate"
        />
      </div>
      <GlobalSettings
        :config="config"
        @update="handleGlobalUpdate"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { getDrives, loadConfig } from './lib/tauri'
import DriveList from './components/DriveList.vue'
import DriveSettings from './components/DriveSettings.vue'
import GlobalSettings from './components/GlobalSettings.vue'
import EmptyState from './components/EmptyState.vue'

const drives = ref([])
const selectedDriveIndex = ref(null)
const config = ref({ active: true, poll_interval_ms: 100, drives: {} })

async function loadData() {
  try {
    const [drivesData, configData] = await Promise.all([
      getDrives(),
      loadConfig(),
    ])
    drives.value = drivesData
    config.value = configData
  } catch (error) {
    console.error('Failed to load data:', error)
  }
}

function handleDriveSelect(index) {
  selectedDriveIndex.value = index === selectedDriveIndex.value ? null : index
}

function handleDriveUpdate(updatedDrive) {
  const index = drives.value.findIndex((d) => d.name === updatedDrive.name)
  if (index !== -1) {
    drives.value[index] = updatedDrive
  }
}

function handleGlobalUpdate(updatedConfig) {
  config.value = { ...config.value, ...updatedConfig }
}

onMounted(() => {
  loadData()
})
</script>

