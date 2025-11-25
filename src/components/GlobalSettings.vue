<template>
  <div class="global-settings">
    <button
      :class="['play-pause-button', { paused: !config.active }]"
      @click="toggleActive"
      :title="config.active ? 'Pause' : 'Play'"
    >
      {{ config.active ? "⏸" : "▶" }}
    </button>

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
  </div>
</template>

<script setup>
import { updateGlobalConfig } from "../lib/tauri";

const props = defineProps({
  config: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(["update"]);

async function toggleActive() {
  try {
    const newActive = !props.config.active;
    await updateGlobalConfig(newActive, undefined);
    emit("update", { active: newActive });
  } catch (error) {
    console.error("Failed to update active state:", error);
  }
}

async function updateInterval(event) {
  try {
    const newInterval = parseInt(event.target.value);
    await updateGlobalConfig(undefined, newInterval);
    emit("update", { poll_interval_ms: newInterval });
  } catch (error) {
    console.error("Failed to update interval:", error);
  }
}
</script>
