import { ref, type Ref } from "vue";
import type { Drive } from "../lib/tauri";
import type { TauriService } from "../lib/tauriService";
import { createTauriService } from "../lib/tauriService";

export function useDrives(service?: TauriService) {
  const tauriService = service || createTauriService();
  const drives: Ref<Drive[]> = ref([]);
  const loading = ref(false);
  const error = ref<Error | null>(null);

  async function loadDrives(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      drives.value = await tauriService.getDrives();
    } catch (err) {
      error.value = err instanceof Error ? err : new Error(String(err));
      console.error("Failed to load drives:", err);
    } finally {
      loading.value = false;
    }
  }

  function updateDrive(updatedDrive: Drive): void {
    const index = drives.value.findIndex((d) => d.name === updatedDrive.name);
    if (index !== -1) {
      drives.value[index] = updatedDrive;
    }
  }

  return {
    drives,
    loading,
    error,
    loadDrives,
    updateDrive,
  };
}

