---
tags: [converge, organism, patterns]
source: mixed
---
# Organism Patterns

Organism is the intelligence layer above Converge. Where Converge handles promotion gates, convergence, and policy — Organism handles *how agents think together*.

Every pattern here is available via `organism-pack` and `organism-runtime`. Import, compose, extend.

## The Six-Stage Pipeline

Every intent flows through a mandatory pipeline. No shortcuts, no "trusted plan" exceptions.

```
Intent → Admission → Planning → Adversarial → Simulation → Learning → Converge
```

Each stage is pluggable — you control the logic at every point.

## 1. Intent — The Entry Point

An `IntentPacket` is the contract between a human and the runtime. It declares the outcome, not the steps.

```rust
use organism_pack::IntentPacket;

let intent = IntentPacket::new("Evaluate AI vendors for procurement", expires)
    .with_context(json!({
        "vendors": ["Acme AI", "Beta ML"],
        "goal": "screen, compare, and recommend with explicit evidence"
    }))
    .with_authority(vec!["vendor_evaluation".into()])
    .with_reversibility(Reversibility::Partial);
```

**Admission control** runs first — a cheap 4-dimensional feasibility gate (capability, context, resources, authority) that rejects obviously infeasible intents before wasting compute.

**Intent decomposition** breaks complex intents into an `IntentNode` tree. Critical constraint: authority can only narrow during decomposition — a subtask never has more authority than its parent.

## 2. Planning — Collaborative Reasoning

Multiple reasoning systems propose plans **in parallel**. Failures are dropped; survivors proceed to debate.

```rust
use organism_pack::{Reasoner, ReasoningSystem, Plan};

impl Reasoner for ComplianceReasoner {
    fn name(&self) -> &str { "compliance-reasoning" }
    fn system_type(&self) -> ReasoningSystem { ReasoningSystem::DomainModel }
    async fn propose(&self, intent: &IntentPacket) -> Result<Plan> {
        // Your domain logic produces a plan
    }
}
```

### Collaboration Topologies

Four ways agents can work together, each encoding different team discipline:

| Topology | Discipline | When to use |
|---|---|---|
| **Huddle** | Enforced — round-robin, mandatory dissent map, done-gate vote | High-stakes decisions needing dissent tracking |
| **Discussion Group** | Moderated — moderator frames, then steps back | Strategy brainstorms, advisory output |
| **Panel** | Curated — judges vote but don't contribute, report writers write but don't vote | Expert evaluation with formal output |
| **Self-Organizing** | Loose — agents coordinate themselves | Exploratory phases with minimal constraints |

```rust
use organism_pack::CollaborationCharter;

let charter = CollaborationCharter::huddle();           // strict
let charter = CollaborationCharter::discussion_group(); // advisory
let charter = CollaborationCharter::panel();            // curated expert
let charter = CollaborationCharter::self_organizing();  // loose
```

**Topology transitions** happen dynamically. Swarm → Huddle when evidence clusters. Huddle → Panel when contradictions spike. Panel → Synthesis when stable. The system discovers collaboration patterns that no human would design.

### Charter Derivation

Charters can be derived automatically from intent properties:

```rust
let derived = derive_charter(&intent, Utc::now());
// Every charter field is justified via multi-signal derivation
println!("{}", derived.rationale); // explains why each field was chosen
```

## 3. Adversarial Review — Institutionalized Disagreement

Five kinds of skepticism challenge every plan before it reaches simulation:

| Skeptic | Question |
|---|---|
| **Assumption Breaking** | What are the unstated assumptions? |
| **Constraint Checking** | Do the declared constraints actually hold? |
| **Causal Skepticism** | What are the second-order effects? |
| **Economic Skepticism** | What does this really cost? |
| **Operational Skepticism** | Can the organization actually execute this? |

