<template>
  <div
    class="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-600"
    role="status"
    aria-live="polite"
  >
    <div
      class="text-6xl mb-4 drop-shadow-[0_2px_4px_rgba(0,0,0,0.1)]"
      aria-hidden="true"
    >
      💾
    </div>
    <h3 class="text-2xl font-semibold mb-2 text-gray-600 dark:text-gray-400">
      No Device Selected
    </h3>
    <p class="text-sm text-gray-500 dark:text-gray-500">
      Select a device to manage.
    </p>
  </div>
</template>

<script setup lang="ts"></script>
