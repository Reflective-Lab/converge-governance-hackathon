---
tags: [presentation, demo, vendor-selection, helm]
---
# Team Pitch - Governed Vendor Selection

Purpose: sell the team on the idea today, while making the next few days of demo work concrete.

## Core Message

We are not building a chatbot that writes procurement summaries. We are building a governed execution environment where a buyer's intent becomes an auditable decision.

The hackathon app is **Helm**: the operator surface. Axiom defines the truth contract. Organism forms the team. Converge runs the proposal, promotion, policy, and fixed-point loop.

For the first demo, the concrete case is AI partner selection. The buyer submits a document pack with vendors, capabilities, risks, compliance status, certifications, and costs. The system detects missing, underspecified, or contradictory inputs. Given a good specification, it converges on the best candidate from the submitted list.

The future case is the breakout: the formation can challenge the original problem frame. Instead of forcing a single vendor, it may discover that the real need is a governed provider mix behind Kong or OpenRouter.

## RFI/RFP Boundary

The real-world process is broader than the demo. For the pitch, be explicit that we are not claiming to automate the entire procurement lifecycle today.

Five-phase framing:

| Phase | Real RFI/RFP activity | Demo status |
|---|---|---|
| 1. Define and scope | Management problem definition, ownership, stakeholders, budget, timeline, success criteria | Assumed upstream |
| 2. Explore market | Market discovery, internal interviews, longlist, optional RFI | Assumed upstream |
| 3. Specify and compete | Requirements refinement, RFP package, evaluation model, vendor submissions | Input to demo |
| 4. Evaluate and validate | Compliance, price, risk, scoring, trade-offs, PoC/pilot evidence | **Primary demo focus** |
| 5. Decide and justify | Recommendation, management approval, authority gate, decision record | **Primary demo focus** |
| 6. Contract and onboard | Negotiation, legal terms, SLA, implementation kickoff | Assumed downstream |

What the demo focuses on today:

- Given a structured buyer document pack and submitted vendors, run a governed evaluation.
- Detect obvious gaps and hard blockers such as pending compliance or excessive risk.
- Apply transparent scoring and hard constraints instead of gut-feel ranking.
- Produce a recommendation with trade-offs and an explicit policy/HITL gate.
- Show that the system can converge or honestly stop.

What is assumed today:

- The management need is already defined well enough to create a vendor selection job.
- Stakeholders have already agreed on first-pass criteria and constraints.
- The vendor list and declared evidence already exist in the document pack.
- Commercial negotiation, PoC evidence, contracting, and onboarding are outside the current run.

Speaker text when starting the demo:

> "Before Helm starts, we assume the organization has already done the upstream procurement work: management has defined the problem, ownership is assigned, stakeholders are known, the market has been explored, the RFI/RFP path has produced candidate vendors, and the first-pass evaluation criteria exist. We are starting at the point where the buyer has a document pack and needs a governed, defensible evaluation."

What Helm receives in the demo:

- Candidate vendors and declared capabilities.
- Compliance status and certifications.
- Cost inputs.
- Risk inputs.
- Evaluation thresholds.
- Authority and HITL policy context.

What happens downstream after the demo:

- Management reviews the recommendation, alternatives, assumptions, and open risks.
- Procurement may run clarification, best-and-final-offer, or a commercial negotiation round.
- Legal and security validate contract terms, data handling, SLAs, and regulatory exposure.
- Finance validates budget and commercial impact.
- A PoC or pilot may be required before final award.
- Contracting formalizes the vendor selection.
- Onboarding turns the selected option into an implementation plan.
- Outcomes feed back into the learning registry for the next decision.

Speaker text when closing the demo:

> "This does not end with a magic purchase order. The output is a decision package: recommended vendor or router strategy, ranked alternatives, rejected candidates, compliance/price/risk evidence, assumptions, policy outcome, and open issues for legal, finance, security, and implementation. That package moves into the normal downstream procurement path."

Important answer for RFI/RFP experts:

> "We are not replacing the entire RFI/RFP lifecycle in this demo. We are showing the governed evaluation and decision slice: once the RFP responses and criteria exist, Helm makes the scoring, evidence, gaps, recommendation, and approval boundary explicit. The future-state flow moves earlier into RFI/RFP creation, clarification loops, PoC evidence, and eventually reframing the problem when the submitted vendor set is not the real answer."

## Act 1: Stack Vocabulary - 30 Seconds

Say this bottom-up:

| Layer | One sentence |
|---|---|
| **Converge** | The runtime that promotes proposals into facts only when evidence, criteria, and policy allow it. |
| **Organism** | The intelligence layer that forms the right team for the shape of the problem. |
| **Axiom** | The truth-and-policy contract: what must be true before the result is allowed to count. |
| **Helm** | The application: where the operator uploads, inspects, approves, and acts. |

Product-facing version:

> "The operator sits in Helm. Axiom says what must be true. Organism forms the team. Converge makes the decision auditable."

## Act 2: Today - Governed Vendor Selection

Thesis:

