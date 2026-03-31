# Developers Handbook

This guide walks through how Converge works from the ground up. It follows the reference `evaluate-vendor` truth end-to-end: from a Gherkin spec, through Rust agents, into the convergence engine, and out the other side with a governed decision.

Read this before writing your first agent.

## Quick Reference: Justfile Commands

| Command | What it does |
|---------|-------------|
| `just hit-the-ground-running` | First time setup: build, test, lint — confirms everything works |
| `just test` | Run all tests |
| `just build` | Build the workspace |
| `just server` | Start the local HTTP harness on port 8080 |
| `just lint` | Run clippy |
| `just fmt` | Format all code |
| `just clean` | Delete all build artifacts and start fresh |
| `just install-desktop` | Install desktop frontend dependencies |
| `just dev-desktop` | Run the desktop app in dev mode |
| `just package-desktop` | Build a native desktop bundle for your platform |

## 1. The Gherkin Spec

Everything starts with a Gherkin file. This is the human-readable contract for what the system should do.

```gherkin
Feature: Enterprise AI vendor selection

  Scenario: Evaluate candidate vendors for governed rollout
    Given truth "evaluate-vendor"
    And vendors:
      | name      |
      | Acme AI   |
      | Beta ML   |
      | Gamma LLM |
```

This says: run the `evaluate-vendor` truth with three vendors as input. The Gherkin file is not decoration. It is a loadable input that the desktop app can parse and execute directly.

The equivalent JSON format:

```json
{
  "title": "Enterprise AI vendor selection",
  "truth_key": "evaluate-vendor",
  "vendors": ["Acme AI", "Beta ML", "Gamma LLM"],
  "inputs": {
    "decision_context": "Select a primary AI vendor for an enterprise rollout."
  }
}
```

Both formats produce the same execution. The Gherkin parser extracts the truth key and vendor table, builds an inputs map, and hands it to the same executor.

## 2. What Converge Gives You

These are the building blocks from `converge-core`. You don't implement them, you use them.

| Type | What it does |
|------|-------------|
| `Engine` | Runs the convergence loop. Registers agents, detects fixed point, enforces budgets. |
| `Context` | Shared state that all agents read. Partitioned by `ContextKey`. |
| `ContextKey` | Partition labels: `Seeds`, `Hypotheses`, `Evaluations`, `Corrections`, `Metadata`. |
| `Fact` | An immutable piece of evidence in the context. Has an `id`, `key`, and `content`. |
| `ProposedFact` | What agents emit. Includes `confidence` and `provenance`. Gets promoted to `Fact` by the engine. |
| `AgentEffect` | The return value of `execute()`. Contains proposed facts. |
| `TypesRootIntent` | Declares the objective, active packs, success criteria, and budgets for a run. |
| `Criterion` | A success condition. The engine checks these after convergence. |
| `CriterionResult` | `Met`, `Unmet`, `Blocked`, or `Indeterminate`. |
| `StreamingCallback` | Trait for real-time notifications during convergence (`on_cycle_start`, `on_fact`, `on_cycle_end`). |

## 3. What You Write

For each truth, you write four things:

### a. Truth definition

In `governance-truths/src/lib.rs`:

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

This is the catalog entry. It declares which agent packs participate and what criteria must be met for the truth to succeed.

### b. Agents

Each agent implements the `Agent` trait:

```rust
impl Agent for ComplianceScreenerAgent {
    fn name(&self) -> &str {
        "compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        // Empty: this agent runs first, no prerequisites
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Pure predicate. No side effects, no I/O.
        true
    }

    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        // Read the context, return proposed facts.
        // Never mutate anything. Never call another agent.
        let mut proposals = vec![];
        for name in &self.vendor_names {
            let fact_id = format!("compliance:screen:{}", slug(name));
            if ctx.get(ContextKey::Seeds).iter().any(|f| f.id == fact_id) {
                continue; // Already screened, skip
            }
            proposals.push(ProposedFact {
                key: ContextKey::Seeds,
                id: fact_id,
                content: serde_json::json!({
                    "vendor_name": name,
                    "gdpr_pass": true,
                    "ai_act_pass": true,
                }).to_string(),
                confidence: 0.85,
                provenance: "agent:compliance-screener".into(),
            });
        }
        AgentEffect { proposals, ..Default::default() }
    }
}
```

