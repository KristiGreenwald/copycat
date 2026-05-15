<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getOccupiedSlots, clearSlot, getHudDuration, getHudAlwaysVisible, hideHudWindow } from "$lib/api";
  import type { SlotInfo } from "$lib/api";
  import SlotItem from "./SlotItem.svelte";

  let occupiedSlots = $state<SlotInfo[]>([]);
  let visible = $state(false);
  let animClass = $state("");
  let dismissTimer: ReturnType<typeof setTimeout> | null = null;
  let hudDuration = $state(10000);
  let alwaysVisible = $state(false);
  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    hudDuration = (await getHudDuration()) * 1000;
    alwaysVisible = await getHudAlwaysVisible();

    // If always-visible is on, show HUD immediately
    if (alwaysVisible) {
      await refreshSlots();
      showHud();
    }

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
      if (visible && !alwaysVisible) {
        await hideHud();
      } else {
        await refreshSlots();
        showHud();
      }
    });
    unlisteners.push(unlisten3);

    const unlisten4 = await listen("show-hud", async () => {
      // Reload settings in case always-visible changed
      alwaysVisible = await getHudAlwaysVisible();
      await refreshSlots();
      showHud();
    });
    unlisteners.push(unlisten4);

    const unlisten5 = await listen("slots-updated", async () => {
      await refreshSlots();
      if (occupiedSlots.length === 0 && !alwaysVisible) {
        await hideHud();
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
    if (!alwaysVisible) {
      dismissTimer = setTimeout(() => hideHud(), hudDuration);
    }
  }

  async function hideHud() {
    if (dismissTimer) clearTimeout(dismissTimer);
    dismissTimer = null;
    animClass = "hud-exit";
    setTimeout(async () => {
      visible = false;
      animClass = "";
      await hideHudWindow();
    }, 200);
  }

  async function handleClear(index: number) {
    await clearSlot(index);
    await refreshSlots();
    if (occupiedSlots.length === 0) {
      await hideHud();
    }
  }
</script>

{#if visible}
  <div class="hud-container {animClass}">
    <div class="hud-header">
      <span class="hud-title">CopyCat</span>
      {#if occupiedSlots.length > 0}
        <span class="hud-count">{occupiedSlots.length} slot{occupiedSlots.length !== 1 ? 's' : ''}</span>
      {/if}
    </div>
    {#if occupiedSlots.length > 0}
      <div class="hud-slots">
        {#each occupiedSlots as slot (slot.index)}
          <SlotItem {slot} onClear={handleClear} />
        {/each}
      </div>
    {:else}
      <div class="hud-empty">All slots empty</div>
    {/if}
  </div>
{/if}

<style>
  .hud-container {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: 14px;
    overflow-y: auto;
    background: rgba(30, 30, 30, 0.75);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
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
    color: #f0f0f0;
    letter-spacing: 0.5px;
  }

  .hud-count {
    font-size: 11px;
    color: #a0a0a0;
  }

  .hud-slots {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .hud-empty {
    text-align: center;
    padding: 16px 0;
    color: #a0a0a0;
    font-size: 12px;
  }

  .hud-enter {
    animation: slideIn 0.25s ease-out forwards;
  }

  .hud-exit {
    animation: slideOut 0.2s ease-in forwards;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes slideOut {
    from {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
    to {
      opacity: 0;
      transform: translateY(20px) scale(0.95);
    }
  }
</style>
