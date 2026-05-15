// Safe invoke wrapper that handles Tauri IPC not being ready
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const w = window as any;

  // Wait up to 2 seconds for Tauri IPC to be available
  for (let i = 0; i < 20; i++) {
    if (w.__TAURI_INTERNALS__?.invoke) {
      return await w.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    await new Promise((r) => setTimeout(r, 100));
  }

  throw new Error(`Tauri IPC not available (command: ${cmd})`);
}

export interface SlotInfo {
  index: number;
  preview: string;
  occupied: boolean;
  processing_state: "Idle" | "Processing" | "Complete" | { Error: string };
  original_preview: string | null;
  has_prompt: boolean;
}

export interface PromptTemplate {
  id: string;
  name: string;
  template: string;
  assigned_slot: number | null;
}

export interface AppConfig {
  shortcuts: {
    copy_modifier: string;
    paste_modifier: string;
    toggle_hud: string;
    clear_all: string;
  };
  ai_model: {
    model_name: string;
    model_path: string | null;
    download_url: string | null;
    downloaded: boolean;
  };
  prompts: PromptTemplate[];
  hud_duration_secs: number;
  launch_at_startup: boolean;
}

export async function getAllSlots(): Promise<SlotInfo[]> {
  return invoke("get_all_slots");
}

export async function getOccupiedSlots(): Promise<SlotInfo[]> {
  return invoke("get_occupied_slots");
}

export async function copyToSlot(slotIndex: number): Promise<SlotInfo> {
  return invoke("copy_to_slot", { slotIndex });
}

export async function pasteFromSlot(slotIndex: number): Promise<void> {
  return invoke("paste_from_slot", { slotIndex });
}

export async function clearSlot(slotIndex: number): Promise<SlotInfo[]> {
  return invoke("clear_slot", { slotIndex });
}

export async function clearAllSlots(): Promise<SlotInfo[]> {
  return invoke("clear_all_slots");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

export async function getPrompts(): Promise<PromptTemplate[]> {
  return invoke("get_prompts");
}

export async function savePrompt(prompt: PromptTemplate): Promise<void> {
  return invoke("save_prompt", { prompt });
}

export async function deletePrompt(promptId: string): Promise<void> {
  return invoke("delete_prompt", { promptId });
}

export async function getHudDuration(): Promise<number> {
  return invoke("get_hud_duration");
}

export async function getHudAlwaysVisible(): Promise<boolean> {
  return invoke("get_hud_always_visible");
}

export async function hideHudWindow(): Promise<void> {
  return invoke("hide_hud_window");
}

export async function showHudWindow(): Promise<void> {
  return invoke("show_hud_window");
}

// AI commands
export interface AiStatus {
  ollama_running: boolean;
  model_available: boolean;
}

export async function aiCheckStatus(): Promise<AiStatus> {
  return invoke("ai_check_status");
}

export async function aiListModels(): Promise<{ name: string; size: number }[]> {
  return invoke("ai_list_models");
}

export async function aiPullModel(modelName: string): Promise<void> {
  return invoke("ai_pull_model", { modelName });
}

export async function aiGenerate(prompt: string): Promise<string> {
  return invoke("ai_generate", { prompt });
}

export async function aiProcessSlot(slotIndex: number): Promise<void> {
  return invoke("ai_process_slot", { slotIndex });
}