Rules for agents:

- **`dependencies()`** declares which `ContextKey` partitions this agent watches. The engine only wakes the agent when those keys change.
- **`accepts()`** is a pure predicate. No I/O, no side effects. The engine calls it to decide if the agent should run this cycle.
- **`execute()`** reads the context and returns proposals. It never mutates the context directly. The engine handles promotion.
- **Agents never call each other.** All communication happens through the shared context.
- **Check before proposing.** If a fact already exists, skip it. This is how you get idempotency.

### c. Criterion evaluator

The evaluator tells the engine whether the truth's success criteria are met:

```rust
impl CriterionEvaluator for EvaluateVendorEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "all-vendors-screened" => {
                if context.get(ContextKey::Seeds).iter()
                    .any(|f| f.id.starts_with("compliance:screen:"))
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no vendors screened yet".into(),
                    }
                }
            }
            "recommendation-produced" => {
                if context.get(ContextKey::Evaluations).iter()
                    .any(|f| f.id == "decision:recommendation")
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no recommendation produced".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}
```

### d. Executor

The executor wires everything together:

```rust
pub fn execute(store: &InMemoryStore, inputs: &HashMap<String, String>, persist: bool)
    -> Result<TruthExecutionResult, String>
{
    let truth = find_truth("evaluate-vendor").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let vendor_names = parse_vendor_input(inputs)?;

    let mut engine = Engine::new();
    engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
    engine.register_in_pack("cost-pack", CostAnalysisAgent);
    engine.register_in_pack("cost-pack", DecisionSynthesisAgent);

    let result = engine.run_with_types_intent_and_hooks(
        Context::new(),
        &intent,
        TypesRunHooks {
            criterion_evaluator: Some(Arc::new(EvaluateVendorEvaluator)),
            event_observer: None,
        },
    ).map_err(|e| format!("convergence failed: {e}"))?;

    // Optionally persist the decision to the domain store
    if persist {
        project_decision(store, &result)?;
    }

    Ok(build_result(result))
}
```

## 4. How Convergence Works

The engine runs a loop. Here is what happens step by step for the vendor evaluation:

```
Cycle 1:
  Engine checks all agents.
  ComplianceScreenerAgent has no dependencies → accepts() → true
  CostAnalysisAgent depends on Seeds → Seeds is empty → accepts() → false
  DecisionSynthesisAgent depends on Seeds + Evaluations → accepts() → false

  ComplianceScreenerAgent.execute() proposes:
    Seeds: "compliance:screen:acme-ai"
    Seeds: "compliance:screen:beta-ml"
    Seeds: "compliance:screen:gamma-llm"

  Engine promotes proposals to facts. Seeds key is now dirty.

Cycle 2:
  Seeds changed → CostAnalysisAgent wakes up.
  accepts() checks: are there compliance:screen: facts? Yes → true
  DecisionSynthesisAgent: Seeds dirty but no cost estimates yet → accepts() → false

  CostAnalysisAgent.execute() proposes:
    Evaluations: "cost:estimate:acme-ai"
    Evaluations: "cost:estimate:beta-ml"
    Evaluations: "cost:estimate:gamma-llm"

  Engine promotes. Evaluations key is now dirty.

Cycle 3:
  Evaluations changed → DecisionSynthesisAgent wakes up.
  accepts() checks: compliance facts AND cost facts exist? Yes → true

  DecisionSynthesisAgent.execute() proposes:
    Evaluations: "decision:recommendation"

  Engine promotes.

Cycle 4:
  No keys changed that wake any new agents.
  All agents either already ran or don't accept.
  Fixed point detected → convergence.

Post-convergence:
  Engine evaluates criteria:
    "all-vendors-screened" → Met (compliance facts exist)
    "recommendation-produced" → Met (decision fact exists)

Result: converged=true, cycles=4
```

