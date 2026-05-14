import { writable } from "svelte/store";
import type { AppConfig } from "$lib/api";

export const config = writable<AppConfig | null>(null);
