# Helm Flow Extraction Plan

## Goal
Extract presentation/workflow orchestration from `AIProviderEvaluation.svelte` into reusable Helm primitives.

**Not a line-count goal.** Goal is: flow state, pacing, HITL form, and document intake are no longer vendor-selection-specific.

---

## Phase 1: Headless Flow Runtime + Adapters

### Step 1: Create helm-flow Package

**Location:** `~/dev/work/helms/packages/helm-flow/`

**Structure:**
```
packages/helm-flow/
  package.json
  tsconfig.json
  src/
    types.ts
    player.ts
    replay.ts
    HitlGate.svelte
    DocumentIntake.svelte
    index.ts
```

---

## Extraction Details

### `types.ts`

**Symbols moving to Helm:**
- `RunState` (enum: bootstrap | running | gate-review | hitl | finished)
- `RunMode` (enum: mock | live | replay)
- `FlowStep` (interface: id, label, detail, agent, purpose)
- `FlowPhase` (interface: name, steps[], gateName?, gateReason?)
- `DemoSession` (interface: schema_version, recorded_at, source_hash, mode, runs[])
- `DemoRun` (interface: stage, result, compressed_delay_ms, original_elapsed_ms)
- `ReplayStatus` (interface: available, run_count, mode, recorded_at, source_hash, source_matches, error)
- `FlowState` (interface: runState, activeStepIndex, activeLabelText, progressPercent, currentPhase)

**Symbols staying in hackathon:**
- All vendor-selection-specific types (VendorInput, TodayRunResponse, ExperienceSnapshot, etc.)
- All domain result shapes (criteria_outcomes, projection details, etc.)

---

### `player.ts`

**Class moving to Helm:**
- `FlowPlayer`
  - Constructor(config: FlowPlayerConfig)
  - Methods:
    - `getState(): FlowState`
    - `activeStep(): FlowStep | null`
    - `currentPhase(): FlowPhase | undefined`
    - `totalSteps(): number`
    - `start(): void`
    - `scheduleSteps(stepCount, onStepChange): void`
    - `pauseAtGate(gateName, onGate): void`
    - `approveGate(): void`
    - `finish(): void`
    - `reset(): void`

**Config interface:**
```ts
export interface FlowPlayerConfig {
  phases: FlowPhase[]
  stepDelayMs?: number     // Default 1650ms
  reviewPauseMs?: number   // Default 1100ms
}
```

**Symbols staying in hackathon:**
- All flow logic specific to vendor-selection pipeline (pipelineSteps array, step details)
- State management for domain-specific results (analysisRun, approvedRun, learningRuns, etc.)

---

### `replay.ts`

**Class moving to Helm:**
- `ReplayRunner`
  - Constructor(adapter: ReplayAdapter)
  - Methods:
    - `ensureSession(): Promise<DemoSession>`
    - `recordSession(): Promise<DemoSession>`
    - `takeRun(session, stage): Promise<DemoRun>`
    - `playDelay(recorded): Promise<void>`

**Adapter interface (domain provides implementation):**
```ts
export interface ReplayAdapter {
  loadSession(): Promise<DemoSession>
  saveSession(session: DemoSession): Promise<void>
  runStage(stage: string, inputs: Record<string, string>): Promise<unknown>
  recordingInProgress?: boolean
  buildingOfflineBackup?: boolean
}
```

