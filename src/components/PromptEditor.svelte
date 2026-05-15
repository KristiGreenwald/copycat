<script lang="ts">
  import type { PromptTemplate } from "$lib/api";

  interface Props {
    prompt: PromptTemplate;
    onSave: (prompt: PromptTemplate) => void;
    onDelete: (promptId: string) => void;
    onCancel: () => void;
  }

  let { prompt, onSave, onDelete, onCancel }: Props = $props();

  // These initialize once on mount. The {#key} wrapper in the parent
  // ensures this component is recreated when switching prompts.
  let name = $state(prompt.name);
  let template = $state(prompt.template);
  let assignedSlot: number | null = $state(prompt.assigned_slot ?? null);
  let isNew = !prompt.name;

  function handleSave() {
    onSave({
      id: prompt.id || crypto.randomUUID(),
      name,
      template,
      assigned_slot: assignedSlot,
    });
  }

  function selectSlot(i: number) {
    if (assignedSlot === i) {
      assignedSlot = null;
    } else {
      assignedSlot = i;
    }
  }

  function slotDisplay(i: number): string {
    return i === 9 ? "0" : String(i + 1);
  }
</script>

<div class="prompt-editor">
  <div class="field">
    <label for="prompt-name">Name</label>
    <input id="prompt-name" type="text" bind:value={name} placeholder="e.g., Summarize" />
  </div>

  <div class="field">
    <label for="prompt-template">Prompt Template</label>
    <textarea
      id="prompt-template"
      bind:value={template}
      placeholder="Use {{content}} where the clipboard content should be inserted"
      rows="5"
    ></textarea>
    <span class="hint">Use <code>{"{{content}}"}</code> as a placeholder for the copied text</span>
  </div>

  <div class="field">
    <label>Assign to Slot</label>
    <div class="slot-picker">
      {#each Array(10) as _, i}
        <button
          class="slot-pick-btn"
          class:selected={assignedSlot === i}
          onclick={() => selectSlot(i)}
          type="button"
        >
          {slotDisplay(i)}
        </button>
      {/each}
    </div>
    <span class="hint">
      {#if assignedSlot !== null}
        Assigned to slot {slotDisplay(assignedSlot)} — copy with ⌘⌥{slotDisplay(assignedSlot)}
      {:else}
        Click a number to assign this prompt to a slot
      {/if}
    </span>
  </div>

  <div class="actions">
    <button class="btn btn-primary" onclick={handleSave} disabled={!name.trim() || !template.trim()}>
      Save
    </button>
    {#if !isNew}
      <button class="btn btn-danger" onclick={() => onDelete(prompt.id)}>Delete</button>
    {/if}
    <button class="btn btn-secondary" onclick={onCancel}>Cancel</button>
  </div>
</div>

<style>
  .prompt-editor {
    padding: 16px 0;
  }

  .field {
    margin-bottom: 16px;
  }

  label {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: #ccc;
    margin-bottom: 6px;
  }

  input, textarea {
    width: 100%;
    padding: 10px 12px;
    font-size: 13px;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    background: #252525;
    color: #f0f0f0;
    outline: none;
    box-sizing: border-box;
    transition: border-color 0.15s;
  }

  input:focus, textarea:focus {
    border-color: #6c8cff;
  }

  textarea {
    font-family: 'SF Mono', 'Fira Code', monospace;
    resize: vertical;
  }

  .hint {
    font-size: 11px;
    color: #888;
    margin-top: 6px;
    display: block;
  }

  code {
    background: #333;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 11px;
  }

  .slot-picker {
    display: flex;
    gap: 6px;
  }

  .slot-pick-btn {
    width: 36px;
    height: 36px;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    background: #252525;
    color: #888;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .slot-pick-btn:hover {
    border-color: #6c8cff;
    color: #ddd;
  }

  .slot-pick-btn.selected {
    background: #6c8cff;
    border-color: #6c8cff;
    color: #fff;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-primary { background: #6c8cff; color: #fff; }
  .btn-primary:hover:not(:disabled) { background: #8aa4ff; }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-danger { background: #ff6b6b; color: #fff; }
  .btn-danger:hover { background: #ff8585; }
  .btn-secondary { background: #333; color: #aaa; }
  .btn-secondary:hover { background: #444; color: #ddd; }
</style>
