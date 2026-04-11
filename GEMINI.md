# Gemini CLI Entrypoint

Read and follow `AGENTS.md` — it is the canonical project documentation.

## Gemini-Specific Mandates

1. **Workflow Intents**: Treat slash commands (e.g., `/focus`, `/sync`, `/status`, `/checkpoint`) as workflow intents. Execute them using the corresponding `just` recipes or scripts in `scripts/workflow/`.
2. **Knowledge Persistence**: When you learn something that should outlive this session, update the relevant page in `kb/` or create a new one. Do NOT rely solely on `save_memory` for project-level architectural or domain knowledge.
3. **Sub-Agent Usage**: 
   - Use `codebase_investigator` for deep architectural research or bug root-cause analysis.
   - Use `generalist` for batch refactoring or high-volume file operations across the workspace.
4. **Architectural Guardrails**: Rigorously enforce the rules in `AGENTS.md`:
   - No `unsafe` Rust.
   - Svelte/Tauri for UI (no React).
   - Rust-first for all system logic and agent orchestration.
5. **Tool Preference**: Prefer `grep_search` and `glob` for initial discovery over reading entire files or the whole `kb/` directory. Lazy-load documentation as needed.
