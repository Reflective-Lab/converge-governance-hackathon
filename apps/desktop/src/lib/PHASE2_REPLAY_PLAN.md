# Phase 2: ReplayRunner Full Integration

## Goal
Wire ReplayRunner adapter pattern to enable replay mode with FlowPlayer orchestration.

## What Exists (Phase 1)
- ReplayRunner class with adapter-based load/save/run pattern
- Component has loadReplayStatus, recordReplaySession, buildOfflineReplaySession
- Component tracks replayStatus, replaySession, replayCursor, runMode

## Phase 2 Work

### Step 1: Create VendorSelectionReplayAdapter
New file: `replay-adapter.ts`

Implements ReplayAdapter interface:
- `loadSession()` → fetch from Tauri or HTTP
- `saveSession()` → invoke Tauri or HTTP POST
- `runStage()` → call runTodayStage (domain-specific)

### Step 2: Wire ReplayRunner
- Create replayRunner instance
- Replace loadReplayStatus → replayRunner.ensureSession()
- Replace recordReplaySession → replayRunner.recordSession()
- Replace takeRun calls → replayRunner.takeRun()
- Replace replayThinkingDelay → replayRunner.playDelay()

### Step 3: Refactor runTodayStage
Current logic:
- mock mode: Tauri or HTTP
- replay mode: takeRun from session
- live mode: Tauri or HTTP with live=true

New logic:
- mock mode: adapter runs stage
- replay mode: replayRunner takes from session
- live mode: adapter runs stage with live=true

### Step 4: Test
- Verify mock mode works
- Verify live mode works (if credentials available)
- Verify replay mode works (with recorded session)
- Verify HITL → approval → promotion flow

## Files to Change
1. Create: replay-adapter.ts
2. Modify: AIProviderEvaluation.svelte
   - Import ReplayRunner, ReplayAdapter
   - Wire adapter and runner
   - Refactor runTodayStage
   - Simplify replay session handling

## Success Criteria
✅ All three modes functional
✅ FlowPlayer + ReplayRunner work together
✅ No Tauri-specific knowledge in generic replay logic
✅ ~100 LOC of replay glue code in adapter