The engine guarantees termination through budgets. If agents keep producing new facts without converging, `TypesBudgets::with_cycles(10)` stops the run after 10 cycles.

## 5. Fan-Out: Adding More Agents

The reference has three agents in a chain. Real governance needs fan-out: multiple agents working the same stage in parallel.

Add agents to the same pack and they all run when their dependencies are met:

```rust
engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
engine.register_in_pack("compliance-pack", DataResidencyAgent { vendor_names });
engine.register_in_pack("compliance-pack", CertificationAgent { vendor_names });
```

All three share dependencies on the same context keys. In a single cycle, all three run and propose facts into `Seeds`. The engine merges all effects deterministically (sorted by agent name).

Fan-out is the default. You get it by registering multiple agents in the same pack with the same dependencies.

## 6. Gated Decisions: Human-in-the-Loop

Converge has built-in HITL support. When a decision is too sensitive for full automation, you gate it.

### How gating works

The engine supports `HitlPolicy` which pauses convergence when proposals match a filter:

```rust
use converge_core::gates::hitl::{HitlPolicy, GateDecision, GateVerdict, TimeoutPolicy};

let hitl = HitlPolicy::gate_all()
    .with_timeout(TimeoutPolicy {
        duration_secs: 3600,
        action: TimeoutAction::Reject,
    });
```

When a gated proposal arrives, the engine pauses and emits a `GateRequest`. The request contains the proposal details, the agent that produced it, and the context at the time of the pause. Your application is responsible for presenting this to a reviewer and collecting a `GateDecision`:

```rust
GateDecision {
    verdict: GateVerdict::Approved,
    reason: "CFO reviewed and approved vendor selection".into(),
    reviewer: "jane.doe@company.com".into(),
}
```

The engine resumes with `engine.resume()`. The proposal is promoted (or rejected) based on the verdict.

You can also signal that a criterion itself needs human input. Use `CriterionResult::Blocked` in your evaluator:

```rust
CriterionResult::Blocked {
    reason: "requires procurement approval above $50k".into(),
    approval_ref: Some("PROC-2026-0412".into()),
}
```

### Where the review happens

The `GateRequest` is a data structure. Converge does not decide how it reaches a human. That is your integration layer. There are three patterns that make sense here:

**1. Desktop UI (recommended default)**

The Tauri app receives the `GateRequest` and shows a review panel inline. The operator sees the proposal, the evidence, and the agent that produced it. They approve or reject directly in the app.

This is the simplest integration. It works offline, has no external dependencies, and fits the local-first architecture. Build this first.

```
Engine pauses
  → GateRequest sent to Tauri frontend via command
  → Svelte renders review panel with proposal details
  → Operator clicks Approve / Reject
  → GateDecision sent back to engine
  → Engine resumes
```

**2. Slack**

For async review when the operator is not watching the desktop app. The engine pauses, your integration posts a structured message to a Slack channel with the proposal summary and approve/reject buttons. The reviewer clicks a button, the Slack interaction webhook fires, and your app converts it to a `GateDecision`.

```
Engine pauses
  → GateRequest serialized to Slack Block Kit message
  → Posted to channel via Slack webhook
  → Reviewer clicks Approve / Reject button
  → Slack interaction payload hits your callback endpoint
  → GateDecision sent back to engine
  → Engine resumes
```

This requires a Slack app with an incoming webhook and an interaction endpoint. Route the interaction callback through Kong if you want it governed alongside everything else.

Good for: team visibility, decisions that need a paper trail in a shared channel, reviews that happen while someone is away from the desktop app.

