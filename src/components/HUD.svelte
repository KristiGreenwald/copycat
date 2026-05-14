<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getAllSlots, getOccupiedSlots, clearSlot, getHudDuration } from "$lib/api";
  import type { SlotInfo } from "$lib/api";
  import SlotItem from "./SlotItem.svelte";

  let occupiedSlots = $state<SlotInfo[]>([]);
  let visible = $state(false);
  let animClass = $state("");
  let dismissTimer: ReturnType<typeof setTimeout> | null = null;
  let hudDuration = $state(5000);
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    hudDuration = (await getHudDuration()) * 1000;

    const unlisten1 = await listen<SlotInfo>("slot-copied", async () => {
      await refreshSlots();
      showHud();
    });
    unlisteners.push(unlisten1);

    const unlisten2 = await listen<number>("slot-pasted", () => {
      showHud();
    });
    unlisteners.push(unlisten2);

    const unlisten3 = await listen("toggle-hud", async () => {
      if (visible) {
        hideHud();
      } else {
        await refreshSlots();
        showHud();
      }
    });
    unlisteners.push(unlisten3);

    const unlisten4 = await listen("show-hud", async () => {
      await refreshSlots();
      showHud();
    });
    unlisteners.push(unlisten4);

    const unlisten5 = await listen("slots-updated", async () => {
      await refreshSlots();
      if (occupiedSlots.length === 0) {
        hideHud();
      }
    });
    unlisteners.push(unlisten5);
  });

  onDestroy(() => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    if (dismissTimer) clearTimeout(dismissTimer);
  });

  async function refreshSlots() {
    occupiedSlots = await getOccupiedSlots();
  }

  function showHud() {
    if (dismissTimer) clearTimeout(dismissTimer);
    visible = true;
    animClass = "hud-enter";
    dismissTimer = setTimeout(() => hideHud(), hudDuration);
  }

  function hideHud() {
    if (dismissTimer) clearTimeout(dismissTimer);
    animClass = "hud-exit";
    setTimeout(() => {
      visible = false;
      animClass = "";
    }, 200);
  }

  async function handleClear(index: number) {
    await clearSlot(index);
    await refreshSlots();
    if (occupiedSlots.length === 0) {
      hideHud();
    }
  }
</script>

{#if visible && occupiedSlots.length > 0}
  <div class="hud-container glass-panel {animClass}">
    <div class="hud-header">
      <span class="hud-title">ClipX</span>
      <span class="hud-count">{occupiedSlots.length} slot{occupiedSlots.length !== 1 ? 's' : ''}</span>
    </div>
    <div class="hud-slots">
      {#each occupiedSlots as slot (slot.index)}
        <SlotItem {slot} onClear={handleClear} />
      {/each}
    </div>
  </div>
{/if}

<style>
  @import "../styles/glassmorphic.css";

  .hud-container {
    position: fixed;
    bottom: 16px;
    right: 16px;
    width: 280px;
    max-height: 380px;
    padding: 14px;
    overflow-y: auto;
    z-index: 9999;
  }

  .hud-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .hud-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 0.5px;
  }

  .hud-count {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .hud-slots {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
