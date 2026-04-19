---
tags: [kong, converge, challenge]
---
# Why Kong — Challenge Framing

## The Participant Pitch (30-Min Meeting)

> **"You're building AI that justifies every decision. Kong is how you govern what goes out over the wire."**

### The Two-Layer Story

```
┌──────────────────────────────┐
│  Your Decision (Converge)    │  ← Internal: who decides what
│  • Agents propose facts      │
│  • Policies gate decisions   │
│  • Converges or stops honest │
└──────────────┬───────────────┘
               │ Every LLM call must be guarded
               ↓
┌──────────────────────────────┐
│  External Access (Kong)      │  ← Boundary: what leaves the system
│  • Rate limits per team      │
│  • PII sanitizer             │
│  • Prompt guard              │
│  • Audit & cost tracking     │
└──────────────────────────────┘
```

**Converge** governs *what agents decide*. **Kong** governs *what agents access*. Together, zero gaps.

### The Live Demo

1. **Run vendor eval** — Converge engine cycles through compliance, cost, risk, synthesis agents
2. **Show Kong dashboard** — every LLM call logged: tokens, cost, latency, which agent, timestamp
3. **Trigger PII redaction** — propose a vendor with sensitive data in the name, watch Kong sanitize it before it hits Claude
4. **Hit rate limit** — show what happens when a team exhausts their token budget
5. **View cost-per-decision** — "This governance decision cost $0.47 in Claude tokens"

### Why Both Layers Matter

| Layer | What It Sees | What It Can't See |
|-------|---|---|
| **Converge** | Agent proposals, policy enforcement, convergence criteria | What leaves the system, who's calling the LLM, token costs |
| **Kong** | LLM requests, PII patterns, budgets, audit logs | Whether the agent's reasoning was sound, if the decision makes sense |
| **Together** | Complete governance: sound decisions + safe access | (Nothing — every gate is covered) |

## The Challenge

> Build an enterprise-grade AI governance infrastructure that fulfills the AI compliance requirements and business needs. Your infrastructure must be secure, scalable and compliant, enforce usage policies, monitor LLM consumption and include automated guardrails that protect sensitive enterprise data without slowing down innovation.

## Our Answer: Two-Layer AI Governance

Converge governs what agents **decide**. Kong governs what agents **access**. Together they cover every requirement.

### Requirement Map

| Challenge Requirement | Converge (Internal Governance) | Kong (External Governance) |
|---|---|---|
| **Secure** | Every mutation needs an Actor. Cedar policies enforce who can propose, promote, commit. Ed25519 delegation tokens. | Centralized credentials — provider API keys never leave Kong. Key Auth per team. TLS everywhere. |
| **Scalable** | Engine runs in-process, zero network overhead for governance logic. Budgets prevent runaway agents. | Kong scales horizontally. Load balancing across LLM providers. Semantic routing to cheapest model that fits. |
| **Compliant** | Full audit trail: every fact has provenance (who proposed, what evidence, what confidence). Cedar policies enforced at promotion gates. | PII Sanitizer strips sensitive data before it reaches models. Prompt Guard blocks prohibited content. Data residency routing. |
| **Enforce usage policies** | Cedar policies gate agent authority levels (advisory, participatory, supervisory, sovereign). Promotion requires passing criteria. | AI Rate Limiting per team. Token budgets. Prompt Guard allow/deny lists. |
| **Monitor LLM consumption** | ExperienceStore captures per-agent cost, tokens, model — metadata agents learn from across runs. | Kong Audit Log: every request logged with tokens, cost, latency, model, status. Konnect dashboard for real-time visibility. |
| **Automated guardrails** | Agents emit proposals, never direct facts. Criteria evaluate convergence. HITL gates fire automatically on low confidence. | PII redaction (20 categories, 9 languages). Semantic Prompt Guard. Content safety. Rate limiting. All automatic, no code changes. |
| **Protect sensitive data** | Facts have typed provenance. No agent can mutate context directly. Cedar policies control access. | PII Sanitizer runs before prompts reach models. Credentials centralized — never in application code. |

### Why You Need Both Layers

**Kong alone** can route, rate-limit, and redact. But it has no opinion about what agents decide. An agent could hallucinate a recommendation and Kong would not know. Kong governs the pipe, not the logic.

**Converge alone** can govern what agents propose and promote. But it has no visibility into what goes out over the wire. A suggestor could leak PII in a prompt and Converge would not see it. Converge governs the logic, not the pipe.

**Together** there is no gap between what is decided and what is accessed:

```
Participant writes a suggestor
  → Converge: Does this agent have Cedar authority to propose?
  → Kong: Is this prompt PII-safe? Within budget? Allowed content?
  → LLM responds
  → Kong: Log tokens, cost, latency
  → Converge: Does this proposal pass criteria? Promote or reject?
  → ExperienceStore: Remember what this cost for next time
```

### Without Slowing Down Innovation

Participants never see Kong. They write `ChatRequest`, call `select_chat_backend()`, and get a response. They can:

- Write new suggestors (only touch `converge-pack`)
- Add Cedar policies (only touch policy fixtures)
- Build new truths (only touch `governance-truths`)
- Visualize governance (only touch Svelte)

Zero Kong configuration. Zero credential management. Zero awareness of which upstream model they hit. Governance that is invisible until you need to audit it.

### The Audit Story

Together Converge and Kong answer every question an auditor asks:

1. **How did you reach this decision?** Converge: full convergence trail, criteria evaluation, Cedar policy gates, every fact's provenance chain.
2. **What external resources did you consult?** Kong: API audit log, token counts, cost, model used per request.
3. **Did you leak sensitive data?** Kong: PII Sanitizer logs showing what was redacted before prompts reached models.
4. **Did you stay within budget?** Kong: Rate limiter enforcement, per-team token budget tracking, cumulative cost.
5. **Who authorized this?** Converge: Cedar policy evaluation, Actor on every mutation, delegation token chain.

See also: [[Integrations/Kong Gateway]], [[Integrations/Kong Demo Story]], [[Converge/Core Concepts]]