> "This run replaces human document exchange with AI-supported convergence, but still selects among the original RFI/RFP vendors."

The buyer submits a document pack. In the executable demo this is the `vendors_json` input for the `vendor-selection` truth.

The formation assembled today:

| Agent | Role | Current model tier |
|---|---|---|
| Planning Seed | Intent and formation assembly | Deterministic |
| Compliance Screener | GDPR, AI Act, certifications, data residency | Live: fast OpenRouter model, with direct-provider fallback; offline: deterministic |
| Cost Analysis | TCO and budget fit | Live: mid OpenRouter model, with direct-provider fallback; offline: deterministic |
| Vendor Risk | Lock-in, operational, and compliance risk | Live: mid OpenRouter model, with direct-provider fallback; offline: deterministic |
| Shortlist | Hard constraints and objective-function ranking | Deterministic constraint solver |
| Decision Synthesis | Evidence-backed recommendation | Live: strong OpenRouter model, with direct-provider fallback; offline: stub synthesis |
| Policy Gate | Promote, escalate, or reject | Cedar policy engine |

Live-provider note: the script first tries the intended OpenRouter model tiers (`meta-llama/llama-3.1-8b-instruct`, `google/gemini-2.0-flash-001`, `anthropic/claude-sonnet-4`). If OpenRouter is unavailable or out of credits, it falls through to configured direct providers so the demo can still exercise live LLM calls.

Accuracy note: capability is represented today by the submitted vendor `score` and the weighted objective function. A separate capability matcher agent is future work unless we add it before the demo.

Key points to show:

1. **Gap and contradiction detection**: pending compliance is not papered over. A pending or failed vendor is flagged and rejected by the shortlist if it violates the constraints.
2. **Transparent objective function**: `0.35*capability + 0.25*risk_adjusted + 0.20*cost_efficiency + 0.20*certification_coverage`.
3. **HITL gate**: high-value commitments require human approval. The system honestly escalates instead of pretending approval exists.
4. **Learning loop**: `ExperienceRegistry` records cycles, elapsed time, confidence, and recommendation. Prior context feeds later synthesis.
5. **Fixed point**: Converge runs until no suggestor can propose a new promotable fact. This is not "three LLM rounds"; it is convergence to a fixed point or an honest stop.

Suggested live line:

> "The important part is not that an AI recommended a vendor. The important part is that every claim had to become a promoted fact, and policy had a chance to stop it."

## Act 3: Tomorrow - Pareto Breakout

Thesis:

> "We thought we were selecting one AI vendor. The formation found that the better answer is a governed provider mix behind a router."

This is implemented as `DemoMode::ParetoBreakout`.

The `router_hypothesis` turns true when the candidate set shows enough differentiated viable providers:

| Signal | Threshold |
|---|---:|
| Compliant vendors | 3 or more |
| High-capability vendors | 2 or more |
| Low-risk vendors | 2 or more |
| Cost spread | At least $20K |

The important boundary:

> "The system can break out of the single-vendor sandbox. It cannot break out of governance."

OpenClaw framing:

> "Stick to the needs rather than suggesting values and solutions too early. The AI assistance can spend more cycles expanding around the real needs."

Provider mix narrative:

| Need | Route |
|---|---|
| Programming and agentic reasoning | Strong reasoning or coding model when ambiguity or risk is high |
| Routine structured synthesis | Fast reliable structured-output model |
| Broad web evidence | Brave-style wide search |
| Deep canonical evidence | Tavily-style focused retrieval |
| Governance controls | Kong-style gateway for policy, rate limits, audit, PII, and cost controls |

How to say it:

> "Today's flow stays inside the RFI/RFP sandbox. Tomorrow's flow can reframe the need: you may not need one AI vendor; you may need model routing with governance, cost controls, evidence routing, and audit built in."

## Act 4: Demo Close

Governed flow close:

> "This is AI-supported convergence for procurement: faster than human document exchange, but still bounded by policy, evidence, and authority."

Breakout close:

> "The surprising result is not a vendor. The surprising result is that the right architecture might be a governed provider mix."

Final team ask:

> "For the next few days, we should make the story unmistakable: Helm shows the operator what was submitted, what was missing, what was promoted, where policy stopped the run, and why the final recommendation is defensible."

## What To Show Live

Fastest path is the headless demo:

```bash
just demo              # full business-facing walkthrough, offline
just demo-live         # full business-facing walkthrough, live providers
just demo-step 3       # jump to a presentation step
just demo-verify       # assert the expected business story
```

The guided walkthrough prints a liveness line before each long run, for example `Start processing: governed selection without HITL approval ... estimated time 90 seconds`, then rotates through the same spinner verbs used by the desktop flows.

Lower-level flow commands:

```bash
just demo-flow-governed
just demo-flow-breakout
```

For the clearest terminal output, use the business printer:

```bash
just demo-ai-vendors --mode=governed --business
just demo-ai-vendors --mode=pareto-breakout --business
```

JSON output for screenshots or inspection:

```bash
just demo-flow --mode=governed --json
just demo-flow --mode=pareto-breakout --json
```

