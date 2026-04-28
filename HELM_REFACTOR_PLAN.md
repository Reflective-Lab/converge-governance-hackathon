# Helm-Flow Refactoring Plan for AIProviderEvaluation.svelte

## Scope
Refactor 1110-line component to use helm-flow for orchestration while keeping vendor-selection business logic local.

## Commits

### Commit 1: FlowPlayer Integration
- Create FlowPlayer instance with pipeline phases
- Refactor startEvaluation() to use FlowPlayer.scheduleSteps()
- Replace runState with FlowPlayer.getState().runState
- Remove: pushStep(), completeThrough(), scheduleAnalysisSteps(), clearRunTimers(), startSpinner(), stopSpinner()
- Result: ~200 LOC removed from script

### Commit 2: Component Swap - HitlGate
- Replace HITL form markup (lines ~1568-1618) with <HitlGate /> component
- Wire approverName, approvalNote, delegateToPolicy bindings
- Wire policyPreview and onApprove
- Result: ~50 LOC removed from template

### Commit 3: Component Swap - DocumentIntake
- Replace document dropbox markup (lines ~1177-1252) with <DocumentIntake /> component
- Wire documents, fastLoadEnabled, executableReady bindings
- Wire expectedDocs and onFilesSelected
- Result: ~75 LOC removed from template

### Commit 4: Cleanup
- Remove unused local state (steps, timers, spinnerInterval, spinnerVerb)
- Remove helper methods that FlowPlayer owns (clearRunTimers, etc.)
- Verify all three modes (mock, live, replay) work
- Final result: ~300 LOC removed, ~750 LOC kept

## Success Criteria
- ✅ FlowPlayer drives choreography
- ✅ HitlGate and DocumentIntake components in use
- ✅ Vendor-selection logic intact
- ✅ All modes work end-to-end
- ✅ ~40% size reduction
- ✅ `just lint` clean
