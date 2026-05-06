---
tags: [domain, truths]
---
# Truths

A truth is an Axiom contract: it declares what a governed decision must prove before the app treats the outcome as admissible.

This product has one product truth: `vendor-selection`.

Supporting truth runtimes may remain while the repo is being migrated from hackathon scaffolding, but they are examples, fixtures, or historical references. They should not become separate product workflows.

## Product Truth

Defined in `governance-truths/src/lib.rs`.

### vendor-selection
The canonical truth. Multi-agent vendor evaluation with intake, evidence, scoring, policy gates, audit output, and final recommendation.

- **Packs:** compliance-pack, risk-pack, cost-pack
- **Criteria:** all vendors evaluated, policy gates satisfied, recommendation produced, audit evidence available

## Supporting Runtimes

These are not product truths. Keep them only if they support tests, migration, demos, or implementation references.

### evaluate-vendor
Reference vendor evaluation runtime.

### dynamic-due-diligence
Advanced truth inspired by Monterro's dynamic research loop. Organism seeds typed breadth and depth strategies, Converge governs research signals and extracted hypotheses, contradictions are promoted explicitly, and a final due-diligence brief is synthesized as structured output.

- **Packs:** planning-pack, research-pack, analysis-pack, synthesis-pack
- **Inputs:** company, optional focus_areas
- **Criteria:** critical-evidence-collected, final-brief-produced
- **Projection:** writes a `DecisionRecord` and returns the final brief in `projection.details`

### audit-vendor-decision
Uses the [[Converge/Domain Packs|trust pack]] with zero custom agents. Demonstrates domain packs out of the box.

- **Packs:** trust-pack
- **Agents:** SessionValidatorAgent, RbacEnforcerAgent, AuditWriterAgent, ProvenanceTrackerAgent, ComplianceScannerAgent

### authorize-vendor-commitment
Uses `converge-policy` as a pure library inside the governance runtime. This is
the business authorization step after a recommendation exists and before a team
commits budget or contract.

- **Packs:** policy-pack
- **Inputs:** principal identity and authority, commitment id, vendor, amount, gates, human approval
- **Outcomes:** authorize, escalate for human approval, or reject
- **Projection:** writes a `DecisionRecord` so the audit trail captures the policy outcome

## Truth Definition Shape

```rust
TruthDef {
    key: "vendor-selection",
    display_name: "Vendor Selection",
    summary: "Governed vendor selection with evidence, policy gates, and audit output",
    packs: &["compliance-pack", "risk-pack", "cost-pack"],
    criteria: &[
        ("all-vendors-evaluated", "All vendors have evaluation facts"),
        ("recommendation-produced", "A decision recommendation fact exists"),
    ],
}
```

The dynamic due-diligence example is a good reference when you need a loop that is more open-ended than vendor scoring:

1. Organism seeds typed strategies
2. Research suggestors emit source-aware signals
3. Analysis suggestors extract hypotheses and contradictions
4. Gap suggestors propose follow-up strategies
5. Synthesis emits the final governed brief

## Truth Execution Pattern

```rust
fn execute(store, inputs, persist) -> Result<TruthExecutionResult> {
    let truth = find_truth("vendor-selection")?;
    let intent = build_intent(truth);

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack("your-pack", YourAgent { ... });

    let result = engine.run_with_types_intent_and_hooks(
        context, &intent, hooks
    )?;

    if persist {
        store.write_with_events(|kernel| { /* project facts */ })?;
    }

    Ok(result)
}
```

See also: [[Development/Writing Truths]], [[Domain/Key Types]], [[Architecture/Convergence Loop]]
