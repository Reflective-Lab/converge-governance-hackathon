---
tags: [domain, truths]
---
# Truths

A truth is a governed job: it declares which agent packs participate and what criteria must be met.

## Truth Catalog

Defined in `governance-truths/src/lib.rs`.

### evaluate-vendor
The primary truth. Multi-agent vendor evaluation: compliance, risk, cost, decision.

- **Packs:** compliance-pack, risk-pack, cost-pack
- **Criteria:** all-vendors-screened, recommendation-produced

### audit-vendor-decision
Uses the [[Converge/Domain Packs|trust pack]] with zero custom agents. Demonstrates domain packs out of the box.

- **Packs:** trust-pack
- **Agents:** SessionValidatorAgent, RbacEnforcerAgent, AuditWriterAgent, ProvenanceTrackerAgent, ComplianceScannerAgent

## Truth Definition Shape

```rust
TruthDef {
    key: "evaluate-vendor",
    display_name: "Evaluate AI Vendor",
    summary: "Multi-agent vendor evaluation: compliance, risk, cost, decision",
    packs: &["compliance-pack", "risk-pack", "cost-pack"],
    criteria: &[
        ("all-vendors-screened", "All vendors have compliance screening facts"),
        ("recommendation-produced", "A decision recommendation fact exists"),
    ],
}
```

## Truth Execution Pattern

```rust
fn execute(store, inputs, persist) -> Result<TruthExecutionResult> {
    let truth = find_truth("your-truth-key")?;
    let intent = build_intent(truth);

    let mut engine = Engine::new();
    engine.register_in_pack("your-pack", YourAgent { ... });

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
