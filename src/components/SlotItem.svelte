<script lang="ts">
  import type { SlotInfo } from "$lib/api";

  interface Props {
    slot: SlotInfo;
    onClear: (index: number) => void;
  }

  let { slot, onClear }: Props = $props();

  function displayNumber(index: number): string {
    return index === 9 ? "0" : String(index + 1);
  }

  function isProcessing(state: SlotInfo["processing_state"]): boolean {
    return state === "Processing";
  }
</script>

<div class="slot-item">
  <span class="slot-number">{displayNumber(slot.index)}</span>
  <span class="slot-preview">
    {#if isProcessing(slot.processing_state)}
      <span style="color: var(--text-secondary)">{slot.original_preview ?? slot.preview}</span>
      <span class="processing-indicator">⟳ AI</span>
    {:else}
      {slot.preview}
    {/if}
    {#if slot.has_prompt}
      <span style="color: var(--accent); font-size: 10px; margin-left: 4px;">✦</span>
    {/if}
  </span>
  <button
    class="slot-close"
    onclick={() => onClear(slot.index)}
    title="Clear slot {displayNumber(slot.index)}"
  >
    ×
  </button>
</div>

<style>
  @import "../styles/glassmorphic.css";
</style>
