---
tags: [integrations, web-search, demo, providers]
---
# Web Search Demo Story

This demo shows that model choice is only half of provider strategy. Search choice matters too. The strongest setup is a mixed formation: some agents use broad discovery to avoid blind spots, while others use deep retrieval to ground specific claims.

## The Thesis

Different agents need different evidence shapes.

| Provider | Search shape | Best use |
|---|---|---|
| **Brave** | Wide discovery | Find many candidate sources, market signals, news, vendor pages, alternative claims |
| **Tavily** | Deep retrieval | Pull focused, high-signal evidence from known topics, documents, pricing pages, policy references |
| **No search** | Local-only reasoning | Synthesize already-promoted facts without introducing new unsupported claims |

The governance point is simple: the system should not pick one provider for every task. It should route each agent to the provider that matches the evidence need.

## Demo Framing

The operator runs `evaluate-vendor` for three AI vendors. The app shows a provider mix before execution:

```
Compliance Screener     Tavily deep search + structured model
Cost Analysis           Brave wide scan, then Tavily pricing lookup
Risk Scorer             Brave wide scan for market/news signals
Decision Synthesis      No search by default; consumes promoted facts
Summary Writer          Writer/Palmyra-style polishing model
Escalation              Claude/GPT Pro only for high-risk ambiguity
```

This is the visible lesson: agents are not interchangeable. Each agent has a job, and provider routing should reflect the job.

## Headless Demo Modes

The CLI demo has two tracks:

| Mode | What it proves | Boundary |
|---|---|---|
| **Governed selection** | AI-supported convergence can replace human document exchange while keeping HITL, Cedar policy, evidence, and authority explicit. | The selection remains among the vendors supplied through the RFI/RFP intake. |
| **Creative Pareto breakout** | The formation can challenge a local-minimum framing and propose a multi-provider/router strategy when the candidate set has differentiated strengths. | It can break out of the single-vendor assumption, not out of governance. Cedar, provenance, authority, and policy gates still apply. |

The second mode is the audience wake-up moment:

> "We thought we were selecting one AI vendor. The formation found that the better answer is a governed provider mix behind a router."

This is the safe version of creative problem solving: the system can challenge the problem frame, but the promoted result remains typed, policy-gated, and auditable.

## Desktop Slide Mapping

The desktop app now carries the extended story as numbered slides. The presenter can type a subset such as `1,2,3,8,12,20,24,25` in the slide selector and present only those original slide numbers.

| Slide | Topic |
|---:|---|
| 15 | RFI/RFP intake turns uploaded documents into normalized requirements, vendors, constraints, and artifacts. |
| 16 | Formation declares needs instead of hard-coding providers or models. |
| 17 | Huddle shows agents coordinating through promoted facts rather than direct chat. |
| 18 | Steps show compliance, price, risk, and optimization as separate evidence shapes. |
| 19 | Consensus explains fixed-point convergence and honest stopping. |
| 20 | Wide + deep search explains Brave for breadth and Tavily for depth. |
| 21 | Non-LLM teammates explains RAG, policy, optimization, statistics, ML, and data analysis. |
| 22 | Governed selection mode replaces human document exchange while staying inside the RFI/RFP vendor set. |
| 23 | Creative Pareto breakout mode challenges the single-vendor local minimum. |
| 24 | Router hypothesis shows Kong/OpenRouter as the discovered architecture. |
| 25 | Demo close: governed team, not one magic model. |

## Agent Mix

### Compliance: Deep Search

Compliance wants precise evidence, not broad chatter. Use Tavily for vendor security pages, data residency documentation, DPA pages, SOC 2 references, AI Act notes, and privacy policy clauses.

The agent should produce narrow facts:

```
compliance:screen:Acme AI
evidence:
  - source: vendor DPA
  - source: security documentation
  - source: privacy policy
```

Why this is good: compliance failure should block the decision, so evidence quality matters more than source volume.

### Cost: Wide Then Deep

Cost benefits from a two-step pattern. Brave finds the pricing surfaces and recent plan changes. Tavily then pulls the specific pricing page or calculator detail.

```
Brave: "Acme AI pricing enterprise inference 2026"
Tavily: retrieve focused pricing terms from the likely canonical page
```

Why this is good: pricing data is fragmented. Wide search finds where the answer lives; deep search extracts the answer with fewer hallucination risks.

### Risk: Wide Search

Risk needs weak signals: outages, security incidents, funding news, layoffs, product deprecations, customer complaints, regional restrictions, and terms-of-service changes. Brave is the better default because the agent wants breadth before scoring.

The agent should produce weighted risk evidence:

```
risk:score:Acme AI
signals:
  - operational risk
  - financial stability
  - vendor lock-in
  - compliance/regulatory exposure
```

Why this is good: risk is often outside official vendor docs. A wide scan catches signals a narrow lookup would miss.

### Decision Synthesis: No Search First

Synthesis should not go searching by default. It should consume promoted facts from compliance, cost, and risk, then explain the recommendation. Search during synthesis is an escalation path, not the normal path.

