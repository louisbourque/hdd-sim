<template>
  <nav
    class="w-[280px] bg-white dark:bg-[#2d2d2d] rounded-xl overflow-y-auto shrink-0 shadow-md dark:shadow-[0_2px_8px_rgba(0,0,0,0.3),0_1px_2px_rgba(0,0,0,0.2)]"
    @click="$emit('select', undefined)"
    data-testid="drive-list"
    aria-label="Hard drive list"
  >
    <ul
      class="list-none"
      @click.stop
      role="listbox"
      aria-label="Available hard drives"
    >
      <li
        v-for="(drive, index) in drives"
        :key="drive.name"
        role="option"
        :aria-selected="index === selectedIndex"
        :class="[
          'p-4 mb-2 rounded-lg cursor-pointer transition-all duration-200 ease-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-[#2d2d2d]',
          index === selectedIndex
            ? 'bg-linear-to-br from-blue-100 to-blue-200 dark:from-[#1e3a5f] dark:to-[#2d4a6f] border-l-4 border-blue-500 dark:border-blue-300 shadow-lg dark:shadow-[0_4px_12px_rgba(100,181,246,0.2),0_2px_4px_rgba(100,181,246,0.1)]'
            : 'bg-transparent hover:bg-gray-50 dark:hover:bg-[#3a3a3a] hover:shadow-[0_2px_4px_rgba(0,0,0,0.04)] dark:hover:shadow-[0_2px_4px_rgba(0,0,0,0.2)] hover:-translate-y-px',
        ]"
        @click="$emit('select', index)"
        @keydown.enter.prevent="$emit('select', index)"
        @keydown.space.prevent="$emit('select', index)"
        tabindex="0"
      >
        <div
          class="font-semibold text-sm mb-1 text-gray-900 dark:text-gray-100"
        >
          {{ drive.size_display }} Disk
        </div>
        <div class="text-xs text-gray-600 dark:text-gray-400">
          {{ drive.model }}
        </div>
      </li>
      <li
        v-if="drives.length === 0"
        class="p-4 mb-2 rounded-lg bg-transparent"
        role="status"
        aria-live="polite"
      >
        <div class="font-semibold text-sm text-gray-900 dark:text-gray-100">
          No hard drives found
        </div>
      </li>
    </ul>
  </nav>
</template>

<script setup lang="ts">
import type { Drive } from "../lib/tauri"

defineProps<{
  drives: Drive[]
  selectedIndex: number | undefined
}>()

defineEmits<{
  select: [index: number | undefined]
}>()
</script>
