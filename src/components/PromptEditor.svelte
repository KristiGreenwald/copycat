<script lang="ts">
  import type { PromptTemplate } from "$lib/api";

  interface Props {
    prompt: PromptTemplate;
    onSave: (prompt: PromptTemplate) => void;
    onDelete: (promptId: string) => void;
    onCancel: () => void;
  }

  let { prompt, onSave, onDelete, onCancel }: Props = $props();

  let name = $state(prompt.name);
  let template = $state(prompt.template);
  let assignedSlot = $state<number | null>(prompt.assigned_slot);
  let isNew = $derived(!prompt.name);

  function handleSave() {
    onSave({
      id: prompt.id || crypto.randomUUID(),
      name,
      template,
      assigned_slot: assignedSlot,
    });
  }

  function slotLabel(i: number): string {
    return i === 9 ? "Slot 0" : `Slot ${i + 1}`;
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
    <label for="prompt-slot">Assign to Slot</label>
    <select id="prompt-slot" bind:value={assignedSlot}>
      <option value={null}>None</option>
      {#each Array(10) as _, i}
        <option value={i}>{slotLabel(i)}</option>
      {/each}
    </select>
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

  input, textarea, select {
    width: 100%;
    padding: 8px 12px;
    font-size: 13px;
    border: 1px solid #444;
    border-radius: 6px;
    background: #2a2a2a;
    color: #f0f0f0;
    outline: none;
    box-sizing: border-box;
  }

  input:focus, textarea:focus, select:focus {
    border-color: #6c8cff;
  }

  textarea {
    font-family: 'SF Mono', 'Fira Code', monospace;
    resize: vertical;
  }

  .hint {
    font-size: 11px;
    color: #888;
    margin-top: 4px;
    display: block;
  }

  code {
    background: #333;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 11px;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-primary {
    background: #6c8cff;
    color: #fff;
  }
  .btn-primary:hover:not(:disabled) {
    background: #8aa4ff;
  }
  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-danger {
    background: #ff6b6b;
    color: #fff;
  }
  .btn-danger:hover {
    background: #ff8585;
  }

  .btn-secondary {
    background: #444;
    color: #ccc;
  }
  .btn-secondary:hover {
    background: #555;
  }
</style>
