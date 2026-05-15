import { invoke } from "@tauri-apps/api/core";

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

export async function hideHudWindow(): Promise<void> {
  return invoke("hide_hud_window");
}

export async function showHudWindow(): Promise<void> {
  return invoke("show_hud_window");
}