```rust
use organism_pack::{Skeptic, Challenge, SkepticismKind, Severity};

impl Skeptic for CostSkeptic {
    async fn review(&self, plan: &Plan) -> Vec<Challenge> {
        // Challenge cost assumptions with evidence
        vec![Challenge {
            kind: SkepticismKind::EconomicSkepticism,
            severity: Severity::Warning,
            description: "Monthly cost estimate assumes 2024 pricing".into(),
            evidence: vec!["vendor announced 30% price increase for 2026".into()],
            suggestion: Some("Re-estimate with 2026 pricing sheet".into()),
        }]
    }
}
```

**Blocker** findings stop the plan. Plans revise and adversaries challenge again. Converge's fixed-point detection handles convergence naturally — no special machinery needed.

## 4. Simulation Swarm — Parallel Stress Testing

Five dimensions tested in parallel. Each returns probability distributions, not point estimates.

| Dimension | What it tests |
|---|---|
| **Outcome** | Does the plan achieve the intent? |
| **Cost** | Resource consumption within bounds? |
| **Policy** | Any policy violations? |
| **Causal** | Second-order effects managed? |
| **Operational** | Can the team and systems execute? |

Results are `Proceed`, `ProceedWithCaution`, or `DoNotProceed`.

## 5. Learning — Calibrating Priors

Execution outcomes calibrate planning priors. The cycle:

```
Intent → Plan → Execute → Observe → Learn → Calibrate priors → (next intent benefits)
```

```rust
use organism_pack::{LearningEpisode, PriorCalibration};

let episode = build_episode(intent_id, plan_id, subject, &context);
let calibrations = calibrate_priors(&[episode]);
// calibrations feed back into planning — never into authority
```

Critical: learning signals feed **backward** into planning priors, never directly into authority. The system gets better at planning, not at bypassing governance.

## 6. Domain Packs — Pre-Built Workflows

15 organizational workflow packs, ready to compose:

| Pack | Lifecycle |
|---|---|
| `knowledge` | Signal → Hypothesis → Experiment → Decision → Canonical |
| `customers` | Lead → Enrich → Score → Route → Propose → Close |
| `people` | Hire → Identity → Access → Onboard → Pay → Offboard |
| `legal` | Contract → Review → Sign → Execute |
| `autonomous_org` | Policy → Enforce → Approve → Budget → Delegate |
| `procurement` | Request → Approve → Order → Asset → Renewal |
| `due_diligence` | Research → Extract → Detect Gaps → Synthesize |
| `product_engineering` | Roadmap → Feature → Task → Release → Incident |
| `ops_support` | Ticket → Triage → Route → SLA → Escalate → Resolve |
| `growth_marketing` | Campaign → Channel → Budget → Experiment → Attribution |

### Blueprints — Composing Packs

| Blueprint | Packs | Use Case |
|---|---|---|
| `lead_to_cash` | Customers → Legal → Money | Full sales cycle |
| `hire_to_retire` | Legal → People → Money | Employee lifecycle |
| `procure_to_pay` | Procurement → Legal → Money | Vendor management |
| `diligence_to_decision` | Due Diligence → Legal → Knowledge | M&A / research |
| `idea_to_launch` | Product Engineering → Delivery | Feature development |

## Intent Resolution — How Intent Finds Its Packs

| Level | How | Confidence |
|---|---|---|
| **Declarative** | App explicitly declares requirements | 1.0 |
| **Structural** | Match fact prefixes to packs (deterministic) | 0.85 |
| **Semantic** | Huddle matches intent to pack descriptions (LLM) | 0.5–0.9 |
| **Learned** | Prior calibration from execution history | Compounds |

The flywheel: more intents → more episodes → better Level 4 → fewer manual bindings → faster resolution.

## Quick Start for Hackathon

1. **Start with `organism-pack`** — one import, full pipeline semantics
2. **Read `evaluate_vendor.rs`** — it already uses `IntentPacket`, `Plan`, and `Registry`
3. **Pick a topology** — `CollaborationCharter::huddle()` for high-stakes, `panel()` for curated experts
4. **Implement `Reasoner` + `Skeptic`** — these are your domain logic
5. **Wire to Converge** — implement `Suggestor`, register with `Engine`, run

See also: [[Building Blocks]], [[Domain Packs]], [[Organism Blueprints]], [[../Development/Writing Suggestors]]
