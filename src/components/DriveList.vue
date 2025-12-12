<template>
  <div class="sidebar">
    <ul class="drive-list">
      <li
        v-for="(drive, index) in drives"
        :key="drive.name"
        :class="['drive-item', { selected: index === selectedIndex }]"
        @click="$emit('select', index)"
      >
        <div class="drive-item-title">{{ drive.size_display }} Disk</div>
        <div class="drive-item-model">{{ drive.model }}</div>
      </li>
      <li v-if="drives.length === 0" class="drive-item">
        <div class="drive-item-title">No hard drives found</div>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import type { Drive } from "../lib/tauri"

defineProps<{
  drives: Drive[]
  selectedIndex: number | undefined
}>()

defineEmits<{
  select: [index: number]
}>()
</script>