Why this is good: final recommendations should be grounded in governed evidence, not new unreviewed claims introduced at the last step.

## Model Mix

The model competition results support the same principle for LLMs:

| Role | Practical default |
|---|---|
| Discussion and synthesis | `mistralai/mistral-small-2603` or `arcee-ai/trinity-large-preview` |
| Fast structured output | `google/gemma-4-31b-it` or `google/gemini-3.1-flash-lite-preview` |
| Stakeholder-ready summary | `writer/palmyra-x5` |
| Escalation | Claude/GPT Pro for high-risk or high-ambiguity decisions |

Do not treat all large models as better by default. The tests showed that latency, structured-output reliability, and fallback rate matter as much as raw reasoning strength.

## Slide: Not Every Agent Is An LLM

The next demo slide should make the architecture point explicit: Converge is not an "LLM swarm." It is a governed decision runtime where each agent can use the right kind of intelligence.

| Agent capability | Best used for | Example in vendor selection |
|---|---|---|
| **LLM** | Discussion, synthesis, structured interpretation | Summarize promoted facts into a recommendation |
| **Web search** | External evidence gathering | Find pricing pages, security docs, incident signals |
| **Knowledgebase / RAG** | Retrieve known internal context | Pull procurement policy, prior vendor decisions, internal requirements |
| **Policy engine** | Hard allow/deny gates | Block vendors that fail mandatory compliance requirements |
| **Optimization / math** | Constraint solving and trade-off selection | Pick the best vendor under cost, latency, risk, and residency constraints |
| **Statistics** | Confidence, variance, anomaly detection | Flag unstable pricing or high variance in benchmark results |
| **Machine learning** | Classification, prediction, scoring | Predict operational risk from historical incidents or support metrics |
| **Data analysis** | Aggregation and comparison | Compare benchmark results, cost curves, and usage forecasts |

The key message: not every decision step should be delegated to a language model. Some steps are better handled by deterministic policy, numerical optimization, statistical scoring, or existing organizational knowledge.

## Why RAG Alone Is Not Enough

RAG is useful, but it is only one evidence source. A RAG-only solution can retrieve and summarize documents, but it does not by itself:

- enforce mandatory policies,
- solve optimization constraints,
- prove that all required criteria were evaluated,
- separate proposed facts from promoted facts,
- capture provenance for every decision step,
- decide when human approval is required,
- combine live web evidence with internal knowledge and numerical scoring.

For this demo, RAG belongs inside the formation as one capability. It is not the formation itself.

The stronger framing:

```
RAG retrieves context.
Search discovers external evidence.
Policy decides what is allowed.
Math and statistics score trade-offs.
LLMs interpret and synthesize.
Converge governs what becomes part of the record.
```

## Slide: Relentless Exploration, Governed Promotion

The formation should not hard-code provider or model details. It should express needs:

```
Need broad external evidence.
Need deep canonical evidence.
Need policy enforcement.
Need cost optimization.
Need risk scoring.
Need synthesis.
Need adversarial challenge when evidence conflicts.
```

Lower layers decide the provider mix: Brave, Tavily, knowledgebase retrieval, policy engine, optimizer, statistics, ML model, or LLM.

The formation should be persistent against uncertainty:

- If a provider times out, try another provider.
- If an LLM returns malformed JSON, repair or retry with a smaller prompt.
- If evidence is thin, widen search.
- If evidence is noisy, deepen retrieval.
- If evidence conflicts, trigger adversarial analysis.
- If confidence is weak, run optimization, statistics, or prior-case comparison before giving up.

But it must remain obedient to governance:

- Cedar policy can block or escalate.
- Mandatory compliance failures stop the recommendation path.
- Authority boundaries cannot be bypassed.
- Evidence that lacks provenance cannot be promoted.
- Learning from prior runs can propose policy calibration, but it cannot silently override policy.

This is the key posture:

> Relentless against uncertainty, provider failure, and incomplete evidence. Obedient to policy, authority, and hard constraints.

## What The Audience Should Learn

1. Provider selection is a governance decision, not a static config choice.
2. Wide search and deep search solve different evidence problems.
3. The right agent mix reduces hallucination because each agent handles the evidence shape it is good at.
4. The best formation mixes LLMs with search, policy, knowledgebase retrieval, optimization, statistics, ML, and data analysis.
5. Converge makes the mix auditable: each promoted fact records which agent produced it and what evidence supported it.
6. Escalation models are valuable, but they should be reserved for ambiguity and risk rather than used for every call.

## Demo Close

The closing line:

> "We did not ask one model to do everything. We formed a governed team: Brave for breadth, Tavily for depth, specialized models for each role, and Converge to decide which evidence becomes part of the record."

Alternate close for the non-LLM slide:

> "This is not RAG with extra steps. RAG is one teammate. The governed system also needs policy, optimization, statistics, data analysis, and typed promotion before a decision becomes trustworthy."