**3. Email**

For formal escalation and audit trails. The engine pauses, your integration sends an email with the proposal details and a link to approve or reject. The link hits an endpoint that converts the action to a `GateDecision`.

Good for: procurement approvals that need a paper trail, compliance sign-offs, anything where the reviewer is outside the team's Slack.

Slower than the other two. Use it for escalation, not for the primary review path.

### Choosing a timeout action

Every HITL gate should have a timeout policy. The two options:

- `TimeoutAction::Reject` — if nobody reviews in time, the proposal is rejected. The engine continues without it. Safe default.
- `TimeoutAction::Approve` — if nobody reviews in time, the proposal is auto-approved. Only use this for low-risk proposals where silence means consent.

### Combining gates

You can gate selectively. Not every proposal needs human review. A common pattern:

- Auto-approve compliance screening results (rule-based, low risk)
- Gate cost estimates above a threshold (financial impact)
- Always gate the final recommendation (high-stakes decision)

Filter in `HitlPolicy::requires_approval()` by checking the proposal content, the agent that produced it, or the confidence score.

## 7. External Tools and MCP

Agents often need data or actions from systems outside the app: vendor registries, compliance databases, procurement workflows, policy engines. These external tools are accessed through Kong, either as REST APIs or as MCP (Model Context Protocol) tool servers.

### The pattern

Every external tool follows the same shape:

1. Define a trait for the capability the agent needs.
2. Implement it against Kong for production.
3. Implement it as a local mock for development.
4. Inject it into the agent at construction time.

The agent never knows whether it is talking to a real service or a mock. It calls a trait method and gets data back.

### REST APIs through Kong

For traditional HTTP services (vendor databases, pricing catalogs, policy endpoints):

```rust
use converge_provider::KongGateway;

let gateway = KongGateway::from_env()?;

// Build the URL for a Kong-routed service
let policy_url = gateway.api_url("compliance/v1/policies");
let (header_name, header_value) = gateway.auth_header();
```

All API access goes through the gateway object. This means Kong logs, rate-limits, and governs the call exactly like LLM traffic.

Wrap the HTTP call in a trait so your agent stays testable:

```rust
trait PolicyService: Send + Sync {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String>;
}

struct KongPolicyService {
    gateway: KongGateway,
}

impl PolicyService for KongPolicyService {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        let url = self.gateway.api_url(&format!("compliance/v1/policies?jurisdiction={jurisdiction}"));
        // HTTP call through Kong
        // ...
    }
}

struct MockPolicyService;

impl PolicyService for MockPolicyService {
    fn get_policies(&self, _jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        Ok(vec![
            PolicyRule { id: "gdpr-1".into(), description: "Data must stay in EU".into() },
            PolicyRule { id: "ai-act-1".into(), description: "High-risk AI requires conformity assessment".into() },
        ])
    }
}
```

Then inject it:

```rust
struct ComplianceScreenerAgent {
    policies: Arc<dyn PolicyService>,
}
```

### MCP tools through Kong

MCP (Model Context Protocol) lets agents call external tools using a structured tool-call interface instead of raw HTTP. This is useful when the tool server exposes multiple actions and the agent needs to discover and invoke them dynamically.

Kong can front MCP servers the same way it fronts REST APIs. The agent talks to Kong, Kong routes to the MCP server.

```rust
use converge_provider::{KongGateway, McpClient, McpTransport};

let gateway = KongGateway::from_env()?;
let mcp = McpClient::new(
    "vendor-registry",
    McpTransport::Http {
        url: gateway.mcp_url("vendor-registry"),
    },
);

// Discover available tools
let tools = mcp.list_tools()?;

// Call a specific tool
let result = mcp.call_tool("lookup_vendor", serde_json::json!({
    "vendor_name": "Acme AI",
    "fields": ["certifications", "regions", "pricing"]
}))?;
```

When to use MCP vs REST:

