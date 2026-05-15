<script lang="ts">
  import { onMount } from "svelte";
  import { getConfig, saveConfig, getPrompts, savePrompt, deletePrompt, aiCheckStatus, aiListModels, aiPullModel } from "$lib/api";
  import type { AppConfig, PromptTemplate, AiStatus } from "$lib/api";

  let config: AppConfig | null = $state(null);
  let prompts: PromptTemplate[] = $state([]);
  let activeTab: string = $state("prompts");
  let aiStatus: AiStatus | null = $state(null);
  let availableModels: { name: string; size: number }[] = $state([]);
  let pulling = $state(false);
  let pullModelName = $state("tinyllama");
  let saveStatus = $state("");
  let copiedCommand = $state(false);
  let loadError = $state("");

  // Prompt editor state (inline, no separate component)
  let editMode = $state(false);
  let editId = $state("");
  let editName = $state("");
  let editTemplate = $state("");
  let editSlot: number | null = $state(null);
  let editIsNew = $state(true);

  const modifierPresets = [
    { label: "⌘ Cmd + ⌥ Option", value: "CmdOrCtrl+Alt" },
    { label: "⌃ Ctrl + ⌥ Option", value: "Control+Alt" },
    { label: "⌘ Cmd + ⇧ Shift", value: "CmdOrCtrl+Shift" },
    { label: "⌃ Ctrl + ⇧ Shift", value: "Control+Shift" },
  ];
  const pastePresets = [
    { label: "⌘ Cmd + ⌥ Option + ⇧ Shift", value: "CmdOrCtrl+Alt+Shift" },
    { label: "⌃ Ctrl + ⌥ Option + ⇧ Shift", value: "Control+Alt+Shift" },
    { label: "⌘ Cmd + ⇧ Shift", value: "CmdOrCtrl+Shift" },
    { label: "⌃ Ctrl + ⇧ Shift", value: "Control+Shift" },
  ];
  const hudKeyPresets = [
    { label: "⌃⌥⌘ Space", value: "Control+Alt+Super+Space" },
    { label: "⌘⌥ V", value: "CmdOrCtrl+Alt+V" },
    { label: "⌘⌥ H", value: "CmdOrCtrl+Alt+H" },
    { label: "⌃⌥⌘ H", value: "Control+Alt+Super+H" },
  ];
  const clearKeyPresets = [
    { label: "⌘⌥ Backspace", value: "CmdOrCtrl+Alt+Backspace" },
    { label: "⌃⌥⌘ Backspace", value: "Control+Alt+Super+Backspace" },
    { label: "⌘⌥ Delete", value: "CmdOrCtrl+Alt+Delete" },
  ];

  onMount(async () => {
    try {
      config = await getConfig();
      prompts = await getPrompts();
      loadError = "";
    } catch (e) {
      loadError = String(e);
      // Fallback: use default prompts so UI is still interactive
      prompts = [
        { id: "summarize", name: "Summarize in 5 bullet points", template: "Summarize the following text in exactly 5 concise bullet points:\n\n{{content}}", assigned_slot: null },
        { id: "fix-grammar", name: "Fix grammar", template: "Fix the grammar and spelling in the following text. Only return the corrected text:\n\n{{content}}", assigned_slot: null },
        { id: "translate-spanish", name: "Translate to Spanish", template: "Translate the following text to Spanish. Only return the translation:\n\n{{content}}", assigned_slot: null },
      ];
    }
    refreshAiStatus();
  });

  async function refreshAiStatus() {
    try {
      aiStatus = await aiCheckStatus();
      if (aiStatus.ollama_running) {
        availableModels = await aiListModels();
      }
    } catch {
      aiStatus = { ollama_running: false, model_available: false };
    }
  }

  async function handlePullModel() {
    pulling = true;
    try {
      await aiPullModel(pullModelName);
      await refreshAiStatus();
      config = await getConfig();
      showSaved();
    } catch (e) {
      saveStatus = `Error: ${e}`;
      setTimeout(() => (saveStatus = ""), 4000);
    }
    pulling = false;
  }

  function showSaved() {
    saveStatus = "Saved!";
    setTimeout(() => (saveStatus = ""), 2000);
  }

  function slotLabel(i: number): string {
    return i === 9 ? "0" : String(i + 1);
  }

  function slotShortcut(i: number): string {
    return `⌘⌥${slotLabel(i)}`;
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    copiedCommand = true;
    setTimeout(() => (copiedCommand = false), 2000);
  }

  // ── Prompt Editor Functions ──

  function openNewPrompt() {
    editMode = true;
    editId = crypto.randomUUID();
    editName = "";
    editTemplate = "";
    editSlot = null;
    editIsNew = true;
  }

  function openEditPrompt(p: PromptTemplate) {
    editMode = true;
    editId = p.id;
    editName = p.name;
    editTemplate = p.template;
    editSlot = p.assigned_slot;
    editIsNew = false;
  }

  function cancelEdit() {
    editMode = false;
  }

  function toggleSlot(i: number) {
    editSlot = editSlot === i ? null : i;
  }

  async function doSavePrompt() {
    try {
      await savePrompt({
        id: editId,
        name: editName,
        template: editTemplate,
        assigned_slot: editSlot,
      });
      prompts = await getPrompts();
      editMode = false;
      showSaved();
    } catch (e) {
      saveStatus = `Error: ${e}`;
      setTimeout(() => (saveStatus = ""), 4000);
    }
  }

  async function doDeletePrompt() {
    try {
      await deletePrompt(editId);
      prompts = await getPrompts();
      editMode = false;
      showSaved();
    } catch (e) {
      saveStatus = `Error: ${e}`;
      setTimeout(() => (saveStatus = ""), 4000);
    }
  }

  async function handleSaveConfig() {
    if (config) {
      await saveConfig(config);
      showSaved();
    }
  }

  function promptForSlot(slotIndex: number): PromptTemplate | undefined {
    return prompts.find(p => p.assigned_slot === slotIndex);
  }
