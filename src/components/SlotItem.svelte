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
  .slot-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    margin: 4px 0;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    transition: background 0.2s ease;
  }

  .slot-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .slot-number {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: #6c8cff;
    color: #fff;
    font-size: 11px;
    font-weight: 700;
    flex-shrink: 0;
    margin-right: 10px;
  }

  .slot-preview {
    flex: 1;
    font-size: 13px;
    color: #f0f0f0;
    font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .slot-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: transparent;
    color: #a0a0a0;
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
    flex-shrink: 0;
    margin-left: 8px;
    transition: all 0.15s ease;
  }

  .slot-close:hover {
    background: #ff6b6b;
    color: #fff;
  }

  .processing-indicator {
    animation: pulse 1.5s ease-in-out infinite;
    color: #6c8cff;
    font-size: 11px;
    margin-left: 6px;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }
</style>
