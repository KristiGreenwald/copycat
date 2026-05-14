import { writable } from "svelte/store";
import type { SlotInfo } from "$lib/api";

export const slots = writable<SlotInfo[]>([]);
export const hudVisible = writable(false);
