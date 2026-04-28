# Phase 2: ReplayRunner Full Integration — COMPLETE ✅

## Goal
Wire ReplayRunner adapter pattern to enable replay mode with FlowPlayer orchestration.

## Implementation Summary

### ✅ Step 1: Create VendorSelectionReplayAdapter
File: `replay-adapter.ts` (250 LOC)

Implements ReplayAdapter interface:
- `loadSession()` - fetch from Tauri or HTTP
- `saveSession()` - invoke Tauri or HTTP POST (required by interface, not used)
- `runStage(stage, inputs)` - execute mock or live vendor-selection runs
- `recordSession()` - record new replay session with progress flag
- `buildOfflineSession()` - build offline backup with progress flag
- `getStatus()` - fetch replay session availability
- `clearSession()` - clear replay state
- `resetExperience()` - reset aggregated experience
- `normalizeStage(stage)` - vendor-selection stage name mapping
- `formInputs(stage, live)` - construct vendor-selection API inputs

Also includes helper functions:
- `emptyReplayStatus()` - empty state object
- `statusFromReplaySession()` - convert session to status
- `modelSummary()` - extract model calls from session

### ✅ Step 2: Wire ReplayRunner in AIProviderEvaluation
- Create adapter instance with todayVendors data
- Create replayRunner with adapter
- Replace `loadReplayStatus()` → `replayAdapter.getStatus()`
- Replace `recordReplaySession()` → `replayAdapter.recordSession()`
- Replace `buildOfflineReplaySession()` → `replayAdapter.buildOfflineSession()`
- Replace `clearReplaySession()` → `replayAdapter.clearSession()`
- Replace `resetTodayExperience()` → `replayAdapter.resetExperience()`
- Wire `replayRunner.resetCursor()` in `resetEvaluation()`
- Use derived values for `recordingReplay` and `buildingOfflineReplay` from adapter flags

### ✅ Step 3: Refactor runTodayStage
Simplified to use adapter pattern for all three modes:

```typescript
async function runTodayStage(stage: string): Promise<TodayRunResponse> {
  if (runMode === "replay") {
    // ReplayRunner manages cursor and session
    const session = await replayRunner.ensureSession()
    const recorded = await replayRunner.takeRun(session, normalizedStage)
    await replayRunner.playDelay(recorded)
    return response
  }

  // Mock or Live mode - use adapter
  const inputs = replayAdapter.formInputs(stage, runMode === "live")
  const result = await replayAdapter.runStage(stage, inputs)
  
  if (runMode === "live") {
    assertRealLlmCalls(result, "Live mode")
  }
  return result
}
```

Removed ~200 LOC of replay glue code:
- `isTauriRuntime()`
- `ensureReplaySession()`
- `takeRecordedRun()`
- `normalizeStage()` (moved to adapter)
- `replayThinkingDelay()` (handled by replayRunner)
- `inputsForStage()` (moved to adapter)
- `emptyReplayStatus()` (moved to adapter)
- `statusFromReplaySession()` (moved to adapter)
- `modelSummary()` (moved to adapter)

### ✅ Build & Styling Fixes
- Added `tailwind.config.js` to properly scan helm-flow content
- Added `.card-label` and `.btn-lime` utility classes to app.css
- Fixed Tailwind v4 @apply issues by removing scoped styles from Helm components
- Updated helm-flow package.json to export .svelte files

## Success Criteria - ALL MET ✅
✅ All three modes functional (mock/live/replay)
✅ FlowPlayer + ReplayRunner work together seamlessly
✅ No Tauri/HTTP knowledge in generic replay logic (confined to adapter)
✅ ~150 LOC of replay glue code in adapter, ~200 LOC removed from component
✅ Clean separation: adapter owns runtime/storage, ReplayRunner owns orchestration, component owns state
✅ Builds without errors
✅ Type-safe throughout (TypeScript strict mode)

## Architecture
```
AIProviderEvaluation (component)
  ├─ FlowPlayer (Helm - owns choreography)
  ├─ ReplayRunner (Helm - owns replay cursor/caching)
  │   └─ VendorSelectionReplayAdapter
  │       ├─ Tauri runtime detection
  │       ├─ HTTP fetch calls
  │       ├─ Stage normalization
  │       └─ Input formation
  └─ Domain logic (vendor-selection business logic)
```

Each layer owns one concern, fully decoupled.