- **REST** when you know the exact endpoint and payload shape at compile time. Simpler, faster, easier to type.
- **MCP** when the agent needs to discover tools dynamically, when the tool server exposes many actions, or when you want the LLM to select which tool to call based on context.

Both go through Kong. Both get the same governance, logging, and rate limiting.

### Mocking external tools

During the hackathon, real enterprise services will not be available. Do not hardcode data into agents to work around this. Instead, build a mock that implements the same trait:

1. Define the trait (e.g., `VendorService`, `PolicyService`, `ProcurementService`).
2. Build a local mock with realistic test data.
3. Use the mock in development, swap in the Kong-backed implementation when the real service is available.
4. If the mock is useful to other teams, expose it through Kong so they can call it as an MCP tool or REST endpoint.

Good candidates for mocks:

- Vendor profile service (certifications, regions, pricing plans)
- Policy engine (internal guardrails, jurisdiction rules)
- Procurement approval service (budget thresholds, escalation rules)
- Compliance evidence store (structured documents for screening)

The goal is that swapping from mock to real is a one-line change at the injection site, not a rewrite of agent logic.

## 8. Giving Agents LLM Access

The reference agents use hardcoded data. Real agents need LLM reasoning.

Use `converge-provider` with the Kong gateway. The provider implements the LLM interface that agents consume:

```rust
use converge_provider::kong::{KongGateway, KongRoute};

let gateway = KongGateway::from_env();
let route = KongRoute::builder()
    .route("default")
    .upstream_provider("openai")
    .upstream_model("gpt-4")
    .build();

let provider = gateway.llm_provider(route);
```

Pass the provider into your agent at construction time. The agent calls it inside `execute()`:

```rust
struct SmartComplianceAgent {
    provider: Arc<dyn LlmProvider>,
}

impl Agent for SmartComplianceAgent {
    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let prompt = format!("Evaluate GDPR compliance for vendor: {}", vendor_name);
        let response = self.provider.complete(&prompt);
        // Parse the response into a ProposedFact
        // ...
    }
}
```

All LLM traffic goes through Kong. This gives you rate limiting, cost tracking, guardrails, and audit logs without writing any of that yourself.

## 9. Streaming: Watching Convergence in Real Time

Implement `StreamingCallback` to get notified as the engine runs:

```rust
struct UiCallback;

impl StreamingCallback for UiCallback {
    fn on_cycle_start(&self, cycle: u32) {
        println!("--- Cycle {cycle} ---");
    }

    fn on_fact(&self, cycle: u32, fact: &Fact) {
        println!("  [cycle {cycle}] new fact: {}", fact.id);
    }

    fn on_cycle_end(&self, cycle: u32, facts_added: usize) {
        println!("  cycle {cycle} complete: {facts_added} facts added");
    }
}
```

This is how you build a live dashboard in the Tauri desktop app. The callback fires as the engine runs, not after.

## 10. Context Keys: Organizing Evidence

Facts are partitioned by `ContextKey`. Use them to organize the flow:

| Key | Purpose | Example |
|-----|---------|---------|
| `Seeds` | Initial evidence, screening results | `compliance:screen:acme-ai` |
| `Hypotheses` | Tentative conclusions, intermediate analysis | `hypothesis:acme-best-fit` |
| `Evaluations` | Scored assessments, cost estimates, decisions | `cost:estimate:acme-ai`, `decision:recommendation` |
| `Corrections` | Revisions to earlier facts | `correction:cost:acme-ai` |
| `Metadata` | Run metadata, configuration | `meta:run-config` |

Agents declare which keys they depend on. The engine only wakes agents when their dependencies change. This is how convergence stays efficient: agents don't re-run unless there's new information.

## 11. Experience and Recall

Converge tracks experience events during runs. The `ExperienceEventObserver` receives these in real time:

