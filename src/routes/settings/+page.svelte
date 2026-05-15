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
        <div class="prompt-list">
          {#each prompts as prompt}
            <div class="prompt-row" onclick={() => (editingPrompt = { ...prompt })} role="button" tabindex="0">
              <div class="prompt-info">
                <span class="prompt-name">{prompt.name}</span>
                {#if prompt.assigned_slot !== null}
                  <span class="prompt-slot">Slot {slotLabel(prompt.assigned_slot)}</span>
                {/if}
              </div>
              <span class="prompt-arrow">→</span>
            </div>
          {/each}
        </div>
        <button class="btn btn-primary" onclick={newPrompt}>+ New Prompt</button>
      {/if}

    {:else if activeTab === "shortcuts" && config}
      <div class="section">
        <h3>Keyboard Shortcuts</h3>
        <p class="section-desc">Shortcuts use modifier keys + number (1-0) for each slot.</p>

        <div class="field">
          <label for="copy-mod">Copy to slot</label>
          <input id="copy-mod" type="text" bind:value={config.shortcuts.copy_modifier} />
          <span class="hint">+ [1-0] for each slot</span>
        </div>
        <div class="field">
          <label for="paste-mod">Paste from slot</label>
          <input id="paste-mod" type="text" bind:value={config.shortcuts.paste_modifier} />
          <span class="hint">+ [1-0] for each slot</span>
        </div>
        <div class="field">
          <label for="toggle-hud">Toggle HUD</label>
          <input id="toggle-hud" type="text" bind:value={config.shortcuts.toggle_hud} />
        </div>
        <div class="field">
          <label for="clear-all">Clear all slots</label>
          <input id="clear-all" type="text" bind:value={config.shortcuts.clear_all} />
        </div>

        <button class="btn btn-primary" onclick={handleSaveConfig}>Save Shortcuts</button>
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
            <p class="hint">Install Ollama: <code>brew install ollama</code> then run <code>ollama serve</code></p>
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
                <div class="model-row">{model.name} <span class="hint">({(model.size / 1e9).toFixed(1)} GB)</span></div>
              {/each}
            </div>
          {/if}

          <div class="field">
            <label for="pull-model">Pull a Model</label>
            <div style="display: flex; gap: 8px;">
              <input id="pull-model" type="text" bind:value={pullModelName} placeholder="e.g., tinyllama, phi3, llama3.2:1b" />
              <button class="btn btn-primary" onclick={handlePullModel} disabled={pulling || !pullModelName.trim()}>
                {pulling ? "Pulling..." : "Pull"}
              </button>
            </div>
            <span class="hint">Recommended small models: tinyllama, phi3, llama3.2:1b</span>
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

  h1 {
    font-size: 20px;
    font-weight: 700;
    margin: 0;
  }

  h3 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 12px 0;
    color: #ddd;
  }

  .save-indicator {
    font-size: 12px;
    color: #6c8cff;
    font-weight: 600;
  }

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

  .tab:hover {
    color: #ccc;
  }

  .tab.active {
    color: #6c8cff;
    border-bottom-color: #6c8cff;
  }

  .section {
    padding: 4px 0;
  }

  .section-desc {
    font-size: 12px;
    color: #888;
    margin: 0 0 16px 0;
  }

  .field {
    margin-bottom: 14px;
  }

  label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: #aaa;
    margin-bottom: 4px;
  }

  input[type="text"],
  input[type="number"] {
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

  input:focus {
    border-color: #6c8cff;
  }

  .hint {
    font-size: 11px;
    color: #666;
    margin-top: 4px;
    display: block;
  }

  .checkbox-field label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: #ddd;
  }

  .prompt-list {
    margin-bottom: 16px;
  }

  .prompt-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border: 1px solid #333;
    border-radius: 8px;
    margin-bottom: 6px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .prompt-row:hover {
    background: #2a2a2a;
  }

  .prompt-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .prompt-name {
    font-size: 13px;
    font-weight: 600;
    color: #ddd;
  }

  .prompt-slot {
    font-size: 11px;
    color: #6c8cff;
    background: rgba(108, 140, 255, 0.12);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .prompt-arrow {
    color: #555;
  }

  .model-name {
    font-size: 14px;
    color: #ddd;
    font-weight: 500;
  }

  .model-row {
    font-size: 13px;
    color: #ccc;
    padding: 4px 0;
  }

  .status-ok {
    color: #4caf50;
    font-size: 13px;
  }

  .status-pending {
    color: #ff9800;
    font-size: 13px;
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

  .btn-primary:hover {
    background: #8aa4ff;
  }
</style>