Live-provider path, if API keys and provider config are ready:

```bash
just demo-flow --mode=governed --live --human-approval
just demo-flow --mode=pareto-breakout --live --human-approval
```

For the presentation walkthrough, use:

```bash
just demo-live
```

If OpenRouter returns an account or credit error, the script falls through to direct configured providers (`GEMINI_API_KEY`, `OPENAI_API_KEY`, or `ANTHROPIC_API_KEY`) before using deterministic per-agent fallbacks.

HTTP path through the local server:

```bash
just server
```

Then, from another shell:

```bash
VENDORS=$(cat examples/vendor-selection/seed-vendors.json)

curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
  -H "Content-Type: application/json" \
  -d "{\"inputs\":{\"vendors_json\":$(echo "$VENDORS" | jq -c '.' | jq -Rs .),\"min_score\":\"75\",\"max_risk\":\"30\",\"demo_mode\":\"governed\"},\"persist_projection\":false}" | jq .

curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
  -H "Content-Type: application/json" \
  -d "{\"inputs\":{\"vendors_json\":$(echo "$VENDORS" | jq -c '.' | jq -Rs .),\"min_score\":\"75\",\"max_risk\":\"30\",\"demo_mode\":\"pareto-breakout\"},\"persist_projection\":false}" | jq .
```

Look for these response fields:

| Field | Why it matters |
|---|---|
| `converged` and `cycles` | Shows fixed-point execution. |
| `criteria_outcomes` | Shows which governance criteria passed or blocked. |
| `projection.details.formation` | Shows the assembled Organism formation. |
| `projection.details.shortlist` | Shows ranking, rejection reasons, and composite scores. |
| `projection.details.policy` | Shows the Cedar outcome and HITL state. |
| `projection.details.router_hypothesis` | Shows the breakout strategy and provider mix. |
| `projection.details.learning` | Shows prior-run learning metrics when experience is available. |

## Fulfillment Plan

### Today: Team Pitch

- Use this page as speaker notes.
- Run `just demo-verify` before the meeting.
- Run `just demo` for the reliable offline walkthrough.
- Run `just demo-live` only when provider keys/network are ready.
- Present the breakout as a future-facing capability that is already represented in code, not as a fully finished UI workflow.
- Be explicit that live LLM/search mode depends on provider configuration; the deterministic path is the reliable fallback.

### Day 1: Make The Demo Reliable

- Verify both headless modes from a clean terminal.
- Capture one good governed output and one good breakout output.
- Confirm the HTTP request shape uses `/v1/truths/vendor-selection/execute` with nested `inputs`.
- Add a single "known good" command or script if the live demo needs fewer moving parts.
- Decide whether the presentation uses headless CLI, desktop Helm, or both.

### Day 2: Tighten The Product Story

- Make the desktop surface show the same concepts in the same order: intake, formation, promoted facts, shortlist, policy, learning, router hypothesis.
- Ensure the UI distinguishes "as of today" from "hypothetical breakout" without weakening the vision.
- Keep the first screen focused on the usable Helm workflow, not marketing copy.
- Add screenshots or a short screen-recording backup in case live provider calls fail.

### Day 3: Close Implementation Gaps

- Decide whether to add a real `CapabilityMatcherAgent` or keep capability as the submitted `score`.
- Strengthen gap detection for missing fields, underspecified requirements, and contradictions in the document pack.
- Add or update tests for the two presentation paths: governed selection and Pareto breakout.
- Run `just test` and `just lint` before treating the demo branch as ready.

### Demo Day: Runbook

- Start from a clean terminal.
- Run `just demo` for the guided walkthrough.
- In Step 3, point to the boxed process map, agent roster, and per-agent outcomes.
- Use yourself as HITL: promote, escalate, or reject the Mistral commitment.
- After your HITL decision, show the Cedar delegation candidate and explain that delegation is reviewed, scoped, and never a bypass.
- In Step 4, explain the gate mechanics: recommendation, optimization, human approval, Cedar authorization, and Converge audit are separate.
- In Step 5, call out that this is a negative-control run: the same Mistral recommendation is rejected when authority is only advisory.
- In Step 6, explain that the process restarts to grow `prior_context`; hard constraints do not change.
- In Step 7, use the richer provider-mix data and point to the governed Pareto frontier before the router hypothesis.
- End with the fixed-point line: "This converged because no agent had a new promotable fact."

## Risks To Avoid In The Pitch

| Risk | Clean phrasing |
|---|---|
| Overclaiming live intelligence | "Live mode can use LLM-backed suggestors; offline mode is deterministic for reliability." |
| Making Cedar sound optional | "HITL is present today; Cedar is the formal policy gate and delegation path." |
| Saying the breakout ignores the RFI/RFP | "It breaks the single-vendor assumption, not the governance boundary." |
| Talking about models too early | "The formation starts from needs, then lower layers choose models, search, tools, and policies." |
| Hiding uncertainty | "If evidence is missing, contradictory, or unauthorized, the system stops or escalates." |
