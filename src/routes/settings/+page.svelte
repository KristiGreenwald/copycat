<script lang="ts">
  import { onMount } from "svelte";
  import { getConfig, saveConfig, getPrompts, savePrompt, deletePrompt, aiCheckStatus, aiListModels, aiPullModel } from "$lib/api";
  import type { AppConfig, PromptTemplate, AiStatus } from "$lib/api";
  import PromptEditor from "../../components/PromptEditor.svelte";

  let config = $state<AppConfig | null>(null);
  let prompts = $state<PromptTemplate[]>([]);
  let editingPrompt = $state<PromptTemplate | null>(null);
  let activeTab = $state<"prompts" | "shortcuts" | "model" | "general">("prompts");
  let aiStatus = $state<AiStatus | null>(null);
  let availableModels = $state<{ name: string; size: number }[]>([]);
  let pulling = $state(false);
  let pullModelName = $state("tinyllama");
  let saveStatus = $state("");
  let copiedCommand = $state(false);

  // Shortcut builder state
  const modifierPresets = [
    { label: "⌘ Cmd + ⌥ Option", value: "CmdOrCtrl+Alt" },
    { label: "⌃ Ctrl + ⌥ Option", value: "Control+Alt" },
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
    config = await getConfig();
    prompts = await getPrompts();
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
      if (config) {
        config = await getConfig();
      }
      showSaved();
    } catch (e) {
      saveStatus = `Error: ${e}`;
      setTimeout(() => (saveStatus = ""), 4000);
    }
    pulling = false;
  }

  async function handleSavePrompt(prompt: PromptTemplate) {
    await savePrompt(prompt);
    prompts = await getPrompts();
    editingPrompt = null;
    showSaved();
  }

  async function handleDeletePrompt(promptId: string) {
    await deletePrompt(promptId);
    prompts = await getPrompts();
    editingPrompt = null;
    showSaved();
  }

  async function handleSaveConfig() {
    if (config) {
      await saveConfig(config);
      showSaved();
    }
  }

  function showSaved() {
    saveStatus = "Saved!";
    setTimeout(() => (saveStatus = ""), 2000);
  }

  function newPrompt() {
    editingPrompt = {
      id: crypto.randomUUID(),
      name: "",
      template: "",
      assigned_slot: null,
    };
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

  function promptForSlot(slotIndex: number): PromptTemplate | undefined {
    return prompts.find(p => p.assigned_slot === slotIndex);
  }
</script>

<div class="settings-container">
  <header class="settings-header">
    <h1>ClipX Settings</h1>
    {#if saveStatus}
      <span class="save-indicator">{saveStatus}</span>
    {/if}
  </header>

  <nav class="tabs">
    <button class="tab" class:active={activeTab === "prompts"} onclick={() => (activeTab = "prompts")}>
      Prompts
    </button>
    <button class="tab" class:active={activeTab === "shortcuts"} onclick={() => (activeTab = "shortcuts")}>
      Shortcuts
    </button>
    <button class="tab" class:active={activeTab === "model"} onclick={() => (activeTab = "model")}>
      AI Model
    </button>
    <button class="tab" class:active={activeTab === "general"} onclick={() => (activeTab = "general")}>
      General
    </button>
  </nav>

  <div class="tab-content">
    {#if activeTab === "prompts"}
      {#if editingPrompt}
        <PromptEditor
          prompt={editingPrompt}
          onSave={handleSavePrompt}
          onDelete={handleDeletePrompt}
          onCancel={() => (editingPrompt = null)}
        />
      {:else}
        <p class="section-desc">Assign AI prompts to slots. When you copy to a slot with a prompt, the AI will automatically transform the content.</p>
        <div class="prompt-list">
          {#each prompts as prompt}
            <button class="prompt-row" onclick={() => (editingPrompt = { ...prompt })}>
              <div class="prompt-info">
                {#if prompt.assigned_slot !== null}
                  <span class="slot-badge">{slotLabel(prompt.assigned_slot)}</span>
                {:else}
                  <span class="slot-badge unassigned">—</span>
                {/if}
                <div class="prompt-details">
                  <span class="prompt-name">{prompt.name}</span>
                  {#if prompt.assigned_slot !== null}
                    <span class="prompt-shortcut">{slotShortcut(prompt.assigned_slot)} to copy</span>
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
              {@const p = promptForSlot(i)}
              <div class="slot-grid-item" class:has-prompt={!!p}>
                <span class="slot-grid-num">{slotLabel(i)}</span>
                <span class="slot-grid-label">{p ? p.name : 'No prompt'}</span>
              </div>
            {/each}
          </div>
        </div>

        <button class="btn btn-primary" onclick={newPrompt}>+ New Prompt</button>
      {/if}

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
            <option value="CmdOrCtrl+Alt+Shift">⌘ Cmd + ⌥ Option + ⇧ Shift</option>
            <option value="Control+Alt+Shift">⌃ Ctrl + ⌥ Option + ⇧ Shift</option>
            <option value="CmdOrCtrl+Shift">⌘ Cmd + ⇧ Shift</option>
            <option value="Control+Shift">⌃ Ctrl + ⇧ Shift</option>
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

    {:else if activeTab === "model" && config}
      <div class="section">
        <h3>AI Model (Ollama)</h3>
        <p class="section-desc">ClipX uses Ollama for local AI processing. All data stays on your machine.</p>

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

  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid #333;
    margin-bottom: 20px;
  }

  .tab {
    padding: 8px 16px;
    border: none;
    background: none;
    color: #888;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }
  .tab:hover { color: #ccc; }
  .tab.active { color: #6c8cff; border-bottom-color: #6c8cff; }

  .section { padding: 4px 0; }
  .section-desc { font-size: 12px; color: #888; margin: 0 0 16px 0; }

  .field { margin-bottom: 14px; }

  label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: #aaa;
    margin-bottom: 4px;
  }

  input[type="text"],
  input[type="number"],
  select {
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
  input:focus, select:focus { border-color: #6c8cff; }

  .hint { font-size: 11px; color: #666; margin-top: 4px; display: block; }

  .shortcut-preview {
    display: block;
    font-size: 12px;
    color: #6c8cff;
    margin-top: 4px;
    font-family: 'SF Mono', 'Fira Code', monospace;
  }

  .checkbox-field label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: #ddd;
  }

  /* Prompts */
  .prompt-list { margin-bottom: 16px; }

  .prompt-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid #333;
    border-radius: 8px;
    margin-bottom: 6px;
    cursor: pointer;
    transition: background 0.15s;
    background: none;
    color: inherit;
    text-align: left;
    font: inherit;
  }
  .prompt-row:hover { background: #2a2a2a; }

  .prompt-info { display: flex; align-items: center; gap: 10px; }

  .slot-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    background: #6c8cff;
    color: #fff;
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
  }
  .slot-badge.unassigned { background: #444; color: #888; }

  .prompt-details { display: flex; flex-direction: column; gap: 2px; }
  .prompt-name { font-size: 13px; font-weight: 600; color: #ddd; }
  .prompt-shortcut { font-size: 11px; color: #6c8cff; font-family: 'SF Mono', 'Fira Code', monospace; }
  .prompt-shortcut.unassigned-text { color: #666; font-family: inherit; }
  .prompt-arrow { color: #555; font-size: 18px; }

  /* Slot overview grid */
  .slot-overview { margin-top: 16px; margin-bottom: 16px; }
  .slot-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 6px; }
  .slot-grid-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 8px 4px;
    border: 1px solid #333;
    border-radius: 6px;
    gap: 4px;
  }
  .slot-grid-item.has-prompt { border-color: #6c8cff; background: rgba(108, 140, 255, 0.06); }
  .slot-grid-num { font-size: 14px; font-weight: 700; color: #888; }
  .slot-grid-item.has-prompt .slot-grid-num { color: #6c8cff; }
  .slot-grid-label {
    font-size: 9px;
    color: #555;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .slot-grid-item.has-prompt .slot-grid-label { color: #aaa; }

  /* Model */
  .model-name { font-size: 14px; color: #ddd; font-weight: 500; }
  .model-row { font-size: 13px; color: #ccc; padding: 4px 0; }
  .status-ok { color: #4caf50; font-size: 13px; }
  .status-pending { color: #ff9800; font-size: 13px; }

  .install-guide {
    margin-top: 12px;
    padding: 12px;
    background: #252525;
    border-radius: 8px;
    border: 1px solid #333;
  }
  .install-label { font-size: 12px; color: #aaa; margin: 0 0 8px 0; }

  .code-block {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 6px;
    padding: 10px 12px;
    gap: 8px;
  }
  .code-block code {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
    color: #e0e0e0;
    user-select: all;
  }
  .copy-btn {
    padding: 4px 10px;
    border: 1px solid #555;
    border-radius: 4px;
    background: #333;
    color: #ccc;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s;
  }
  .copy-btn:hover { background: #444; border-color: #6c8cff; color: #fff; }

  /* Buttons */
  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }
  .btn-primary { background: #6c8cff; color: #fff; }
  .btn-primary:hover:not(:disabled) { background: #8aa4ff; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { background: #444; color: #ccc; }
  .btn-secondary:hover { background: #555; }
</style>