</script>

<div class="settings-container">
  <header class="settings-header">
    <h1>CopyCat Settings</h1>
    {#if saveStatus}
      <span class="save-indicator">{saveStatus}</span>
    {/if}
  </header>

  {#if loadError}
    <div class="error-banner">Failed to load: {loadError}</div>
  {/if}

  <nav class="tabs">
    <button class="tab" class:active={activeTab === "prompts"} onclick={() => { editMode = false; activeTab = "prompts"; }}>
      Prompts
    </button>
    <button class="tab" class:active={activeTab === "shortcuts"} onclick={() => { editMode = false; activeTab = "shortcuts"; }}>
      Shortcuts
    </button>
    <button class="tab" class:active={activeTab === "model"} onclick={() => { editMode = false; activeTab = "model"; }}>
      AI Model
    </button>
    <button class="tab" class:active={activeTab === "general"} onclick={() => { editMode = false; activeTab = "general"; }}>
      General
    </button>
  </nav>

  <div class="tab-content">

    <!-- ═══ PROMPTS TAB ═══ -->
    {#if activeTab === "prompts"}
      <!-- Debug: remove this after testing -->
      <div style="font-size: 10px; color: #555; margin-bottom: 8px;">
        editMode={editMode} | prompts={prompts.length} | editSlot={editSlot}
      </div>

      {#if editMode}
        <div class="prompt-editor">
          <div class="field">
            <label for="ed-name">Name</label>
            <input id="ed-name" type="text" bind:value={editName} placeholder="e.g., Summarize" />
          </div>
          <div class="field">
            <label for="ed-template">Prompt Template</label>
            <textarea id="ed-template" bind:value={editTemplate} placeholder="Use {{content}} where the clipboard content should be inserted" rows="5"></textarea>
            <span class="hint">Use <code>{"{{content}}"}</code> as a placeholder for the copied text</span>
          </div>
          <div class="field">
            <label>Assign to Slot</label>
            <div class="slot-picker">
              {#each Array(10) as _, i}
                <button class="slot-pick-btn" class:selected={editSlot === i} onclick={() => toggleSlot(i)} type="button">
                  {slotLabel(i)}
                </button>
              {/each}
            </div>
            <span class="hint">
              {#if editSlot !== null}
                Assigned to slot {slotLabel(editSlot)} — copy with ⌘⌥{slotLabel(editSlot)}
              {:else}
                Click a number to assign this prompt to a slot
              {/if}
            </span>
          </div>
          <div class="actions">
            <button class="btn btn-primary" onclick={doSavePrompt} disabled={!editName.trim() || !editTemplate.trim()}>Save</button>
            {#if !editIsNew}
              <button class="btn btn-danger" onclick={doDeletePrompt}>Delete</button>
            {/if}
            <button class="btn btn-secondary" onclick={cancelEdit}>Cancel</button>
          </div>
        </div>
      {:else}
        <p class="section-desc">Assign AI prompts to slots. When you copy to a slot with a prompt, the AI will automatically transform the content.</p>
        <div class="prompt-list">
          {#each prompts as p}
            <button class="prompt-row" onclick={() => openEditPrompt(p)}>
              <div class="prompt-info">
                {#if p.assigned_slot !== null}
                  <span class="slot-badge">{slotLabel(p.assigned_slot)}</span>
                {:else}
                  <span class="slot-badge unassigned">—</span>
                {/if}
                <div class="prompt-details">
                  <span class="prompt-name">{p.name}</span>
                  {#if p.assigned_slot !== null}
                    <span class="prompt-shortcut">{slotShortcut(p.assigned_slot)} to copy</span>
                  {:else}
                    <span class="prompt-shortcut unassigned-text">Not assigned to a slot</span>
                  {/if}
                </div>
              </div>
              <span class="prompt-arrow">›</span>
            </button>
          {/each}
        </div>

        <div class="slot-overview">
          <h4>Slot Overview</h4>
          <div class="slot-grid">
            {#each Array(10) as _, i}
              {@const sp = promptForSlot(i)}
              <div class="slot-grid-item" class:has-prompt={!!sp}>
                <span class="slot-grid-num">{slotLabel(i)}</span>
                <span class="slot-grid-label">{sp ? sp.name : 'No prompt'}</span>
              </div>
            {/each}
          </div>
        </div>

        <button class="btn btn-primary" onclick={openNewPrompt}>+ New Prompt</button>
      {/if}

    <!-- ═══ SHORTCUTS TAB ═══ -->
    {:else if activeTab === "shortcuts" && config}
      <div class="section">
        <h3>Keyboard Shortcuts</h3>
        <p class="section-desc">Choose modifier keys for copy/paste. The slot number (1–0) is added automatically.</p>

        <div class="field">
          <label for="copy-mod">Copy to slot — modifier keys + [1-0]</label>
          <select id="copy-mod" bind:value={config.shortcuts.copy_modifier}>
            {#each modifierPresets as preset}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
          <span class="shortcut-preview">Example: {config.shortcuts.copy_modifier.replace('CmdOrCtrl', '⌘').replace('Control', '⌃').replace('Alt', '⌥').replace('Shift', '⇧').replace(/\+/g, ' ')} 1</span>
        </div>

        <div class="field">
          <label for="paste-mod">Paste from slot — modifier keys + [1-0]</label>
          <select id="paste-mod" bind:value={config.shortcuts.paste_modifier}>
            {#each pastePresets as preset}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
          <span class="shortcut-preview">Example: {config.shortcuts.paste_modifier.replace('CmdOrCtrl', '⌘').replace('Control', '⌃').replace('Alt', '⌥').replace('Shift', '⇧').replace(/\+/g, ' ')} 1</span>
        </div>

        <div class="field">
          <label for="toggle-hud">Toggle HUD</label>
          <select id="toggle-hud" bind:value={config.shortcuts.toggle_hud}>
            {#each hudKeyPresets as preset}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <label for="clear-all">Clear all slots</label>
          <select id="clear-all" bind:value={config.shortcuts.clear_all}>
            {#each clearKeyPresets as preset}
              <option value={preset.value}>{preset.label}</option>
            {/each}
          </select>
        </div>

        <button class="btn btn-primary" onclick={handleSaveConfig}>Save Shortcuts</button>
        <p class="hint" style="margin-top: 8px;">Changes require an app restart to take effect.</p>
      </div>

    <!-- ═══ AI MODEL TAB ═══ -->
    {:else if activeTab === "model" && config}
      <div class="section">
        <h3>AI Model (Ollama)</h3>
        <p class="section-desc">CopyCat uses Ollama for local AI processing. All data stays on your machine.</p>

        <div class="field">
          <label>Ollama Status</label>
          {#if aiStatus === null}
            <span style="color: #888;">Checking...</span>
          {:else if aiStatus.ollama_running}
            <span class="status-ok">Running ✓</span>
          {:else}
            <span class="status-pending">Not running ✗</span>
            <div class="install-guide">
              <p class="install-label">Run this in your terminal to install and start Ollama:</p>
              <div class="code-block">
                <code>brew install ollama && ollama serve</code>
                <button class="copy-btn" onclick={() => copyToClipboard('brew install ollama && ollama serve')}>
                  {copiedCommand ? '✓ Copied' : 'Copy'}
                </button>
              </div>
              <p class="hint">After Ollama is running, click Refresh Status below.</p>
            </div>
          {/if}
        </div>

        {#if aiStatus?.ollama_running}
          <div class="field">
            <label>Current Model</label>
            <span class="model-name">{config.ai_model.model_name}</span>
            {#if aiStatus.model_available}
              <span class="status-ok" style="margin-left: 8px;">Available ✓</span>
            {:else}
              <span class="status-pending" style="margin-left: 8px;">Not pulled</span>
            {/if}
          </div>

          {#if availableModels.length > 0}
            <div class="field">
              <label>Installed Models</label>
              {#each availableModels as model}
                <div class="model-row">{model.name} <span class="hint" style="display: inline;">({(model.size / 1e9).toFixed(1)} GB)</span></div>
              {/each}
            </div>
          {/if}

          <div class="field">
            <label for="pull-model">Pull a Model</label>
            <div style="display: flex; gap: 8px;">
              <select id="pull-model" bind:value={pullModelName} style="flex: 1;">
                <option value="tinyllama">tinyllama (1.1B — fast, ~640MB)</option>
                <option value="phi3">phi3 (3.8B — balanced, ~2.2GB)</option>
                <option value="llama3.2:1b">llama3.2:1b (1B — compact, ~1.3GB)</option>
                <option value="llama3.2:3b">llama3.2:3b (3B — capable, ~2GB)</option>
                <option value="mistral">mistral (7B — powerful, ~4.1GB)</option>
              </select>
              <button class="btn btn-primary" onclick={handlePullModel} disabled={pulling}>
                {pulling ? "Pulling..." : "Pull"}
              </button>
            </div>
            <span class="hint">Smaller models are faster. Larger models produce better results.</span>
          </div>
        {/if}

        <button class="btn btn-secondary" style="margin-top: 8px;" onclick={refreshAiStatus}>Refresh Status</button>
      </div>

    <!-- ═══ GENERAL TAB ═══ -->
    {:else if activeTab === "general" && config}
      <div class="section">
        <h3>General</h3>
        <div class="field">
          <label for="hud-duration">HUD display duration (seconds)</label>
          <input id="hud-duration" type="number" min="1" max="30" bind:value={config.hud_duration_secs} />
        </div>
        <div class="field checkbox-field">
          <label>
            <input type="checkbox" bind:checked={config.launch_at_startup} />
            Launch at startup
          </label>
        </div>
        <button class="btn btn-primary" onclick={handleSaveConfig}>Save</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-container {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    max-width: 650px;
    margin: 0 auto;
    padding: 24px;
    color: #f0f0f0;
    background: #1e1e1e;
    min-height: 100vh;
  }
  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }
  h1 { font-size: 20px; font-weight: 700; margin: 0; }
  h3 { font-size: 15px; font-weight: 600; margin: 0 0 12px 0; color: #ddd; }
  h4 { font-size: 13px; font-weight: 600; margin: 16px 0 8px 0; color: #aaa; }
  .save-indicator { font-size: 12px; color: #6c8cff; font-weight: 600; }
  .error-banner { background: #4a1c1c; color: #ff8888; padding: 8px 12px; border-radius: 6px; font-size: 12px; margin-bottom: 12px; }

  .tabs { display: flex; gap: 0; border-bottom: 1px solid #333; margin-bottom: 20px; }
  .tab {
    padding: 8px 16px; border: none; background: none; color: #888;
    font-size: 13px; font-weight: 600; cursor: pointer;
    border-bottom: 2px solid transparent; transition: all 0.15s;
  }
  .tab:hover { color: #ccc; }
  .tab.active { color: #6c8cff; border-bottom-color: #6c8cff; }

  .section { padding: 4px 0; }
  .section-desc { font-size: 12px; color: #888; margin: 0 0 16px 0; }
  .field { margin-bottom: 14px; }

  label {
    display: block; font-size: 12px; font-weight: 600;
    color: #aaa; margin-bottom: 4px;
  }

  input[type="number"], select {
    width: 100%; padding: 10px 12px; font-size: 13px;
    border: 1px solid #3a3a3a; border-radius: 8px;
    background: #252525; color: #f0f0f0;
    outline: none; box-sizing: border-box; transition: border-color 0.15s;
  }
  input:focus, select:focus { border-color: #6c8cff; }
  select {
    appearance: none; -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1L5 5L9 1' stroke='%23666' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat; background-position: right 12px center;
    padding-right: 32px; cursor: pointer;
  }
  select:hover { border-color: #555; }

  .shortcut-preview {
    display: block; font-size: 12px; color: #6c8cff;
    margin-top: 4px; font-family: 'SF Mono', 'Fira Code', monospace;
  }

  .hint { font-size: 11px; color: #666; margin-top: 4px; display: block; }
  code { background: #333; padding: 1px 4px; border-radius: 3px; font-size: 11px; }

  .checkbox-field label {
    display: flex; align-items: center; gap: 8px;
    cursor: pointer; font-size: 13px; color: #ddd;
  }

  /* ── Prompt Editor (inline) ── */
  .prompt-editor { padding: 8px 0; }
  .prompt-editor .field { margin-bottom: 16px; }
  .prompt-editor label { font-size: 13px; color: #ccc; margin-bottom: 6px; }
  .prompt-editor input[type="text"],
  .prompt-editor textarea {
    width: 100%; padding: 10px 12px; font-size: 13px;
    border: 1px solid #3a3a3a; border-radius: 8px;
    background: #252525; color: #f0f0f0;
    outline: none; box-sizing: border-box; transition: border-color 0.15s;
  }
  .prompt-editor input[type="text"]:focus,
  .prompt-editor textarea:focus { border-color: #6c8cff; }
  .prompt-editor textarea { font-family: 'SF Mono', 'Fira Code', monospace; resize: vertical; }

  .slot-picker { display: flex; gap: 6px; }
  .slot-pick-btn {
    width: 36px; height: 36px; border: 1px solid #3a3a3a; border-radius: 8px;
    background: #252525; color: #888; font-size: 14px; font-weight: 700;
    cursor: pointer; transition: all 0.15s;
    display: flex; align-items: center; justify-content: center;
  }
  .slot-pick-btn:hover { border-color: #6c8cff; color: #ddd; }
  .slot-pick-btn.selected { background: #6c8cff; border-color: #6c8cff; color: #fff; }
  .actions { display: flex; gap: 8px; margin-top: 20px; }

  /* ── Prompt List ── */
  .prompt-list { margin-bottom: 16px; }
  .prompt-row {
    display: flex; align-items: center; justify-content: space-between;
    width: 100%; padding: 10px 14px;
    border: 1px solid #3a3a3a; border-radius: 10px; margin-bottom: 6px;
    cursor: pointer; transition: all 0.15s; background: #252525;
    color: inherit; text-align: left; font: inherit;
  }
  .prompt-row:hover { background: #2e2e2e; border-color: #555; }
  .prompt-info { display: flex; align-items: center; gap: 10px; }
  .slot-badge {
    display: inline-flex; align-items: center; justify-content: center;
    width: 28px; height: 28px; border-radius: 7px;
    background: #6c8cff; color: #fff; font-size: 13px; font-weight: 700; flex-shrink: 0;
  }
  .slot-badge.unassigned { background: #444; color: #888; }
  .prompt-details { display: flex; flex-direction: column; gap: 2px; }
  .prompt-name { font-size: 13px; font-weight: 600; color: #ddd; }
  .prompt-shortcut { font-size: 11px; color: #6c8cff; font-family: 'SF Mono', 'Fira Code', monospace; }
  .prompt-shortcut.unassigned-text { color: #666; font-family: inherit; }
  .prompt-arrow { color: #555; font-size: 18px; }

  /* ── Slot Overview Grid ── */
  .slot-overview { margin-top: 16px; margin-bottom: 16px; }
  .slot-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 6px; }
  .slot-grid-item {
    display: flex; flex-direction: column; align-items: center;
    padding: 8px 4px; border: 1px solid #333; border-radius: 6px; gap: 4px;
  }
  .slot-grid-item.has-prompt { border-color: #6c8cff; background: rgba(108, 140, 255, 0.06); }
  .slot-grid-num { font-size: 14px; font-weight: 700; color: #888; }
  .slot-grid-item.has-prompt .slot-grid-num { color: #6c8cff; }
  .slot-grid-label {
    font-size: 9px; color: #555; text-align: center;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%;
  }
  .slot-grid-item.has-prompt .slot-grid-label { color: #aaa; }

  /* ── Model Tab ── */
  .model-name { font-size: 14px; color: #ddd; font-weight: 500; }
  .model-row { font-size: 13px; color: #ccc; padding: 4px 0; }
  .status-ok { color: #4caf50; font-size: 13px; }
  .status-pending { color: #ff9800; font-size: 13px; }
  .install-guide { margin-top: 12px; padding: 12px; background: #252525; border-radius: 8px; border: 1px solid #333; }
  .install-label { font-size: 12px; color: #aaa; margin: 0 0 8px 0; }
  .code-block {
    display: flex; align-items: center; justify-content: space-between;
    background: #1a1a1a; border: 1px solid #444; border-radius: 6px;
    padding: 10px 12px; gap: 8px;
  }
  .code-block code { font-family: 'SF Mono', 'Fira Code', monospace; font-size: 13px; color: #e0e0e0; user-select: all; }
  .copy-btn {
    padding: 4px 10px; border: 1px solid #555; border-radius: 4px;
    background: #333; color: #ccc; font-size: 11px; font-weight: 600;
    cursor: pointer; white-space: nowrap; transition: all 0.15s;
  }
  .copy-btn:hover { background: #444; border-color: #6c8cff; color: #fff; }

  /* ── Buttons ── */
  .btn {
    padding: 9px 18px; border: none; border-radius: 8px;
    font-size: 13px; font-weight: 600; cursor: pointer; transition: all 0.15s;
  }
  .btn-primary { background: #6c8cff; color: #fff; }
  .btn-primary:hover:not(:disabled) { background: #8aa4ff; }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-danger { background: #ff6b6b; color: #fff; }
  .btn-danger:hover { background: #ff8585; }
  .btn-secondary { background: #333; color: #aaa; }
  .btn-secondary:hover { background: #444; color: #ddd; }
</style>