```rust
let observer = Arc::new(|event: &ExperienceEvent| {
    log::info!("experience: {:?}", event);
});

TypesRunHooks {
    criterion_evaluator: Some(Arc::new(evaluator)),
    event_observer: Some(observer),
}
```

The recall system (`converge_core::recall`) lets agents query past experience:

- `RecallQuery` searches for relevant past decisions
- `RecallCandidate` scores results by relevance
- `RecallPolicy` controls what can be recalled and by whom

This is how agents improve over time. A cost analysis agent can recall what similar vendors cost in past evaluations and use that as a baseline.

## 12. Governed Artifacts

`converge_core::governed_artifact` provides lifecycle management for anything the system produces. States include:

- `Draft` → `UnderReview` → `Approved` → `Active`
- `Active` → `Suspended` → `Retired`
- Any state → `RolledBack` (with severity and impact tracking)

Use this when agent outputs become operational artifacts: approved vendor lists, policy documents, compliance certificates. The state machine enforces valid transitions and tracks who changed what.

## 13. Using Domain Packs from converge-domain

Not every agent needs to be written from scratch. The `converge-domain` crate provides pre-built packs with production-grade agents and invariants. These are ready to register and run.

### Available packs

| Pack | What it does | Key agents |
|------|-------------|------------|
| `trust` | Audit trails, access control, provenance, compliance scanning | `SessionValidatorAgent`, `RbacEnforcerAgent`, `AuditWriterAgent`, `ProvenanceTrackerAgent`, `ComplianceScannerAgent`, `ViolationRemediatorAgent`, `PiiRedactorAgent` |
| `money` | Financial transactions, invoicing, reconciliation | `InvoiceCreatorAgent`, `PaymentAllocatorAgent`, `ReconciliationMatcherAgent`, `PeriodCloserAgent` |
| `delivery` | Promise fulfillment, scope tracking, blockers | `PromiseCreatorAgent`, `WorkBreakdownAgent`, `BlockerDetectorAgent`, `RiskAssessorAgent`, `StatusAggregatorAgent` |
| `knowledge` | Signal capture, hypothesis testing, experiments, canonical decisions | `SignalCaptureAgent`, `HypothesisGeneratorAgent`, `ExperimentRunnerAgent`, `DecisionMemoAgent`, `CanonicalKnowledgeAgent` |
| `data_metrics` | Metrics, dashboards, anomaly detection, alerting | `MetricRegistrarAgent`, `DataValidatorAgent`, `DashboardBuilderAgent`, `AnomalyDetectorAgent`, `AlertEvaluatorAgent` |

### Example: the audit-vendor-decision truth

This repo includes a second truth that uses the trust pack with zero custom agents. It demonstrates how domain packs work out of the box.

The Gherkin spec:

```gherkin
Feature: Audit vendor decision

  Scenario: Produce an audit trail for a vendor evaluation
    Given truth "audit-vendor-decision"
    And decision_id "eval-001"
```

The executor registers five trust pack agents and seeds the context with a session token:

```rust
use converge_domain::packs::trust::{
    AuditWriterAgent, ComplianceScannerAgent, ProvenanceTrackerAgent,
    RbacEnforcerAgent, SessionValidatorAgent,
};

let mut engine = Engine::new();
engine.register_in_pack("trust-pack", SessionValidatorAgent);
engine.register_in_pack("trust-pack", RbacEnforcerAgent);
engine.register_in_pack("trust-pack", AuditWriterAgent);
engine.register_in_pack("trust-pack", ProvenanceTrackerAgent);
engine.register_in_pack("trust-pack", ComplianceScannerAgent);
```

The convergence chain:

```
Cycle 1: SessionValidatorAgent validates the session token → emits validated session
Cycle 2: RbacEnforcerAgent sees valid session → emits access decision
Cycle 3: AuditWriterAgent sees access decision → emits audit entry
         ProvenanceTrackerAgent sees audit entry → emits provenance record
Cycle 4: ComplianceScannerAgent sees audit entries → emits compliance scan
Cycle 5: No changes → converged
```

