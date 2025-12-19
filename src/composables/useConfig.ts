import { ref, type Ref } from "vue";
import type { Config } from "../lib/tauri";
import type { TauriService } from "../lib/tauriService";
import { createTauriService } from "../lib/tauriService";

const defaultConfig: Config = {
  active: true,
  poll_interval_ms: 100,
  drives: {},
  theme: "system",
};

export function useConfig(service?: TauriService) {
  const tauriService = service || createTauriService();
  const config: Ref<Config> = ref({ ...defaultConfig });
  const loading = ref(false);
  const error = ref<Error | null>(null);

  async function loadConfig(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      config.value = await tauriService.loadConfig();
    } catch (err) {
      error.value = err instanceof Error ? err : new Error(String(err));
      console.error("Failed to load config:", err);
      config.value = { ...defaultConfig };
    } finally {
      loading.value = false;
    }
  }

  function updateConfig(updatedConfig: Partial<Config>): void {
    config.value = { ...config.value, ...updatedConfig };
  }

  return {
    config,
    loading,
    error,
    loadConfig,
    updateConfig,
  };
}

