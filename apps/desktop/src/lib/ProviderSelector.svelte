<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'

  const dispatch = createEventDispatcher()

  interface ModelOption {
    provider: string
    model: string
    recommended: boolean
  }

  interface AgentModelOptions {
    agent_id: string
    agent_name: string
    description: string
    selected: ModelOption | null
  }

  let agents: AgentModelOptions[] = []
  let selectedModels: Record<string, string> = {}
  let loading = true
  let error: string | null = null

  onMount(async () => {
    try {
      const response = await fetch('http://127.0.0.1:8080/v1/agents/available-models')
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }
      agents = await response.json()

      // Load saved selections from localStorage or use defaults
      agents.forEach((agent) => {
        const saved = localStorage.getItem(`agent-model-${agent.agent_id}`)
        if (saved) {
          selectedModels[agent.agent_id] = saved
        } else if (agent.selected) {
          selectedModels[agent.agent_id] = `${agent.selected.provider}:${agent.selected.model}`
        }
      })

      loading = false
    } catch (err) {
      error = `Failed to load providers: ${err instanceof Error ? err.message : String(err)}`
      loading = false
    }
  })

  function saveSelections() {
    Object.entries(selectedModels).forEach(([agentId, modelStr]) => {
      localStorage.setItem(`agent-model-${agentId}`, modelStr)
    })
  }

  function handleStart() {
    saveSelections()
    dispatch('provider-selection-complete', selectedModels)
  }

  function getRecommendationLabel(agent: AgentModelOptions): string {
    if (agent.selected) {
      return `${agent.selected.provider}/${agent.selected.model}`
    }
    return 'No providers available'
  }
</script>

<div class="provider-selector">
  <div class="header">
    <h2>Configure AI Providers</h2>
    <p>Select which AI model each agent should use. Available models are checked at startup.</p>
  </div>

  {#if loading}
    <div class="loading">
      <p>Detecting available providers...</p>
    </div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <p>Make sure the governance server is running on 127.0.0.1:8080</p>
    </div>
  {:else if agents.length === 0}
    <div class="error">
      <p>No agents found</p>
    </div>
  {:else}
    <div class="agents-grid">
      {#each agents as agent (agent.agent_id)}
        <div class="agent-config">
          <div class="agent-header">
            <h3>{agent.agent_name}</h3>
            <p class="description">{agent.description}</p>
          </div>

          {#if agent.selected}
            <div class="model-selection">
              <label for={`model-${agent.agent_id}`}>Model:</label>
              <div class="model-input">
                <input
                  id={`model-${agent.agent_id}`}
                  type="text"
                  value={getRecommendationLabel(agent)}
                  disabled
                  class="recommended"
                />
                <span class="badge">Recommended</span>
              </div>
            </div>
          {:else}
            <div class="model-selection error-state">
              <p class="warning">⚠️ No suitable provider found for this agent's requirements</p>
              <p class="hint">Add an API key to .env and restart the server</p>
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <div class="actions">
      <button class="start-button" on:click={handleStart}>
        {agents.some((a) => !a.selected) ? "Continue Offline" : "Start Demo"}
      </button>
      <p class="hint">
        {agents.some((a) => !a.selected)
          ? "The convergence showcase can run with the local deterministic baseline."
          : "Each agent will use its recommended model. Settings are saved in localStorage."}
      </p>
    </div>
  {/if}
</div>

<style>
  .provider-selector {
    padding: 2rem;
    max-width: 900px;
    margin: 0 auto;
  }

  .header {
    margin-bottom: 2rem;
  }

  .header h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
  }

  .header p {
    margin: 0;
    color: #666;
  }

  .loading,
  .error {
    padding: 1.5rem;
    border-radius: 8px;
    text-align: center;
  }

  .loading {
    background: #f0f7ff;
    color: #0066cc;
  }

  .error {
    background: #fff0f0;
    color: #cc0000;
  }

  .agents-grid {
    display: grid;
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .agent-config {
    padding: 1.5rem;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    background: #fafafa;
  }

  .agent-header h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
  }

  .description {
    margin: 0;
    font-size: 0.9rem;
    color: #666;
  }

  .model-selection {
    margin-top: 1rem;
  }

  .model-selection label {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: #333;
  }

  .model-input {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .model-input input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9rem;
  }

  .model-input input.recommended {
    background: #f0f8ff;
    border-color: #0066cc;
    color: #0066cc;
  }

  .badge {
    background: #0066cc;
    color: white;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-size: 0.8rem;
    white-space: nowrap;
  }

  .error-state {
    background: #fff5f5;
  }

  .warning {
    margin: 0 0 0.5rem 0;
    color: #cc6600;
    font-weight: 500;
  }

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: #999;
  }

  .actions {
    text-align: center;
  }

  .start-button {
    padding: 0.75rem 2rem;
    background: #0066cc;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .start-button:hover:not(:disabled) {
    background: #0052a3;
  }

  .start-button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .actions .hint {
    display: block;
    margin-top: 1rem;
  }
</style>
