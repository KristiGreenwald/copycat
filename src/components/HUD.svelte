<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getOccupiedSlots, clearSlot, getHudDuration, getHudAlwaysVisible, getHudAppearance, getHudAccentColor, getConfig, hideHudWindow } from "$lib/api";
  import type { SlotInfo } from "$lib/api";

  let occupiedSlots = $state<SlotInfo[]>([]);
  let prompts = $state<{id: string; name: string; assigned_slot: number|null}[]>([]);
  let visible = $state(false);
  let animClass = $state("");
  let dismissTimer: ReturnType<typeof setTimeout> | null = null;
  let hudDuration = $state(10000);
  let alwaysVisible = $state(false);
  let appearance = $state("glass");
  let accentColor = $state("#d4a054");
  let unlisteners: (() => void)[] = [];

  function getPromptNameForSlot(slotIndex: number): string | null {
    const p = prompts.find(pr => pr.assigned_slot === slotIndex);
    return p ? p.name : null;
  }

  function slotLabel(index: number): string {
    return index === 9 ? "0" : String(index + 1);
  }

  onMount(async () => {
    await loadSettings();

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
      await loadSettings();
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

  async function loadSettings() {
    hudDuration = (await getHudDuration()) * 1000;
    alwaysVisible = await getHudAlwaysVisible();
    appearance = await getHudAppearance();
    accentColor = await getHudAccentColor();
    try {
      const config = await getConfig();
      prompts = config.prompts || [];
    } catch {}
  }

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
  <div class="hud-container {animClass} {appearance}" style="--hud-accent: {accentColor};">
    <div class="hud-header">
      <div class="hud-header-left">
        <div class="hud-mark" style="background: {accentColor};"><img src="/logo.png" class="hud-mark-logo" alt="CC"></div>
        <span class="hud-title">CopyCat</span>
      </div>
      {#if occupiedSlots.length > 0}
        <span class="hud-count">{occupiedSlots.length} slot{occupiedSlots.length !== 1 ? 's' : ''}</span>
      {/if}
    </div>
    {#if occupiedSlots.length > 0}
      <div class="hud-slots">
        {#each occupiedSlots as slot (slot.index)}
          <div class="hud-slot-row">
            <div class="hud-slot-num" style="background: {accentColor};">{slotLabel(slot.index)}</div>
            <div class="hud-slot-body">
              {#if appearance !== 'minimal'}
                {@const promptName = getPromptNameForSlot(slot.index)}
                {#if promptName}
                  <div class="hud-slot-prompt">{promptName.toUpperCase()}</div>
                {/if}
              {/if}
              <div class="hud-slot-snippet">
                {#if slot.processing_state === 'Processing'}
                  <span class="processing-text">{slot.original_preview ?? slot.preview}</span>
                  <span class="processing-dot"></span>
                {:else}
                  {slot.preview || 'Empty'}
                {/if}
              </div>
            </div>
            <button class="hud-slot-clear" onclick={() => handleClear(slot.index)} title="Clear">×</button>
          </div>
        {/each}
      </div>
    {:else}
      <div class="hud-empty">No slots filled yet</div>
    {/if}
  </div>
{/if}

<style>
  .hud-container {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
    background: rgba(28, 22, 18, 0.55);
    backdrop-filter: blur(20px) saturate(160%);
    -webkit-backdrop-filter: blur(20px) saturate(160%);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    box-shadow: 0 12px 40px -10px rgba(0, 0, 0, 0.5);
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  .hud-container.solid {
    background: #1c1815;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }

  .hud-container.minimal {
    background: rgba(28, 22, 18, 0.55);
  }

  .hud-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.1);
  }

  .hud-header-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .hud-mark {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  :global(.hud-mark-logo) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .hud-title {
    font-size: 12px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
  }

  .hud-count {
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.6);
  }

  .hud-slots {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
  }

  .hud-slot-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: rgba(255, 255, 255, 0.07);
    border-radius: 8px;
    transition: background 0.15s;
  }

  .hud-slot-row:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .hud-slot-num {
    width: 22px;
    height: 22px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
    color: #1c1408;
    flex-shrink: 0;
  }

  .hud-slot-body {
    flex: 1;
    min-width: 0;
  }

  .hud-slot-prompt {
    font-size: 9.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: rgba(255, 255, 255, 0.6);
    line-height: 1;
    margin-bottom: 2px;
  }

  .hud-slot-snippet {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.9);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .processing-text {
    color: rgba(255, 255, 255, 0.5);
  }

  .processing-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--hud-accent, #d4a054);
    animation: pulseDot 1s infinite;
    flex-shrink: 0;
  }

  .hud-slot-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.3);
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
    flex-shrink: 0;
    opacity: 0;
    transition: all 0.15s;
  }

  .hud-slot-row:hover .hud-slot-clear {
    opacity: 1;
  }

  .hud-slot-clear:hover {
    background: rgba(255, 80, 80, 0.3);
    color: #ff8080;
  }

  .hud-empty {
    text-align: center;
    padding: 16px 0;
    color: rgba(255, 255, 255, 0.5);
    font-size: 11px;
    font-style: italic;
  }

  .hud-enter {
    animation: slideIn 0.25s ease-out forwards;
  }

  .hud-exit {
    animation: slideOut 0.2s ease-in forwards;
  }

  @keyframes slideIn {
    from { opacity: 0; transform: translateY(12px) scale(0.97); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  @keyframes slideOut {
    from { opacity: 1; transform: translateY(0) scale(1); }
    to { opacity: 0; transform: translateY(12px) scale(0.97); }
  }

  @keyframes pulseDot {
    0%, 100% { transform: scale(1); opacity: 0.6; }
    50% { transform: scale(1.4); opacity: 1; }
  }
</style>