No custom agents, no glue code beyond the executor. The pack agents communicate through the shared context using fact ID prefixes (`session:`, `access_decision:`, `audit:`, `provenance:`, `compliance:`).

### Mixing domain packs with custom agents

The real power is combining packs. Register trust pack agents alongside your custom governance agents in the same engine:

```rust
// Custom agents for vendor evaluation
engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
engine.register_in_pack("cost-pack", CostAnalysisAgent);
engine.register_in_pack("cost-pack", DecisionSynthesisAgent);

// Domain pack agents for audit and provenance
engine.register_in_pack("trust-pack", AuditWriterAgent);
engine.register_in_pack("trust-pack", ProvenanceTrackerAgent);
engine.register_in_pack("trust-pack", ComplianceScannerAgent);
```

The trust pack agents will automatically pick up the access decisions and audit the entire evaluation flow. You get audit trails and compliance scanning for free.

### Invariants from domain packs

Domain packs also provide invariants. These are rules the engine checks during convergence:

```rust
use converge_domain::packs::trust::{
    AllActionsAuditedInvariant,      // Every access decision must have an audit entry
    AuditImmutabilityInvariant,      // Audit entries must be marked immutable
    ViolationsHaveRemediationInvariant, // Open violations must have remediation plans
};
```

Register invariants on the engine to enforce them during convergence. If an invariant is violated, the engine stops and reports the violation instead of silently converging on incomplete evidence.

### When to use domain packs vs custom agents

- **Use domain packs** for cross-cutting concerns: audit, access control, provenance, compliance scanning. These are solved problems with well-defined contracts.
- **Write custom agents** for your business logic: vendor scoring, cost analysis, risk assessment, decision synthesis. These encode domain knowledge specific to your use case.
- **Combine both** in the same engine. Domain packs handle the governance substrate, custom agents handle the decision logic.

## 15. Adding a New Truth: Step by Step

1. **Define it** in `governance-truths/src/lib.rs`:
   ```rust
   TruthDef {
       key: "assess-risk",
       display_name: "Assess Vendor Risk",
       summary: "Multi-agent risk assessment across operational and strategic dimensions",
       packs: &["risk-pack"],
       criteria: &[
           ("risk-scores-complete", "All vendors have risk scores"),
       ],
   }
   ```

2. **Create the executor** at `governance-server/src/truth_runtime/assess_risk.rs`

3. **Write your agents** in that file (or split into modules)

4. **Write your criterion evaluator**

5. **Wire it** in `truth_runtime/mod.rs`:
   ```rust
   "assess-risk" => assess_risk::execute(store, inputs, persist),
   ```

6. **Add domain types** to `governance-kernel` if needed

7. **Test it:**
   ```bash
   just test
   ```

## 16. The Full Picture

```
vendor-selection.feature        (human-readable spec)
        │
        ▼
  Source parser                  (Gherkin → inputs map)
        │
        ▼
  TruthDef + build_intent()     (declare packs, criteria, budgets)
        │
        ▼
  Engine                         (register agents in packs)
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │  Convergence Loop                            │
  │                                              │
  │  Cycle 1: Screening agents run               │
  │    → propose facts into Seeds                │
  │                                              │
  │  Cycle 2: Analysis agents wake up            │
  │    → propose facts into Evaluations          │
  │                                              │
  │  Cycle 3: Synthesis agent wakes up           │
  │    → proposes recommendation                 │
  │                                              │
  │  Cycle 4: No changes → fixed point           │
  └─────────────────────────────────────────────┘
        │
        ▼
  Criteria evaluation            (Met / Unmet / Blocked)
        │
        ▼
  Domain projection              (persist decision to kernel)
        │
        ▼
  Audit trail                    (who decided what, when, why)
```

Every fact has provenance. Every decision has evidence. Every run has a stop reason. That is the governance story.
