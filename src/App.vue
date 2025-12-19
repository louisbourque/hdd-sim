<template>
  <div class="flex flex-1 overflow-hidden p-4 gap-4">
    <DriveList
      :drives="drives"
      :selected-index="selectedDriveIndex"
      @select="handleDriveSelect"
    />
    <div class="flex flex-1 flex-col overflow-hidden gap-4">
      <div
        class="flex-1 overflow-y-auto p-8 bg-white dark:bg-[#2d2d2d] rounded-xl shadow-md dark:shadow-[0_2px_8px_rgba(0,0,0,0.3),0_1px_2px_rgba(0,0,0,0.2)]"
      >
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
import { ref, onMounted, watch } from "vue"
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

function applyTheme(theme: string): void {
  const root = document.documentElement
  const isSystemDark = window.matchMedia("(prefers-color-scheme: dark)").matches

  if (theme === "dark" || (theme === "system" && isSystemDark)) {
    root.classList.add("dark")
  } else {
    root.classList.remove("dark")
  }
}

watch(
  () => config.value?.theme || "system",
  (theme) => {
    applyTheme(theme)
  },
  { immediate: true }
)

window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    const theme = config.value?.theme || "system"
    if (theme === "system") {
      applyTheme(theme)
    }
  })

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