**What Helm does NOT know:**
- Tauri invocation commands
- Filesystem paths
- Provider selection or live execution
- Stage normalization (that's domain-specific)

**Symbols staying in hackathon:**
- Tauri-specific recording (invokeTauri("record_today_replay_session"))
- HTTP fetch logic for running stages (fetch to localhost:8080)
- Stage normalization (mapping "before-hitl" → "analysis")
- DemoSession serialization/caching
- Provider-specific error handling

---

### `HitlGate.svelte`

**Component moving to Helm:**
- Generic HITL approval form
- Props: decisionSummary, approverName, approvalNote, delegateToPolicy, policyPreview, onApprove, disabled
- No domain assumptions about what the decision is
- No Cedar-specific knowledge

**Symbols staying in hackathon:**
- Cedar policy template (cedarPreview string literal)
- Cedar-specific naming and semantics
- Vendor-specific decision summary content

---

### `DocumentIntake.svelte`

**Component moving to Helm:**
- Document dropbox, file upload, fast-load toggle
- Readiness indicators (document count, executable ready)
- Expected docs modal (accepts expectedDocs array as prop)
- Props: documents, fastLoadEnabled, executableReady, expectedDocs, onFilesSelected

**Symbols staying in hackathon:**
- sampleDocs array (vendor-selection-specific sample package)
- expectedDocs content (RFI/RFP, workload profile, compliance, pricing, platform context)
- inferKind() heuristics (vendor-selection-specific file categorization)

---

## Refactoring hackathon/AIProviderEvaluation.svelte

### Step 2: Import helm-flow/types

**Changes:**
```ts
// Before:
type RunState = 'bootstrap' | 'running' | 'gate-review' | 'hitl' | 'finished'
type RunMode = 'mock' | 'live' | 'replay'
interface FlowStep { ... }
interface FlowPhase { ... }
interface DemoSession { ... }
interface ReplayStatus { ... }

// After:
import { 
  RunState, RunMode, FlowStep, FlowPhase, DemoSession, 
  ReplayStatus, FlowState 
} from 'helm-flow/types'
```

**Local types that remain:**
- VendorInput, TodayRunResponse, ExperienceSnapshot, TruthResult
- All projection/policy/recommendation shapes
- EvaluationMode, BootstrapMode (specific to this demo)

---

### Step 3: Import helm-flow/player

**Changes:**
```ts
// Before:
let runState = $state<RunState>("bootstrap")
let activeStepIndex = $state(0)
let timers: ReturnType<typeof setTimeout>[] = []

function pushStep(index: number) { ... }
function completeThrough(index: number) { ... }
function scheduleAnalysisSteps() { ... }
function startSpinner() { ... }
function stopSpinner() { ... }
function clearRunTimers() { ... }

// After:
import { FlowPlayer, FlowPlayerConfig } from 'helm-flow/player'

const flowPlayer = new FlowPlayer({
  phases: [
    { name: 'Analysis', steps: pipelineSteps.slice(0, 6) },
    { name: 'HITL Gate', steps: [pipelineSteps[6]], gateName: 'hitl' },
    { name: 'Promotion', steps: pipelineSteps.slice(7) },
  ],
  stepDelayMs: 1650,
  reviewPauseMs: 1100,
})

// In reactive code:
let flowState = $derived(flowPlayer.getState())
```

**Spinner management stays local** because it's demo-specific (randomVerb, changing text).

---

### Step 4: Import helm-flow/HitlGate

**Changes:**
```svelte
{#if runState === 'hitl'}
  <HitlGate
    decisionSummary={{ 
      candidate: stringAt(recommendationFor(analysisRun), 'recommendation'),
      reason: stringAt(beforeHitlPolicy, 'reason'),
      threshold: money(numberAt(beforeHitlPolicy, 'hitl_threshold_major')),
    }}
    bind:approverName
    bind:approvalNote
    bind:delegateToPolicy
    policyPreview={cedarPreview}
    onApprove={approveHitl}
  />
{/if}
```

**Local HITL logic that remains:**
- Cedar policy preview generation (cedarPreview computed from delegateToCedar)
- Cedar-specific delegation semantics
- Vendor-selection-specific decision content

---

### Step 5: Import helm-flow/DocumentIntake

**Changes:**
```svelte
<DocumentIntake
  bind:documents
  bind:fastLoadEnabled={bootstrapMode === 'sample'}
  bind:executableReady
  expectedDocs={expectedDocs}
  onFilesSelected={(files) => handleFiles(files)}
/>
```

**Local document logic that remains:**
- sampleDocs (vendor-selection sample data)
- expectedDocs content (specific to RFI/RFP/vendor evaluation)
- inferKind() function (vendor-specific file categorization)
- useSampleProcess() (sets up demo-specific state)

---

## Phase 2 (Not This PR)

Defer until Phase 1 is merged and proven:
- ReplayRunner full integration (currently adapter-based only)
- Tauri recording commands
- Result rendering primitives (TimelinePanel, DecisionCard, EvidenceList, OutcomeBadge)
- Experience store UI

---

## Implementation Order

1. **Create helm-flow package structure** (package.json, tsconfig.json, src/)
2. **Move types.ts** → types + FlowState + enums
3. **Refactor hackathon to import types** → verify no breakage
4. **Move player.ts** → FlowPlayer class
5. **Refactor hackathon to use FlowPlayer** → connect state to UI
6. **Move HitlGate.svelte** → import into hackathon
7. **Move DocumentIntake.svelte** → import into hackathon
8. **Move replay.ts** → adapter pattern only, not wired to hackathon yet
9. **Create index.ts** → export all public APIs
10. **Update hackathon imports** → single import from @reflective/helm-flow
11. **Test full flow** → mock, live, replay modes all work

Each step is a reviewable commit. Do not batch steps 1–11 into one PR.

---

## Files Modified

| File | Change | Lines ± |
|------|--------|---------|
| helms/packages/helm-flow/src/types.ts | Create | +150 |
| helms/packages/helm-flow/src/player.ts | Create | +200 |
| helms/packages/helm-flow/src/replay.ts | Create | +100 |
| helms/packages/helm-flow/src/HitlGate.svelte | Create | +120 |
| helms/packages/helm-flow/src/DocumentIntake.svelte | Create | +180 |
| helms/packages/helm-flow/src/index.ts | Create | +10 |
| helms/packages/helm-flow/package.json | Create | +30 |
| helms/packages/helm-flow/tsconfig.json | Create | +20 |
| hackathon/apps/desktop/src/lib/AIProviderEvaluation.svelte | Refactor | ~−300 |
| hackathon/apps/desktop/src/lib/types.ts (new) | Create | +150 |
| hackathon/apps/desktop/package.json | Update | +1 (helm-flow dep) |

**Net result:**
- helm-flow: ~750 LOC (new, reusable)
- hackathon AIProviderEvaluation: ~810 LOC (down from ~1110)
- hackathon types: ~150 LOC (new, domain-specific)

---

## Success Criteria

✅ helm-flow types, player, replay, HitlGate, DocumentIntake are in ~/dev/work/helms/packages/helm-flow/
✅ hackathon AIProviderEvaluation imports from @reflective/helm-flow
✅ Flow state machine drives entire demo without vendor-specific logic
✅ HITL gate is generic and reusable
✅ Document intake is generic and reusable
✅ Replay adapter pattern works (ready for Phase 2)
✅ All three run modes (mock, live, replay) still work end-to-end
✅ No broken tests
✅ `just lint` clean in both repos
