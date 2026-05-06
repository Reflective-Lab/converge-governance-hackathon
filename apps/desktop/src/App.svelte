<script lang="ts">
  import { onDestroy } from "svelte";
  import { invokeTauri } from "./lib/tauri";
  import { randomVerb } from "./lib/spinner";
  import ProviderSelector from "./lib/ProviderSelector.svelte";
  import AIProviderEvaluation from "./lib/AIProviderEvaluation.svelte";

  // ─── Phases ───
  const primaryDeckSelection = "5,6,7,8,1,2,3,4,19,26,30,33,34";
  let phase = $state("loop"); // slides | providers | loop | demo | apps | dd
  let currentSlide = $state(0);
  let slideSelectionInput = $state(primaryDeckSelection);
  let slideSelectionStatus = $state("Showing primary demo deck");

  interface Slide {
    number: number;
    eyebrow: string;
    headline: string;
    body: string;
    image: string;
    layout?: "stack" | "flow" | "gateway";
    stackLayers?: {
      name: string;
      role: string;
      detail: string;
    }[];
    flowSteps?: {
      name: string;
      detail: string;
    }[];
    gatewayPoints?: {
      name: string;
      detail: string;
    }[];
  }

  const allSlides: Slide[] = [
    {
      number: 1,
      eyebrow: "The Core Sell",
      headline: "Governance\nby construction.",
      body: "Every AI framework today lets agents do things. We built one that makes agents prove they should. Every decision can be explained to a regulator. You know exactly why something was promoted or rejected. The system stops honestly when stuck. Policy gates are integral to convergence.",
      image: "/images/slides/hero.jpg",
    },
    {
      number: 2,
      eyebrow: "Architecture Layers",
      headline: "Converge + Kong.\nComplementary, not competitive.",
      body: "Two governance layers with different responsibilities: access control before the model call, truth control after the model answers.",
      image: "/images/slides/layers.jpg",
      layout: "gateway",
      gatewayPoints: [
        {
          name: "Kong protects the perimeter",
          detail: "Kong decides whether your request reaches the LLM.",
        },
        {
          name: "Converge protects the facts",
          detail: "Converge decides whether the LLM's answer becomes truth.",
        },
        {
          name: "Together",
          detail: "Kong governs access. Converge governs promotion, evidence, and decision state.",
        },
      ],
    },
    {
      number: 3,
      eyebrow: "The Science",
      headline: "Lamport Clocks.\nSAT/CSP. Hungarian Algorithm.\nDijkstra's Frontier.",
      body: "This is not a chatbot architecture. Converge uses classical computer science to constrain language-based reasoning: proven causal ordering, mathematical feasibility checking, optimal matching, and efficient search.",
      image: "/images/slides/circuit.jpg",
    },
    {
      number: 4,
      eyebrow: "Governance Model",
      headline: "Four layers.\nFrom proposal to decision.",
      body: "Every promoted fact moves through explicit authority, invariant, and review gates before it becomes decision state. The audit trail is created as part of the promotion path.",
      image: "/images/slides/stack.jpg",
      layout: "stack",
      stackLayers: [
        {
          name: "Agents Propose",
          role: "Constructible",
          detail: "Agents freely construct proposals, but proposals are not facts yet.",
        },
        {
          name: "Cedar Scopes Authority",
          role: "Policy",
          detail: "Policy decides who may promote, commit, approve, or escalate.",
        },
        {
          name: "Invariants Gate Truth",
          role: "Hard constraints",
          detail: "Hard constraints block unsafe or unsupported promotions.",
        },
        {
          name: "HITL Is First-Class",
          role: "Human review",
          detail: "Human approval is modeled directly, not bolted on after the run.",
        },
        {
          name: "Audit Trail",
          role: "Record",
          detail: "Every promoted fact keeps actor, evidence, rationale, and path.",
        },
      ],
    },
    {
      number: 5,
      eyebrow: "The Problem",
      headline: "Vendor decisions\nare a black box.",
      body: "Enterprises evaluate AI vendors with spreadsheets, email chains, and gut feel. No audit trail, no reproducibility, no governance.",
      image: "/images/slides/hero.jpg",
    },
    {
      number: 6,
      eyebrow: "The Enterprise Reality",
      headline: "Towers of process.\nZero transparency.",
      body: "Procurement committees, legal review boards, security checklists — layers of approval that produce paper trails instead of machine-readable decisions.",
      image: "/images/slides/towers.jpg",
    },
    {
      number: 7,
      eyebrow: "Why It Matters",
      headline: "Compliance fails\nwhen process is invisible.",
      body: "Regulators ask for evidence. Boards ask for rationale. Without machine-readable governance, you are rebuilding the story after the fact.",
      image: "/images/slides/problem.jpg",
    },
    {
      number: 8,
      eyebrow: "The Scale",
      headline: "Every organization.\nEvery border.\nEvery decision.",
      body: "Vendor governance is not a local problem. Enterprises operate across jurisdictions, regulations, and risk profiles. The rules must travel with the data.",
      image: "/images/slides/earth.jpg",
    },
    {
      number: 9,
      eyebrow: "The Converge Way",
      headline: "Governance\nas code.",
      body: "A Truth is a machine-readable governance spec. Intent, authority, constraints, and evidence — declared up front, validated automatically, auditable forever.",
      image: "/images/slides/converge.jpg",
    },
    {
      number: 10,
      eyebrow: "The Pattern",
      headline: "Structure that\nemerges from flow.",
      body: "Governance is not a gate you pass through once. It is a continuous flow — living constraints that adapt as context shifts, not static checklists that rot.",
      image: "/images/slides/flow.jpg",
    },
    {
      number: 11,
      eyebrow: "How It Works",
      headline: "Intent. Authority.\nConstraint. Evidence.",
      body: "Every vendor decision declares what outcome it seeks, who can approve it, what limits apply, and what proof is required. Agents propose. Humans promote.",
      image: "/images/slides/howit.jpg",
    },
    {
      number: 12,
      eyebrow: "Deep Architecture",
      headline: "Layers that\ncompose cleanly.",
      body: "Domain packs, policy engines, promotion gates, and agent runtimes — each layer has a single responsibility. Compose them to build governance that fits your org.",
      image: "/images/slides/layers.jpg",
    },
    {
      number: 13,
      eyebrow: "The Stack",
      headline: "Converge platform.\nOrganism runtime.\nHelm desktop.",
      body: "Write a truth. Validate it against policy. Act on it with confidence. Governance that learns from every decision — processes that strengthen under stress and keep your organization safe by default.",
      image: "/images/slides/stack.jpg",
    },
    {
      number: 14,
      eyebrow: "Under The Hood",
      headline: "Silicon-level\nconfidence.",
      body: "From circuit board to policy decision — every layer is typed, validated, and auditable. Zero unsafe code. Zero runtime surprises. The machine earns trust.",
      image: "/images/slides/circuit.jpg",
    },
    {
      number: 15,
      eyebrow: "The Grid",
      headline: "Structured decisions\nat enterprise scale.",
      body: "Not a single vendor scorecard — a composable grid of evaluations, constraints, and approvals that scales across teams, business units, and geographies.",
      image: "/images/slides/grid.jpg",
    },
    {
      number: 16,
      eyebrow: "Living Systems",
      headline: "Governance that\nevolves organically.",
      body: "Organisms adapt. So should your vendor governance. As markets shift and regulations change, Truths update — and the audit trail shows exactly what changed and why.",
      image: "/images/slides/organic.jpg",
    },
    {
      number: 17,
      eyebrow: "Corporate Trust",
      headline: "The board sees\nwhat the machine sees.",
      body: "No more translating between technical artifacts and executive summaries. One Truth spec serves the engineer, the auditor, and the board — same artifact, same source of truth.",
      image: "/images/slides/corporate.jpg",
    },
    {
      number: 18,
      eyebrow: "Your Mission",
      headline: "Watch governance\nconverge.",
      body: "Today you will inspect a vendor-selection loop as formation, agents, optimization, policy, and gates turn proposals into an auditable decision.",
      image: "/images/slides/participants.jpg",
    },
    {
      number: 19,
      eyebrow: "RFI / RFP Intake",
      headline: "Documents become\nmachine-readable flow.",
      body: "The buyer uploads an RFI/RFP package. Intake extracts requirements, candidate vendors, constraints, and source artifacts before any agent starts deciding.",
      image: "/images/slides/towers.jpg",
    },
    {
      number: 20,
      eyebrow: "Formation",
      headline: "Declare needs.\nDo not pick tools too early.",
      body: "The top level asks for compliance evidence, pricing analysis, risk scoring, optimization, synthesis, and policy authorization. Lower layers choose the right provider/model/tool mix.",
      image: "/images/slides/converge.jpg",
    },
    {
      number: 21,
      eyebrow: "Huddle",
      headline: "Agents coordinate\nthrough promoted facts.",
      body: "This is not a loose chat room. Agents propose facts into typed context; Converge decides what becomes shared evidence before the next step can depend on it.",
      image: "/images/slides/grid.jpg",
    },
    {
      number: 22,
      eyebrow: "Steps",
      headline: "Compliance. Price.\nRisk. Optimization.",
      body: "Each step has a different shape: policy evidence, cost curves, operational risk, and mathematical trade-offs. The demo makes each step visible before consensus.",
      image: "/images/slides/flow.jpg",
    },
    {
      number: 23,
      eyebrow: "Consensus",
      headline: "The record is\nnot a vibe.",
      body: "Consensus means no suggestor has a new promotable fact under the current context, budget, authority, and policy gates. If a gate fails, the system honestly stops.",
      image: "/images/slides/circuit.jpg",
    },
    {
      number: 24,
      eyebrow: "Wide + Deep Search",
      headline: "Brave for breadth.\nTavily for depth.",
      body: "Risk wants broad market signals. Compliance wants canonical evidence. Cost often needs both: wide discovery to find pricing surfaces, then deep retrieval to ground the numbers.",
      image: "/images/slides/earth.jpg",
    },
    {
      number: 25,
      eyebrow: "Not Every Agent Is An LLM",
      headline: "RAG is one\nteammate.",
      body: "The governed team also needs policy, optimization, statistics, data analysis, machine learning, and knowledgebase retrieval before a decision becomes trustworthy.",
      image: "/images/slides/layers.jpg",
    },
    {
      number: 26,
      eyebrow: "Mode 1",
      headline: "Governed selection\nreplaces document exchange.",
      body: "This mode keeps the vendor choice inside the original RFI/RFP candidate set, while HITL and Cedar policy gates increasingly become formal delegation chains learned from prior runs.",
      image: "/images/slides/corporate.jpg",
    },
    {
      number: 27,
      eyebrow: "Mode 2",
      headline: "Creative breakout\nchallenges the premise.",
      body: "Sometimes the local optimum is the wrong frame. The formation can propose a Pareto breakout: a multi-provider mix behind a governed router, not a single forced winner.",
      image: "/images/slides/organic.jpg",
    },
    {
      number: 28,
      eyebrow: "Router Hypothesis",
      headline: "Kong or OpenRouter\nbecomes the answer.",
      body: "We thought we were evaluating AI vendors. The system can discover that the better architecture is a gateway/router: policy, audit, rate limits, search, models, and workload routing.",
      image: "/images/slides/stack.jpg",
    },
    {
      number: 29,
      eyebrow: "Demo Close",
      headline: "A governed team,\nnot one magic model.",
      body: "We formed a team: Brave for breadth, Tavily for depth, specialized models for each role, and Converge to decide which evidence becomes part of the record.",
      image: "/images/slides/participants.jpg",
    },
    {
      number: 30,
      eyebrow: "Technology Stack",
      headline: "The governed\nAI stack.",
      body: "Helm gives operators a control surface. Axiom defines truth. Organism does the reasoning work. Converge decides what may be promoted. Providers stay behind capability boundaries.",
      image: "/images/slides/stack.jpg",
      layout: "stack",
      stackLayers: [
        {
          name: "Helm",
          role: "Control surface",
          detail: "Desktop and web UX; what operators see.",
        },
        {
          name: "Axiom",
          role: "Truth layer",
          detail: "Truth definitions, projections, validation, domain state.",
        },
        {
          name: "Organism",
          role: "Intelligence",
          detail: "Intent to huddle to debate to suggestors; reasoning, research, gap chasing.",
        },
        {
          name: "Converge",
          role: "Governance",
          detail: "Engine, promotion gates, Cedar policy, budget, audit, authority, trust, stop rules.",
        },
        {
          name: "Providers",
          role: "Capability",
          detail: "OpenRouter, Anthropic, OpenAI, Gemini, Brave, Tavily.",
        },
      ],
    },
    {
      number: 31,
      eyebrow: "Technology Flow",
      headline: "Truth becomes\noperator state.",
      body: "A governed run is not a prompt chain. It is a typed execution path from declared truth to promoted facts to projected product state.",
      image: "/images/slides/flow.jpg",
      layout: "flow",
      flowSteps: [
        {
          name: "Axiom defines the run",
          detail: "Truth, projections, and run configuration are declared before execution.",
        },
        {
          name: "Axiom invokes Engine.run()",
          detail: "The governance runtime receives context, budget, criteria, and hooks.",
        },
        {
          name: "Organism emits proposals",
          detail: "It decomposes, plans, debates, researches, and proposes facts.",
        },
        {
          name: "Converge promotes or stops",
          detail: "Policy, budget, convergence, authority, and promotion gates are evaluated.",
        },
        {
          name: "Providers supply capability",
          detail: "Models, search, and tools are called through adapter boundaries.",
        },
        {
          name: "Axiom projects facts",
          detail: "Converged facts become durable product state for Helm.",
        },
        {
          name: "Helm shows the result",
          detail: "Operators see the decision, rationale, evidence, and next action.",
        },
      ],
    },
    {
      number: 32,
      eyebrow: "Business Architecture",
      headline: "Governed process.\nGateway control.",
      body: "The business value is not just a better answer. It is a governed vendor-selection process that improves as Kong becomes the AI Gateway for policy, access, cost, and audit.",
      image: "/images/slides/circuit.jpg",
      layout: "gateway",
      gatewayPoints: [
        {
          name: "Converge governs decisions",
          detail: "Inside the runtime, facts advance only when authority, evidence, policy, budget, and stop rules agree.",
        },
        {
          name: "Kong governs access",
          detail: "At the AI Gateway, model calls can inherit rate limits, prompt guardrails, PII handling, routing, cost tracking, and audit logs.",
        },
        {
          name: "The process learns",
          detail: "Gateway telemetry and governed outcomes feed the next run, turning vendor selection into an auditable improvement loop.",
        },
      ],
    },
    {
      number: 33,
      eyebrow: "Kong AI Gateway",
      headline: "Responsible AI\nbecomes enforceable.",
      body: "With Kong as the AI Gateway, governed AI moves from hoping teams use models responsibly to an enforceable control plane for every model call.",
      image: "/images/slides/circuit.jpg",
      layout: "gateway",
      gatewayPoints: [
        {
          name: "One path to every model",
          detail: "PII handling, prompt management, provider routing, token limits, cost controls, and audit telemetry happen before calls reach OpenAI, Anthropic, Gemini, or any other backend.",
        },
        {
          name: "Output enters governance",
          detail: "In the Converge stack, AI output is budgeted, policy-aware, traceable, and tied back to an actor, intent, proposal, and promotion path.",
        },
        {
          name: "Speed without loss of control",
          detail: "Enterprises get AI velocity while keeping control of privacy, spend, compliance, and decision accountability.",
        },
      ],
    },
    {
      number: 34,
      eyebrow: "This Demo",
      headline: "AI extends\nhuman capability.",
      body: "We bring in more vendors, more requirement dimensions, and more evidence than a human team can comfortably hold in working memory. Converge turns that complexity into an understandable governed decision.",
      image: "/images/slides/grid.jpg",
      layout: "flow",
      flowSteps: [
        {
          name: "More vendors, more dimensions",
          detail: "AI expands the competition from a small manual shortlist into a broader vendor field scored across many requirement dimensions.",
        },
        {
          name: "LLMs plus mathematical solvers",
          detail: "Language models research, critique, and propose; mathematical solvers make the trade-offs explicit instead of hiding them in prose.",
        },
        {
          name: "Convergence to a fixed point",
          detail: "Converge keeps promoting only governed facts until the system has no new promotable evidence under the current policy, budget, and authority.",
        },
        {
          name: "Obvious 3D understanding",
          detail: "The 3D visualization makes the result legible: operators can see the shape of the decision instead of reading another spreadsheet.",
        },
        {
          name: "Kong controls AI access",
          detail: "Kong keeps AI use controlled and secure with gateway policy, routing, limits, telemetry, and audit before calls reach model providers.",
        },
      ],
    },
  ];

  let slides = $state(slidesFromSelection(primaryDeckSelection));
  const microDeckSelection = "5,6,7,8,1,2,3,4,30,33,34";
  const shortDeckSelection = "5,6,7,8,1,2,3,4,19,26,30,33,34";
  const longerDeckSelection = "5,6,7,8,1,2,3,4,19,26,27,28,30,33,34";

  function parseSlideNumbers(raw: string): number[] {
    const seen = new Set<number>();
    return raw
      .replace(/[{}]/g, "")
      .split(/[,\s]+/)
      .map((part) => Number.parseInt(part.trim(), 10))
      .filter((number) => Number.isInteger(number))
      .filter((number) => {
        if (seen.has(number)) return false;
        seen.add(number);
        return true;
      });
  }

  function slidesFromSelection(raw: string): Slide[] {
    const selected = parseSlideNumbers(raw)
      .map((number) => allSlides.find((slide) => slide.number === number))
      .filter((slide): slide is Slide => Boolean(slide));
    return selected.length > 0 ? selected : allSlides;
  }

  function displaySlideNumber(index: number) {
    return index + 1;
  }

  function applySlideSelectionFrom(raw: string) {
    const numbers = parseSlideNumbers(raw);
    if (numbers.length === 0) {
      slides = allSlides;
      currentSlide = 0;
      slideSelectionStatus = `Showing all ${allSlides.length} slides`;
      return;
    }

    const selected = numbers
      .map((number) => allSlides.find((slide) => slide.number === number))
      .filter((slide): slide is Slide => Boolean(slide));
    const missing = numbers.filter(
      (number) => !allSlides.some((slide) => slide.number === number),
    );

    if (selected.length === 0) {
      slideSelectionStatus = `No valid slide numbers. Available: 1-${allSlides.length}`;
      return;
    }

    slides = selected;
    currentSlide = 0;
    slideSelectionStatus =
      missing.length > 0
        ? `Showing ${selected.length}; ignored ${missing.join(", ")}`
        : `Showing ${selected.length} selected slides`;
  }

  function applySlideSelection() {
    applySlideSelectionFrom(slideSelectionInput);
  }

  function showAllSlides() {
    slideSelectionInput = "";
    applySlideSelectionFrom("");
  }

  function applySlideShortcut(selection: string) {
    slideSelectionInput = selection;
    applySlideSelectionFrom(selection);
  }

  function nextSlide() {
    if (currentSlide < slides.length - 1) currentSlide++;
    else phase = "loop";
  }

  function prevSlide() {
    if (currentSlide > 0) currentSlide--;
  }

  function goToDemo() {
    phase = "demo";
  }

  function goToConvergence() {
    phase = "loop";
  }

  function goToSlides() {
    phase = "slides";
    currentSlide = 0;
  }

  function handleKeydown(e) {
    if (exampleOverlayOpen && e.key === "Escape") {
      e.preventDefault();
      closeExampleOverlay();
      return;
    }

    if (phase === "slides") {
      if (e.key === "ArrowRight" || e.key === " ") {
        e.preventDefault();
        nextSlide();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        prevSlide();
      }
    }
  }

  // ─── Editor (demo phase) ───
  const exampleLibrary = [
    {
      id: "vendor-selection",
      title: "Governed Vendor Selection",
      description: "A complete Truth with approval, constraint, evidence, and a clean scenario path.",
      tone: "ok",
      content: `Truth: Enterprise AI vendor selection is auditable, constrained, and approval-gated
  Vendor choice must be reproducible from explicit evidence.
  Final selection must stay within policy, budget, and review authority.

Intent:
  Outcome: Select a preferred AI vendor with auditable rationale.
  Goal: Evaluate candidate vendors on governance, cost, and capability.

Authority:
  Actor: governance_review_board
  Requires Approval: final_vendor_selection

Constraint:
  Cost Limit: first-year vendor spend must stay within procurement budget.
  Must Not: select a vendor without security review.

Evidence:
  Requires: security_assessment
  Requires: pricing_analysis
  Audit: decision_log

Scenario: Candidate vendors produce traceable evaluation outcomes
  Given candidate vendors "Acme AI, Beta ML, Gamma LLM"
  And each vendor has a security assessment and pricing analysis
  When the governance workflow evaluates each vendor
  Then each vendor should produce a compliance screening result
  And the system should recommend a vendor or require human review
`,
    },
    {
      id: "approval-escalation",
      title: "Approval Escalation",
      description: "Shows authority, exception handling, and a high-risk approval path.",
      tone: "ok",
      content: `Truth: High-risk vendor approvals require explicit escalation
  High-risk procurement decisions must surface accountable escalation paths.

Intent:
  Outcome: Route risky vendor approvals to the right decision makers.

Authority:
  Actor: procurement_board
  Requires Approval: executive_escalation

Constraint:
  Must Not: approve a high-risk vendor without legal review.

Evidence:
  Requires: risk_register
  Audit: escalation_log

Exception:
  Escalates To: chief_risk_officer
  Requires: signed_exception_memo

Scenario: High-risk vendors trigger escalation
  Given candidate vendor "Orion Models" is marked high risk
  And legal review is still pending
  When the governance workflow attempts final approval
  Then the decision should require executive escalation
  And the system should record the escalation path
`,
    },
    {
      id: "audit-trail",
      title: "Audit Trail",
      description: "Keeps the example compact while still exercising evidence and audit language.",
      tone: "ok",
      content: `Truth: Vendor decisions produce a durable audit trail
  Every promoted decision should be backed by traceable review evidence.

Intent:
  Outcome: Record the final decision with the evidence used to justify it.

Evidence:
  Requires: decision_record
  Requires: evidence_bundle
  Audit: vendor_decision_log

Scenario: Approved vendors generate audit artifacts
  Given vendor "Acme AI" has been approved by governance_review_board
  And the evidence bundle is attached to the decision record
  When the system records the final decision
  Then the audit log should include the approving actor and evidence bundle
`,
    },
    {
      id: "tagged-invariant",
      title: "Tagged Invariant",
      description: "Demonstrates Converge scenario tags alongside a valid acceptance-style check.",
      tone: "ok",
      content: `Truth: Vendor scorecards remain within approved bounds
  Scorecards should only surface reviewed and explainable results.

Intent:
  Outcome: Keep vendor scoring within declared governance boundaries.

Constraint:
  Must Not: publish a vendor score without provenance.

Evidence:
  Requires: provenance_bundle

@invariant @acceptance @id:vendor_score_bounds
Scenario: Published vendor scorecards carry provenance
  Given a vendor scorecard marked ready for publication
  And the provenance bundle is attached
  When the scorecard enters the acceptance gate
  Then the scorecard should remain publishable
`,
    },
    {
      id: "missing-then",
      title: "Missing Outcome",
      description: "An intentionally broken scenario that fails the local semantics/convention step.",
      tone: "warn",
      content: `Truth: Broken example missing an outcome
  This example demonstrates a failing local convention check.

Scenario: Validation stops before an assertion
  Given a shortlist of vendors
  When the governance workflow ranks the shortlist
`,
    },
    {
      id: "malformed-governance",
      title: "Malformed Governance Block",
      description: "An intentionally broken declaration that fails the syntax step immediately.",
      tone: "err",
      content: `Truth: Malformed governance block example
  This example demonstrates a syntax failure in Truth declarations.

Intent:
  Outcome Select a vendor with explicit governance.

Scenario: Parse should fail early
  Given a candidate vendor "Acme AI"
  When the validator reads the declarations
  Then the syntax step should fail
`,
    },
  ];
  const exampleSpec = exampleLibrary[0].content;

  let spec = $state(exampleSpec);
  let validation = $state(null);
  let simulation = $state(null);
  let policy = $state(null);
  let error = $state("");
  let busy = $state(false);
  let rightTab = $state("compiler"); // compiler | policy

  // ─── Compiler progress (DD-style) ───
  interface CompileStep {
    step: string;
    detail: string;
    active: boolean;
  }

  const compilePipelineSteps = [
    { step: "Simulation", detail: "Pre-flight convergence check" },
    { step: "Compilation", detail: "Full syntax and semantics validation" },
    { step: "Policy Extraction", detail: "Generating Cedar policy preview" },
  ];

  let compileSteps = $state<CompileStep[]>([]);
  let compileSpinnerVerb = $state(randomVerb());
  let compileSpinnerInterval: ReturnType<typeof setInterval> | null = null;
  let exampleOverlayOpen = $state(false);
  let truthGuidance = $state(null);
  let truthGuidanceError = $state("");
  let truthGuidanceBusy = $state(false);
  let textareaEl = $state(undefined);
  let highlightEl = $state(undefined);
  let guidanceTimer = null;
  let lastGuidanceSpec = "";
  let guidanceSequence = 0;

  function describeError(cause) {
    if (cause instanceof Error) return cause.message;
    if (typeof cause === "string") return cause;
    if (cause && typeof cause === "object") {
      if (typeof cause.message === "string" && cause.message.length > 0) return cause.message;
      try { return JSON.stringify(cause); } catch { return "Validation failed."; }
    }
    return "Validation failed.";
  }

  function advanceCompileStep(index: number) {
    compileSteps = [
      ...compileSteps.map((s) => ({ ...s, active: false })),
      { step: compilePipelineSteps[index].step, detail: compilePipelineSteps[index].detail, active: true },
    ];
  }

  function completeCompileStep() {
    compileSteps = compileSteps.map((s) => ({ ...s, active: false }));
  }

  async function validate() {
    busy = true;
    error = "";
    simulation = null;
    compileSteps = [];
    compileSpinnerVerb = randomVerb();
    compileSpinnerInterval = setInterval(() => { compileSpinnerVerb = randomVerb(); }, 2000);

    try {
      // Step 1: Simulate (pre-flight)
      advanceCompileStep(0);
      simulation = await invokeTauri("simulate_truth", { spec });

      // Step 2: Validate (full compiler)
      advanceCompileStep(1);
      validation = await invokeTauri("validate_gherkin", { spec });

      // Step 3: Extract policy (Cedar preview)
      advanceCompileStep(2);
      policy = await invokeTauri("extract_policy", { spec });

      completeCompileStep();
    } catch (cause) {
      validation = null;
      error = describeError(cause);
    } finally {
      busy = false;
      if (compileSpinnerInterval) { clearInterval(compileSpinnerInterval); compileSpinnerInterval = null; }
    }
  }

  function openExampleOverlay() {
    exampleOverlayOpen = true;
  }

  function closeExampleOverlay() {
    exampleOverlayOpen = false;
  }

  function loadExample(example = exampleLibrary[0]) {
    spec = example.content;
    error = "";
    validation = null;
    closeExampleOverlay();
  }

  function clearGuidanceTimer() {
    if (guidanceTimer) { clearTimeout(guidanceTimer); guidanceTimer = null; }
  }

  function extractTruthHeading(value) {
    const match = value.match(/^\s*(Truth|Feature):\s*(.+)$/m);
    return match ? match[2].trim() : "";
  }

  function replaceTruthHeading(value, suggestedTitle) {
    return value.replace(
      /^(\s*)(Truth|Feature):\s*(.+)$/m,
      (_, indent) => `${indent}Truth: ${suggestedTitle}`
    );
  }

  async function requestTruthGuidance(snapshot) {
    const requestId = ++guidanceSequence;
    truthGuidanceBusy = true;
    truthGuidanceError = "";
    try {
      const response = await invokeTauri("guide_truth_heading", { spec: snapshot });
      if (requestId !== guidanceSequence) return;
      truthGuidance = response;
    } catch (cause) {
      if (requestId !== guidanceSequence) return;
      truthGuidance = null;
      truthGuidanceError = describeError(cause);
    } finally {
      if (requestId === guidanceSequence) truthGuidanceBusy = false;
    }
  }

  function scheduleTruthGuidance(nextSpec) {
    if (nextSpec === lastGuidanceSpec) return;
    lastGuidanceSpec = nextSpec;
    clearGuidanceTimer();
    if (!extractTruthHeading(nextSpec)) {
      truthGuidance = null;
      truthGuidanceError = "";
      truthGuidanceBusy = false;
      return;
    }
    guidanceTimer = setTimeout(() => requestTruthGuidance(nextSpec), 650);
  }

  function applyTruthSuggestion() {
    if (!truthGuidance?.shouldRewrite || !truthGuidance?.suggestedTitle) return;
    spec = replaceTruthHeading(spec, truthGuidance.suggestedTitle);
    validation = null;
    error = "";
  }

  function syncScroll() {
    if (textareaEl && highlightEl) {
      highlightEl.scrollTop = textareaEl.scrollTop;
      highlightEl.scrollLeft = textareaEl.scrollLeft;
    }
  }

  function escapeHtml(value) {
    return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
  }

  function highlightInline(value) {
    let html = escapeHtml(value);
    html = html.replace(/`([^`]+)`/g, '<span class="token inline-code">`$1`</span>');
    html = html.replace(/"([^"]*)"/g, '<span class="token string">"$1"</span>');
    html = html.replace(
      /\b(should|must|always|require|requires|approval|human review)\b/gi,
      '<span class="token modal">$1</span>'
    );
    return html;
  }

  function wrapLine(lineNumber, classes, content) {
    return `<span class="editor-row ${classes}"><span class="line-number-cell">${lineNumber}</span><span class="line-code-cell">${content || "&nbsp;"}</span></span>`;
  }

  function renderTags(rawLine) {
    return rawLine.trim().split(/\s+/).map((tag) => `<span class="token tag">${escapeHtml(tag)}</span>`).join(" ");
  }

  function renderTable(rawLine, lineNumber) {
    const cells = rawLine.split("|").map((cell) => cell.trim());
    const rendered = cells.map((cell, index) => {
      if (index === 0 || index === cells.length - 1) return '<span class="token punctuation">|</span>';
      return `<span class="token table-cell">${highlightInline(` ${cell} `)}</span><span class="token punctuation">|</span>`;
    }).join("");
    return wrapLine(lineNumber, "line-table", rendered);
  }

  function renderStructuredLine(rawLine, index) {
    const lineNumber = index + 1;
    if (rawLine.length === 0) return wrapLine(lineNumber, "line-empty", "");
    if (/^\s*#/.test(rawLine)) return wrapLine(lineNumber, "line-comment", `<span class="token comment">${escapeHtml(rawLine)}</span>`);

    const featureMatch = rawLine.match(/^(\s*)(Truth|Feature)(:)(.*)$/);
    if (featureMatch) {
      const [, indent, keyword, punctuation, rest] = featureMatch;
      return wrapLine(lineNumber, "line-feature", `${indent}<span class="token keyword keyword-feature">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`);
    }

    const governanceMatch = rawLine.match(/^(\s*)(Intent|Authority|Constraint|Evidence|Exception)(:)(.*)$/);
    if (governanceMatch) {
      const [, indent, keyword, punctuation, rest] = governanceMatch;
      return wrapLine(lineNumber, "line-governance", `${indent}<span class="token keyword keyword-governance">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`);
    }

    const scenarioMatch = rawLine.match(/^(\s*)(Scenario(?: Outline)?)(:)(.*)$/);
    if (scenarioMatch) {
      const [, indent, keyword, punctuation, rest] = scenarioMatch;
      return wrapLine(lineNumber, "line-scenario", `${indent}<span class="token keyword keyword-scenario">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`);
    }

    const stepMatch = rawLine.match(/^(\s*)(Given|When|Then|And|But)(\s+)(.*)$/);
    if (stepMatch) {
      const [, indent, keyword, spacer, rest] = stepMatch;
      return wrapLine(lineNumber, "line-step", `${indent}<span class="token keyword keyword-step">${keyword}</span>${spacer}${highlightInline(rest)}`);
    }

    if (/^\s*@/.test(rawLine)) {
      const indent = rawLine.match(/^\s*/)?.[0] ?? "";
      return wrapLine(lineNumber, "line-tags", `${indent}${renderTags(rawLine)}`);
    }

    if (/^\s*\|/.test(rawLine)) return renderTable(rawLine, lineNumber);
    return wrapLine(lineNumber, "line-plain", highlightInline(rawLine));
  }

  function renderHighlightedSpec(value) {
    return value.split("\n").map((line, index) => renderStructuredLine(line, index)).join("");
  }

  function stepStatusLabel(status) {
    if (status === "ok") return "OK";
    if (status === "issue") return "Needs Work";
    return "Unavailable";
  }

  // ─── Derived ───
  let specName = $derived(extractTruthHeading(spec) || "Untitled Spec");
  let validationSteps = $derived(validation?.steps ?? []);
  let syntaxPassed = $derived(validationSteps[0]?.status === "ok");
  let highlightedSpec = $derived(renderHighlightedSpec(spec));

  $effect(() => { scheduleTruthGuidance(spec); });

  onDestroy(() => {
    guidanceSequence += 1;
    clearGuidanceTimer();
    if (compileSpinnerInterval) { clearInterval(compileSpinnerInterval); compileSpinnerInterval = null; }
  });

  // ─── Applications phase ───

  interface AppCard {
    id: string;
    title: string;
    description: string;
    status: "active" | "soon";
    icon: string;
    action?: () => void;
  }

  const appCards: AppCard[] = [
    {
      id: "convergence-loop",
      title: "Vendor Selection Demos",
      description: "Run Today Governed Selection or Creative Pareto Breakout with HITL, Cedar, and learning gates.",
      status: "active",
      icon: "◎",
      action: () => { phase = "loop"; },
    },
    {
      id: "truth-editor",
      title: "Truth Spec Editor",
      description: "Write and validate governance specs. Syntax checking, policy extraction, Cedar generation.",
      status: "active",
      icon: "📝",
      action: () => { phase = "demo"; },
    },
    {
      id: "due-diligence",
      title: "Due Diligence",
      description: "Convergent research on any company. Web search, fact extraction, contradiction detection, structured analysis.",
      status: "active",
      icon: "🔍",
      action: () => { phase = "dd"; },
    },
    {
      id: "governance-audit",
      title: "Governance Audit",
      description: "Scan existing processes for compliance gaps, missing authority, and unstructured decisions.",
      status: "soon",
      icon: "🛡️",
    },
    {
      id: "policy-library",
      title: "Policy Library",
      description: "Browse and compose reusable governance patterns. Cedar policies, approval gates, constraints.",
      status: "soon",
      icon: "📚",
    },
  ];

  function goToApps() {
    phase = "apps";
  }

  // ─── Due Diligence phase ───

  interface DdFlowItem {
    id: string;
    label: string;
    detail: string;
    category: string | null;
    confidence: number | null;
  }

  interface DdFlowStage {
    label: string;
    kind: string;
    items: DdFlowItem[];
  }

  interface DdFlowReport {
    company_name: string;
    focus_areas: string[];
    executive_summary: string;
    market_analysis: string[];
    competitive_landscape: string[];
    technology_assessment: string[];
    ownership_and_financials: string[];
    contradictions: string[];
    remaining_gaps: string[];
    recommendation: string;
    confidence: number;
    needs_human_review: boolean;
  }

  interface DdFlowResult {
    converged: boolean;
    cycles: number;
    stop_reason: string;
    stages: DdFlowStage[];
    report: DdFlowReport;
  }

  const stageIcons: Record<string, string> = {
    formation: "\u25C6",
    breadth: "\u2192",
    depth: "\u2193",
    extraction: "\u25A0",
    contradiction: "\u26A0",
    gap: "\u21BB",
    synthesis: "\u2605",
  };

  const stageColors: Record<string, string> = {
    formation: "var(--color-info, #60a5fa)",
    breadth: "var(--color-ok, #34d399)",
    depth: "var(--color-ok, #34d399)",
    extraction: "var(--color-lime, #a3e635)",
    contradiction: "var(--color-warn, #fbbf24)",
    gap: "var(--color-info, #60a5fa)",
    synthesis: "var(--color-lime, #a3e635)",
  };

  let ddCompanyName = $state("");
  let ddFlow = $state<DdFlowResult | null>(null);
  let ddLoading = $state(false);
  let ddError = $state("");
  let ddRevealedStages = $state(0);
  let ddRevealedItems = $state(0);
  let ddShowReport = $state(false);
  let ddPrintTimestamp = $state("");
  let ddRevealTimers: ReturnType<typeof setTimeout>[] = [];

  let ddRevealing = $derived(ddFlow !== null && !ddShowReport && ddRevealedStages <= (ddFlow?.stages.length ?? 0));
  let ddProgress = $derived(() => {
    if (!ddFlow || ddFlow.stages.length === 0) return 0;
    const totalItems = ddFlow.stages.reduce((sum, s) => sum + s.items.length, 0);
    let revealed = 0;
    for (let i = 0; i < ddRevealedStages && i < ddFlow.stages.length; i++) {
      revealed += ddFlow.stages[i].items.length;
    }
    if (ddRevealedStages < ddFlow.stages.length) {
      revealed += Math.min(ddRevealedItems, ddFlow.stages[ddRevealedStages]?.items.length ?? 0);
    }
    return Math.round((revealed / totalItems) * 100);
  });

  async function runDd() {
    if (!ddCompanyName.trim()) return;
    ddLoading = true;
    ddError = "";
    ddFlow = null;
    ddShowReport = false;
    ddRevealedStages = 0;
    ddRevealedItems = 0;
    ddRevealTimers.forEach(clearTimeout);
    ddRevealTimers = [];

    try {
      const result = await invokeTauri<DdFlowResult>("run_due_diligence", {
        companyName: ddCompanyName.trim(),
        productName: null,
        focusAreas: [],
      });
      ddFlow = result;
      ddLoading = false;
      startFlowReveal();
    } catch (e) {
      ddError = String(e);
      ddLoading = false;
    }
  }

  function startFlowReveal() {
    if (!ddFlow || ddFlow.stages.length === 0) {
      ddShowReport = true;
      return;
    }
    ddRevealedStages = 0;
    ddRevealedItems = 0;
    revealNextItem();
  }

  function revealNextItem() {
    if (!ddFlow) return;
    const stages = ddFlow.stages;

    if (ddRevealedStages >= stages.length) {
      const t = setTimeout(() => { ddShowReport = true; }, 800);
      ddRevealTimers.push(t);
      return;
    }

    const currentStage = stages[ddRevealedStages];
    if (ddRevealedItems < currentStage.items.length) {
      ddRevealedItems++;
      const delay = currentStage.kind === "extraction" ? 150 : 250;
      const t = setTimeout(revealNextItem, delay);
      ddRevealTimers.push(t);
    } else {
      ddRevealedStages++;
      ddRevealedItems = 0;
      const t = setTimeout(revealNextItem, 600);
      ddRevealTimers.push(t);
    }
  }

  function ddNewSearch() {
    ddFlow = null;
    ddShowReport = false;
    ddRevealedStages = 0;
    ddRevealedItems = 0;
    ddError = "";
    ddPrintTimestamp = "";
    ddRevealTimers.forEach(clearTimeout);
    ddRevealTimers = [];
  }

  function printDdReport() {
    if (!ddFlow) return;
    ddPrintTimestamp = new Date().toLocaleString();
    requestAnimationFrame(() => window.print());
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<svelte:head>
  <title>Converge Governance</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=Roboto+Mono:wght@400;500&family=Space+Grotesk:wght@400;500;600;700&display=swap" rel="stylesheet" />
</svelte:head>

<!-- ═══════════════════════════════════════════════
     SLIDES PHASE
     ═══════════════════════════════════════════════ -->
{#if phase === "slides"}
  <div class="fixed inset-0 bg-void">
    <!-- Background image -->
    <div class="absolute inset-0">
      {#each slides as slide, i}
        <div
          class="absolute inset-0 transition-opacity duration-700"
          style="opacity: {i === currentSlide ? 1 : 0}"
        >
          <img
            src={slide.image}
            alt=""
            class="h-full w-full object-cover"
          />
          <!-- Dark overlay -->
          <div class="absolute inset-0 bg-gradient-to-r from-void/90 via-void/70 to-void/40"></div>
          <div class="absolute inset-0 bg-gradient-to-t from-void/60 via-transparent to-void/30"></div>
        </div>
      {/each}
    </div>

    <!-- Slide content -->
    <div class="relative z-10 flex h-full flex-col justify-between p-12">
      <!-- Top bar -->
      <div class="flex items-start justify-between gap-6">
        <div>
          <span class="font-mono text-xs tracking-widest text-muted uppercase">Converge Governance</span>
          <div class="mt-2 flex flex-wrap items-center gap-2">
            <span class="font-mono text-[0.68rem] tracking-widest text-muted uppercase">Deck</span>
            <input
              class="slide-picker"
              aria-label="Slide numbers to present"
              placeholder="1,2,3,8,12"
              bind:value={slideSelectionInput}
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  applySlideSelection();
                }
              }}
            />
            <button class="btn-ghost px-3! py-2! text-xs!" onclick={applySlideSelection}>
              Use
            </button>
            <button class="btn-ghost px-3! py-2! text-xs!" onclick={showAllSlides}>
              All
            </button>
            <button class="btn-ghost px-3! py-2! text-xs!" onclick={() => applySlideShortcut(microDeckSelection)}>
              Micro
            </button>
            <button class="btn-ghost px-3! py-2! text-xs!" onclick={() => applySlideShortcut(shortDeckSelection)}>
              Short
            </button>
            <button class="btn-ghost px-3! py-2! text-xs!" onclick={() => applySlideShortcut(longerDeckSelection)}>
              Longer
            </button>
            {#if slideSelectionStatus}
              <span class="font-mono text-[0.68rem] text-muted">{slideSelectionStatus}</span>
            {/if}
          </div>
        </div>
        <button class="btn-ghost text-sm" onclick={goToConvergence}>
          Skip to Demo &rarr;
        </button>
      </div>

      <!-- Main content -->
      {#key currentSlide}
        {#if slides[currentSlide].layout}
          <div class="fade-in max-w-[min(92vw,86rem)]">
            <div class="grid items-end gap-10 lg:grid-cols-[minmax(0,0.8fr)_minmax(30rem,1fr)]">
              <div class="max-w-3xl">
                <p class="slide-eyebrow mb-4">
                  <span class="mr-3 text-subtle">#{displaySlideNumber(currentSlide)}</span>{slides[currentSlide].eyebrow}
                </p>
                <h1 class="slide-headline mb-6 whitespace-pre-line">{slides[currentSlide].headline}</h1>
                <p class="slide-body">{slides[currentSlide].body}</p>

                {#if currentSlide === slides.length - 1}
                  <div class="mt-10 flex gap-4">
                    <button class="btn-lime" onclick={goToConvergence}>
                      Launch Demo
                    </button>
                    <button class="btn-ghost" onclick={goToDemo}>
                      Spec Studio
                    </button>
                  </div>
                {/if}
              </div>

              {#if slides[currentSlide].layout === "stack"}
                <div class="stack-diagram">
                  {#each slides[currentSlide].stackLayers ?? [] as layer}
                    <div class="stack-layer">
                      <div>
                        <span class="stack-layer-name">{layer.name}</span>
                        <span class="stack-layer-role">{layer.role}</span>
                      </div>
                      <p>{layer.detail}</p>
                    </div>
                  {/each}
                </div>
              {:else if slides[currentSlide].layout === "flow"}
                <div class="flow-diagram">
                  {#each slides[currentSlide].flowSteps ?? [] as step, i}
                    <div class="flow-step">
                      <span class="flow-step-index">{String(i + 1).padStart(2, "0")}</span>
                      <div>
                        <strong>{step.name}</strong>
                        <p>{step.detail}</p>
                      </div>
                    </div>
                  {/each}
                </div>
              {:else if slides[currentSlide].layout === "gateway"}
                <div class="gateway-diagram">
                  {#each slides[currentSlide].gatewayPoints ?? [] as point}
                    <div class="gateway-point">
                      <span>{point.name}</span>
                      <p>{point.detail}</p>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <div class="fade-in max-w-3xl">
            <p class="slide-eyebrow mb-4">
              <span class="mr-3 text-subtle">#{displaySlideNumber(currentSlide)}</span>{slides[currentSlide].eyebrow}
            </p>
            <h1 class="slide-headline mb-6 whitespace-pre-line">{slides[currentSlide].headline}</h1>
            <p class="slide-body">{slides[currentSlide].body}</p>

            {#if currentSlide === slides.length - 1}
              <div class="mt-10 flex gap-4">
                <button class="btn-lime" onclick={goToConvergence}>
                  Launch Demo
                </button>
                <button class="btn-ghost" onclick={goToDemo}>
                  Spec Studio
                </button>
              </div>
            {/if}
          </div>
        {/if}
      {/key}

      <!-- Bottom nav -->
      <div class="flex items-center justify-between">
        <div class="flex max-w-[58vw] flex-wrap items-center gap-2">
          {#each slides as slide, i}
            <button
              class="nav-dot"
              class:active={i === currentSlide}
              onclick={() => (currentSlide = i)}
              aria-label="Go to slide {displaySlideNumber(i)}"
              title="Slide {displaySlideNumber(i)} (source {slide.number})"
            ></button>
          {/each}
        </div>

        <div class="flex items-center gap-3">
          <span class="font-mono text-xs text-muted">
            #{displaySlideNumber(currentSlide)} · {currentSlide + 1} / {slides.length} selected · {allSlides.length} total
          </span>
          <button
            class="btn-ghost text-sm"
            onclick={prevSlide}
            disabled={currentSlide === 0}
          >
            &larr;
          </button>
          <button
            class="btn-primary text-sm"
            onclick={nextSlide}
          >
            {currentSlide === slides.length - 1 ? "Start Demo" : "Next"} &rarr;
          </button>
          <button class="btn-ghost text-sm" onclick={goToApps}>
            Apps
          </button>
        </div>
      </div>
    </div>
  </div>

<!-- ═══════════════════════════════════════════════
     PROVIDER SELECTION PHASE
     ═══════════════════════════════════════════════ -->
{:else if phase === "providers"}
  <div class="min-h-screen bg-void">
    <!-- Top bar -->
    <header class="flex items-center justify-between border-b border-border px-8 py-4">
      <div class="flex items-center gap-4">
        <button class="btn-ghost text-sm" onclick={goToSlides}>
          &larr; Slides
        </button>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Provider Setup</span>
      </div>
    </header>

    <!-- Provider selector -->
    <div class="p-8">
      <ProviderSelector on:provider-selection-complete={(e) => { phase = "loop"; }} />
    </div>
  </div>

<!-- ═══════════════════════════════════════════════
     CONVERGENCE LOOP PHASE
     ═══════════════════════════════════════════════ -->
{:else if phase === "loop"}
  <AIProviderEvaluation onBack={goToSlides} onApps={goToApps} onSpecStudio={goToDemo} />

<!-- ═══════════════════════════════════════════════
     DEMO PHASE
     ═══════════════════════════════════════════════ -->
{:else if phase === "demo"}
  <div class="min-h-screen bg-void">
    <!-- Top bar -->
    <header class="flex items-center justify-between border-b border-border px-8 py-4">
      <div class="flex items-center gap-4">
        <button class="btn-ghost text-sm" onclick={goToSlides}>
          &larr; Slides
        </button>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Spec Studio</span>
      </div>
      <div class="flex items-center gap-2">
        <button class="btn-ghost" type="button" onclick={goToApps}>Apps</button>
        <button class="btn-ghost" type="button" onclick={openExampleOverlay}>Browse Examples</button>
      </div>
    </header>

    {#if exampleOverlayOpen}
      <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-void/84 px-4 py-8 backdrop-blur-sm"
        role="presentation"
        onclick={closeExampleOverlay}
      >
        <div
          class="max-h-[90vh] w-full max-w-6xl overflow-auto rounded-[28px] border border-border bg-deep p-6 shadow-2xl"
          role="dialog"
          aria-modal="true"
          aria-labelledby="example-library-title"
          tabindex="-1"
          onclick={(event) => event.stopPropagation()}
          onkeydown={(event) => event.key === "Escape" && closeExampleOverlay()}
        >
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div class="max-w-2xl">
              <p class="card-label">Example Library</p>
              <h2 id="example-library-title" class="mt-2 font-display text-2xl font-semibold text-bright">
                Load a richer starting point
              </h2>
              <p class="mt-2 text-sm text-subtle">
                Pick a clean example, a warning case, or a syntax failure to see how the validator stages behave.
              </p>
            </div>
            <button class="btn-ghost text-sm" type="button" onclick={closeExampleOverlay}>Close</button>
          </div>

          <div class="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {#each exampleLibrary as example}
              <button
                type="button"
                class="rounded-2xl border border-border bg-raised p-4 text-left transition hover:-translate-y-0.5 hover:border-subtle hover:bg-surface"
                onclick={() => loadExample(example)}
              >
                <div class="flex items-start justify-between gap-3">
                  <strong class="font-display text-base text-bright">{example.title}</strong>
                  <span
                    class="pill"
                    class:pill-ok={example.tone === "ok"}
                    class:pill-warn={example.tone === "warn"}
                    class:pill-err={example.tone === "err"}
                  >
                    {example.tone === "ok" ? "Clean" : example.tone === "warn" ? "Needs Work" : "Syntax Fail"}
                  </span>
                </div>
                <p class="mt-2 text-sm text-subtle">{example.description}</p>
                <p class="mt-4 font-mono text-xs text-muted">
                  {example.content.split("\n")[0]}
                </p>
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Workspace -->
    <div class="mx-auto grid max-w-[1600px] gap-5 p-6" style="grid-template-columns: minmax(0, 1.6fr) minmax(300px, 0.8fr)">

      <!-- Editor + Guidance below -->
      <div class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="card-label">Truth Spec</span>
            <span class="font-display text-sm text-bright">{specName}</span>
          </div>
          <button class="btn-lime text-sm" type="button" onclick={validate} disabled={busy}>
            {#if busy}Validating&hellip;{:else}Validate{/if}
          </button>
        </div>
        <div class="editor-surface">
          <div class="editor-stack">
            <div class="highlight-layer" bind:this={highlightEl} aria-hidden="true">
              {@html highlightedSpec}
            </div>
            {#if !spec.trim()}
              <div class="editor-empty">Paste vendor-selection Gherkin or Truth syntax here.</div>
            {/if}
            <textarea bind:this={textareaEl} bind:value={spec} spellcheck="false" oninput={() => syncScroll()} onscroll={syncScroll}></textarea>
          </div>
        </div>

        <!-- Truth Guidance (below editor) -->
        <section class="flex flex-col gap-3">
          {#if truthGuidanceBusy}
            <div class="callout callout-neutral">
              <strong>Evaluating heading&hellip;</strong>
            </div>
          {:else if truthGuidanceError}
            <div class="callout callout-error">
              <strong>Guidance error</strong>
              <p>{truthGuidanceError.length > 120 ? truthGuidanceError.slice(0, 120) + '...' : truthGuidanceError}</p>
            </div>
          {:else if truthGuidance}
            <div class="callout callout-lime">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <strong>{truthGuidance.shouldRewrite ? "Rewrite suggested" : "Heading looks good"}</strong>
                  <p>{truthGuidance.note}</p>
                </div>
                <div class="flex shrink-0 items-center gap-2">
                  <span
                    class="pill"
                    class:pill-info={truthGuidance.source === "live-chat-backend"}
                    class:pill-warn={truthGuidance.source !== "live-chat-backend"}
                  >
                    {truthGuidance.sourceLabel}
                  </span>
                  {#if truthGuidance.shouldRewrite}
                    <button class="btn-ghost text-xs" type="button" onclick={applyTruthSuggestion}>
                      Apply
                    </button>
                  {/if}
                </div>
              </div>
            </div>

            {#if truthGuidance.shouldRewrite}
              <div class="grid grid-cols-2 gap-3">
                <article class="rounded-lg border border-border bg-raised p-3">
                  <span class="text-xs text-muted">Current</span>
                  <strong class="mt-1 block text-sm text-bright">{truthGuidance.currentTitle}</strong>
                </article>
                <article class="rounded-lg border border-border bg-raised p-3">
                  <span class="text-xs text-muted">Suggested</span>
                  <strong class="mt-1 block text-sm text-lime">{truthGuidance.suggestedTitle}</strong>
                </article>
              </div>
            {/if}

            {#if truthGuidance.rationale.length > 0}
              <div class="flex flex-wrap gap-2">
                {#each truthGuidance.rationale as reason}
                  <span class="rounded-lg border border-border bg-surface px-3 py-1.5 text-xs text-subtle">{reason}</span>
                {/each}
              </div>
            {/if}

            {#if truthGuidance.descriptionHints.length > 0}
              <div class="flex flex-wrap gap-2">
                {#each truthGuidance.descriptionHints as hint}
                  <span class="rounded-lg border border-lime/20 bg-lime-glow px-3 py-1.5 text-xs text-subtle">{hint}</span>
                {/each}
              </div>
            {/if}
          {:else}
            <p class="text-xs text-muted">Add a <code class="font-mono text-lime">Truth:</code> line for live heading guidance.</p>
          {/if}
        </section>
      </div>

      <!-- Right panel -->
      <aside class="flex flex-col gap-4">
        <!-- Tab bar -->
        <div class="flex items-center gap-1 border-b border-border pb-2">
          <button
            class="rounded-lg px-3 py-1.5 text-xs font-medium transition"
            class:bg-raised={rightTab === "compiler"}
            class:text-bright={rightTab === "compiler"}
            class:text-muted={rightTab !== "compiler"}
            onclick={() => (rightTab = "compiler")}
          >Compiler</button>
          <button
            class="rounded-lg px-3 py-1.5 text-xs font-medium transition"
            class:bg-raised={rightTab === "policy"}
            class:text-bright={rightTab === "policy"}
            class:text-muted={rightTab !== "policy"}
            onclick={() => (rightTab = "policy")}
          >Policy</button>

          {#if simulation}
            <span class="ml-auto pill"
              class:pill-ok={simulation.verdict === "ready"}
              class:pill-warn={simulation.verdict === "risky"}
              class:pill-err={simulation.verdict === "will-not-converge"}
            >
              {simulation.verdict === "ready" ? "Can Converge" : simulation.verdict === "risky" ? "Risky" : "Won't Converge"}
            </span>
          {:else if validation}
            <span class="ml-auto pill" class:pill-ok={validation.isValid} class:pill-err={!validation.isValid}>
              {validation.isValid ? "Pass" : "Fail"}
            </span>
          {/if}
        </div>

        {#if rightTab === "compiler"}
          <!-- ─── COMPILER TAB ─── -->
          {#if busy}
            <!-- DD-style progress -->
            <div class="flex flex-col gap-4">
              <div class="space-y-3">
                {#each compileSteps as step}
                  <div class="flex items-start gap-3 fade-in">
                    <div class="mt-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-full"
                      class:bg-lime-glow={step.active} class:text-lime={step.active}
                      style={!step.active ? "background: rgba(52,211,153,0.15); color: var(--color-ok)" : ""}>
                      {#if step.active}
                        <span class="inline-block h-2 w-2 animate-pulse rounded-full bg-lime"></span>
                      {:else}
                        <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                      {/if}
                    </div>
                    <div>
                      <div class="text-sm font-medium" class:text-bright={step.active} class:text-subtle={!step.active}>{step.step}</div>
                      <div class="text-xs text-muted">{step.detail}</div>
                    </div>
                  </div>
                {/each}
              </div>

              <div class="h-1 overflow-hidden rounded-full bg-border">
                <div class="h-full rounded-full bg-lime transition-all duration-1000"
                  style="width: {Math.max(5, (compileSteps.filter(s => !s.active).length / compilePipelineSteps.length) * 100)}%"></div>
              </div>

              <p class="text-center text-sm text-muted">{compileSpinnerVerb}...</p>
            </div>
          {:else if error}
            <div class="callout callout-error">
              <strong>Error</strong>
              <p>{error.length > 200 ? error.slice(0, 200) + '...' : error}</p>
            </div>
          {:else if simulation && !simulation.canConverge}
            <!-- Simulation failed — show findings before compiler -->
            <div class="callout callout-error">
              <strong>Simulation: won't converge</strong>
              <p>The spec is underspecified. Fix these before validating.</p>
            </div>
            <div class="flex flex-col gap-1">
              {#each simulation.findings.filter(f => f.severity === "error") as finding}
                <div class="flex items-start gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                  <span class="mt-0.5 h-2 w-2 shrink-0 rounded-full bg-err"></span>
                  <div class="min-w-0">
                    <span class="text-sm text-bright">{finding.message}</span>
                    {#if finding.suggestion}
                      <p class="mt-0.5 text-xs text-subtle">{finding.suggestion}</p>
                    {/if}
                  </div>
                </div>
              {/each}
              {#each simulation.findings.filter(f => f.severity === "warning") as finding}
                <div class="flex items-start gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                  <span class="mt-0.5 h-2 w-2 shrink-0 rounded-full bg-warn"></span>
                  <div class="min-w-0">
                    <span class="text-sm text-bright">{finding.message}</span>
                  </div>
                </div>
              {/each}
            </div>
          {:else if validation}
            <!-- Simulation passed — show compiler results -->
            {#if simulation && simulation.findings.length > 0}
              <div class="flex flex-wrap gap-1">
                {#each simulation.findings as finding}
                  <span class="rounded px-2 py-0.5 text-xs {finding.severity === 'warning' ? 'bg-raised text-warn' : 'bg-raised text-muted'}"
                  >{finding.message.length > 60 ? finding.message.slice(0, 60) + '...' : finding.message}</span>
                {/each}
              </div>
            {/if}

            <!-- Steps -->
            <div class="flex flex-col gap-2">
              {#each validationSteps as step}
                <div class="flex items-center gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                  <span class="h-2 w-2 shrink-0 rounded-full"
                    class:bg-ok={step.status === "ok"}
                    class:bg-warn={step.status === "issue"}
                    class:bg-muted={step.status === "unavailable"}
                  ></span>
                  <span class="text-sm text-bright">{step.label}</span>
                  <span class="ml-auto text-xs text-muted">{stepStatusLabel(step.status)}</span>
                </div>
              {/each}
            </div>

            <!-- Governance flags -->
            {#if syntaxPassed}
              <div class="grid grid-cols-2 gap-2">
                {#each ["intent", "authority", "constraint", "evidence"] as key}
                  <div class="flex items-center gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                    <span class="h-2 w-2 shrink-0 rounded-full" class:bg-ok={validation.governance[key]} class:bg-err={!validation.governance[key]}></span>
                    <span class="text-xs capitalize text-subtle">{key}</span>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Scenarios -->
            {#if validation.scenarios.length > 0}
              <div class="flex flex-col gap-1">
                <span class="card-label">Scenarios</span>
                {#each validation.scenarios as scenario}
                  <div class="flex items-center justify-between rounded-lg border border-border bg-surface px-3 py-2">
                    <span class="truncate text-sm text-bright">{scenario.name}</span>
                    <span class="shrink-0 text-xs text-muted">{scenario.kind ?? ""}</span>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Issues -->
            {#if validation.issues?.length > 0}
              <div class="flex flex-col gap-1">
                <span class="card-label">Issues ({validation.issues.length})</span>
                {#each validation.issues.slice(0, 5) as issue}
                  <div class="issue" data-severity={issue.severity}>
                    <div class="flex items-center justify-between gap-2">
                      <span class="truncate text-sm text-bright">{issue.message}</span>
                      <span class="shrink-0 text-xs uppercase text-muted">{issue.severity}</span>
                    </div>
                    {#if issue.suggestion}
                      <p class="mt-1 truncate text-xs text-subtle">{issue.suggestion}</p>
                    {/if}
                  </div>
                {/each}
                {#if validation.issues.length > 5}
                  <p class="text-xs text-muted">+{validation.issues.length - 5} more</p>
                {/if}
              </div>
            {/if}
          {:else}
            <div class="callout callout-neutral">
              <strong>Ready</strong>
              <p>Click <em class="text-lime">Validate</em> to simulate, compile, and extract policy.</p>
            </div>
          {/if}

        {:else}
          <!-- ─── POLICY TAB ─── -->
          {#if policy}
            <div class="flex flex-col gap-3">
              <!-- Gated actions -->
              {#if policy.gatedActions.length > 0}
                <div class="flex flex-col gap-1">
                  <span class="card-label">Gated Actions</span>
                  {#each policy.gatedActions as gate}
                    <div class="flex items-start gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                      <span class="mt-0.5 shrink-0 font-mono text-xs text-lime">{gate.action}</span>
                      <span class="text-xs text-subtle">{gate.reason}</span>
                    </div>
                  {/each}
                </div>
              {/if}

              <!-- Required gates -->
              {#if policy.requiredGates.length > 0}
                <div class="flex flex-col gap-1">
                  <span class="card-label">Required Evidence Gates</span>
                  <div class="flex flex-wrap gap-2">
                    {#each policy.requiredGates as gate}
                      <span class="rounded-lg border border-lime/20 bg-lime-glow px-3 py-1 font-mono text-xs text-bright">{gate}</span>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Flags -->
              <div class="grid grid-cols-2 gap-2">
                <div class="flex items-center gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                  <span class="h-2 w-2 shrink-0 rounded-full" class:bg-lime={policy.requiresHumanApproval} class:bg-muted={!policy.requiresHumanApproval}></span>
                  <span class="text-xs text-subtle">Human Approval</span>
                </div>
                {#if policy.authorityLevel}
                  <div class="flex items-center gap-2 rounded-lg border border-border bg-raised px-3 py-2">
                    <span class="h-2 w-2 shrink-0 rounded-full bg-lime"></span>
                    <span class="text-xs text-subtle">{policy.authorityLevel}</span>
                  </div>
                {/if}
              </div>

              <!-- Spending limits -->
              {#if policy.spendingLimits.length > 0}
                <div class="flex flex-col gap-1">
                  <span class="card-label">Spending Limits</span>
                  {#each policy.spendingLimits as limit}
                    <span class="text-xs text-subtle">{limit}</span>
                  {/each}
                </div>
              {/if}

              <!-- Escalation -->
              {#if policy.escalationTargets.length > 0}
                <div class="flex flex-col gap-1">
                  <span class="card-label">Escalation Path</span>
                  <div class="flex flex-wrap gap-2">
                    {#each policy.escalationTargets as target}
                      <span class="rounded-lg border border-warn/20 bg-warn/5 px-3 py-1 font-mono text-xs text-warn">{target}</span>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Cedar preview -->
              <div class="flex flex-col gap-1">
                <span class="card-label">Generated Cedar Policy</span>
                <pre class="overflow-auto rounded-lg border border-border bg-deep p-3 font-mono text-xs leading-relaxed text-subtle">{policy.cedarPreview}</pre>
              </div>
            </div>
          {:else}
            <div class="callout callout-neutral">
              <strong>No policy yet</strong>
              <p>Click <em class="text-lime">Validate</em> to extract the implied Cedar policy from your Truth's governance blocks.</p>
            </div>
          {/if}
        {/if}
      </aside>
    </div>
  </div>

<!-- ═══════════════════════════════════════════════
     APPS PHASE
     ═══════════════════════════════════════════════ -->
{:else if phase === "apps"}
  <div class="min-h-screen bg-void">
    <header class="flex items-center justify-between border-b border-border px-8 py-4">
      <div class="flex items-center gap-4">
        <button class="btn-ghost text-sm" onclick={goToSlides}>&larr; Slides</button>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Converge Platform</span>
      </div>
      <button class="btn-ghost text-sm" onclick={goToDemo}>Spec Studio</button>
    </header>

    <div class="relative">
      <div class="absolute inset-0 h-64">
        <img src="/images/slides/earth.jpg" alt="" class="h-full w-full object-cover opacity-20" />
        <div class="absolute inset-0 bg-gradient-to-b from-void/80 via-void/90 to-void"></div>
      </div>

      <div class="relative z-10 mx-auto max-w-4xl px-8 py-10">
        <p class="slide-eyebrow mb-2">Converge Platform</p>
        <h1 class="slide-headline mb-1 text-4xl!">Applications</h1>
        <p class="slide-body mb-8 text-sm!">Tools for governed decisions, research, and compliance.</p>

        <div class="grid grid-cols-2 gap-3">
          {#each appCards as app}
            {#if app.status === "active"}
              <button
                class="cursor-pointer rounded-xl border border-lime/30 bg-raised p-4 text-left transition-all hover:border-lime/50 hover:bg-surface"
                onclick={() => app.action?.()}
              >
                <div class="mb-2 flex items-center justify-between">
                  <span class="text-lg">{app.icon}</span>
                  <span class="pill pill-ok">Active</span>
                </div>
                <h3 class="mb-1 font-display text-sm font-semibold text-bright">{app.title}</h3>
                <p class="text-xs leading-relaxed text-subtle">{app.description}</p>
              </button>
            {:else}
              <div class="rounded-xl border border-border bg-raised p-4 opacity-50">
                <div class="mb-2 flex items-center justify-between">
                  <span class="text-lg">{app.icon}</span>
                  <span class="pill pill-info">Soon</span>
                </div>
                <h3 class="mb-1 font-display text-sm font-semibold text-bright">{app.title}</h3>
                <p class="text-xs leading-relaxed text-subtle">{app.description}</p>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </div>
  </div>

<!-- ═══════════════════════════════════════════════
     DUE DILIGENCE PHASE
     ═══════════════════════════════════════════════ -->
{:else if phase === "dd"}
  <div class="min-h-screen bg-void">
    <header class="flex items-center justify-between border-b border-border px-8 py-4">
      <div class="flex items-center gap-4">
        <button class="btn-ghost text-sm" onclick={goToApps}>&larr; Apps</button>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Due Diligence</span>
      </div>
      {#if ddFlow}
        <div class="flex items-center gap-3 text-xs text-muted">
          <span>{ddFlow.cycles} cycles</span>
          <span class="pill {ddFlow.converged ? 'pill-ok' : 'pill-warn'}">{ddFlow.converged ? "converged" : "stopped"}</span>
        </div>
      {/if}
    </header>

    <div class="mx-auto max-w-4xl px-8 py-8 dd-stage" class:dd-print-page={ddShowReport}>

      {#if ddShowReport && ddFlow}
        <!-- ── Final Report ── -->
        <div class="mb-8 flex items-center justify-between gap-4 fade-in">
          <div>
            <p class="slide-eyebrow mb-1">Due Diligence Report</p>
            <h1 class="slide-headline text-3xl!">{ddFlow.report.company_name}</h1>
            <p class="print-only mt-2 text-sm text-muted">
              Generated {ddPrintTimestamp || new Date().toLocaleString()} by Converge Governance
            </p>
          </div>
          <div class="no-print flex gap-3">
            <button class="btn-lime" onclick={printDdReport}>Print / Save PDF</button>
            <button class="btn-ghost" onclick={ddNewSearch}>New Search</button>
          </div>
        </div>

        <div class="callout callout-lime mb-6">
          <span class="card-label mb-2 block">Executive Summary</span>
          <p class="whitespace-pre-wrap text-sm text-text">{ddFlow.report.executive_summary}</p>
        </div>

        <div class="mb-6 grid grid-cols-3 gap-3">
          {#if ddFlow.report.market_analysis.length}
            <div class="rounded-xl border border-border bg-raised p-4">
              <span class="card-label mb-2 block">Market</span>
              <p class="whitespace-pre-wrap text-xs text-subtle">{ddFlow.report.market_analysis.join("\n\n")}</p>
            </div>
          {/if}
          {#if ddFlow.report.competitive_landscape.length}
            <div class="rounded-xl border border-border bg-raised p-4">
              <span class="card-label mb-2 block">Competition</span>
              <p class="whitespace-pre-wrap text-xs text-subtle">{ddFlow.report.competitive_landscape.join("\n\n")}</p>
            </div>
          {/if}
          {#if ddFlow.report.technology_assessment.length}
            <div class="rounded-xl border border-border bg-raised p-4">
              <span class="card-label mb-2 block">Technology</span>
              <p class="whitespace-pre-wrap text-xs text-subtle">{ddFlow.report.technology_assessment.join("\n\n")}</p>
            </div>
          {/if}
        </div>

        <div class="mb-6 grid grid-cols-2 gap-3">
          {#if ddFlow.report.contradictions.length}
            <div class="rounded-xl border border-border bg-raised p-4">
              <span class="card-label mb-2 block" style="color: var(--color-warn)">Contradictions</span>
              <ul class="space-y-1">
                {#each ddFlow.report.contradictions as item}
                  <li class="flex gap-2 text-xs text-text"><span class="text-warn">&#x26A0;</span>{item}</li>
                {/each}
              </ul>
            </div>
          {/if}
          {#if ddFlow.report.remaining_gaps.length}
            <div class="rounded-xl border border-border bg-raised p-4">
              <span class="card-label mb-2 block" style="color: var(--color-err)">Remaining Gaps</span>
              <ul class="space-y-1">
                {#each ddFlow.report.remaining_gaps as item}
                  <li class="flex gap-2 text-xs text-text"><span class="text-err">&#x2022;</span>{item}</li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>

        <div class="callout callout-lime mb-6">
          <span class="card-label mb-2 block">Recommendation</span>
          <p class="text-sm text-text">{ddFlow.report.recommendation}</p>
          <div class="mt-2 flex items-center gap-3 text-xs text-muted">
            <span>Confidence: {Math.round(ddFlow.report.confidence * 100)}%</span>
            {#if ddFlow.report.needs_human_review}
              <span class="pill pill-warn">Needs human review</span>
            {/if}
          </div>
        </div>

        <!-- Flow summary -->
        <section class="mb-6 no-print">
          <span class="card-label mb-3 block">Converge Flow ({ddFlow.stages.length} stages, {ddFlow.cycles} cycles)</span>
          <div class="flex flex-wrap gap-2">
            {#each ddFlow.stages as stage}
              <div class="rounded-lg border border-border bg-deep px-3 py-1.5 text-xs">
                <span style="color: {stageColors[stage.kind] ?? 'var(--color-text)'}">{stageIcons[stage.kind] ?? ""}</span>
                {stage.label}
                <span class="text-muted">({stage.items.length})</span>
              </div>
            {/each}
          </div>
        </section>

      {:else if ddFlow && !ddShowReport}
        <!-- ── Flow Reveal ── -->
        <div class="py-4">
          <p class="slide-eyebrow mb-2">Converge Governance Flow</p>
          <h2 class="slide-headline mb-6 text-3xl!">{ddCompanyName}</h2>

          <div class="space-y-5 mb-6">
            {#each ddFlow.stages as stage, stageIdx}
              {#if stageIdx < ddRevealedStages || (stageIdx === ddRevealedStages && ddRevealedItems > 0)}
                <div class="fade-in">
                  <!-- Stage header -->
                  <div class="flex items-center gap-3 mb-2">
                    <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-sm"
                      style="background: {stageColors[stage.kind] ?? 'var(--color-info)'}20; color: {stageColors[stage.kind] ?? 'var(--color-info)'}">
                      {#if stageIdx === ddRevealedStages && ddRevealedItems < stage.items.length}
                        <span class="inline-block h-2 w-2 animate-pulse rounded-full" style="background: {stageColors[stage.kind] ?? 'var(--color-info)'}"></span>
                      {:else}
                        {stageIcons[stage.kind] ?? "\u25CF"}
                      {/if}
                    </div>
                    <div class="text-sm font-semibold" style="color: {stageColors[stage.kind] ?? 'var(--color-bright)'}">{stage.label}</div>
                    <span class="text-xs text-muted">{stage.items.length} items</span>
                  </div>

                  <!-- Stage items -->
                  <div class="ml-10 space-y-1.5">
                    {#each stage.items as item, itemIdx}
                      {#if stageIdx < ddRevealedStages || itemIdx < ddRevealedItems}
                        <div class="flex items-start gap-2 rounded-lg border border-border/50 bg-raised/50 px-3 py-2 fade-in">
                          {#if item.category}
                            <span class="pill mt-0.5 shrink-0 text-[10px]" style="border-color: {stageColors[stage.kind] ?? 'var(--color-border)'}40">{item.category}</span>
                          {/if}
                          <div class="min-w-0 flex-1">
                            <p class="text-xs text-text leading-snug">{item.label}</p>
                            {#if stage.kind !== "extraction"}
                              <p class="mt-0.5 text-[10px] text-muted truncate">{item.detail}</p>
                            {/if}
                          </div>
                          {#if item.confidence}
                            <span class="shrink-0 text-[10px] text-muted">{Math.round(item.confidence * 100)}%</span>
                          {/if}
                        </div>
                      {/if}
                    {/each}
                  </div>
                </div>
              {/if}
            {/each}
          </div>

          <!-- Progress bar -->
          <div class="h-1 overflow-hidden rounded-full bg-border">
            <div class="h-full rounded-full bg-lime transition-all duration-300"
              style="width: {ddProgress()}%"></div>
          </div>
          <p class="mt-3 text-center text-xs text-muted">
            {#if ddRevealedStages < ddFlow.stages.length}
              {ddFlow.stages[ddRevealedStages]?.label ?? "Processing"}...
            {:else}
              Converged
            {/if}
          </p>
        </div>

      {:else if ddLoading}
        <!-- Waiting for engine -->
        <div class="py-8 text-center">
          <p class="slide-eyebrow mb-3">Starting convergence engine</p>
          <h2 class="slide-headline mb-8 text-3xl!">{ddCompanyName}</h2>
          <span class="inline-block h-3 w-3 animate-pulse rounded-full bg-lime"></span>
        </div>

      {:else}
        <!-- Input form -->
        <div class="py-8">
          <p class="slide-eyebrow mb-3">Research</p>
          <h2 class="slide-headline mb-2 text-3xl!">Due Diligence</h2>
          <p class="slide-body mb-8">
            Convergent due diligence powered by Organism planning and Converge governance.
            Formation, breadth research, depth research, fact extraction, contradiction detection, gap chasing, synthesis.
          </p>

          <form class="space-y-4" onsubmit={(e) => { e.preventDefault(); runDd(); }}>
            <div>
              <label class="card-label mb-2 block" for="dd-company">Company Name</label>
              <input
                id="dd-company"
                type="text"
                bind:value={ddCompanyName}
                placeholder="e.g. Stratsys"
                class="w-full rounded-xl border border-border bg-deep px-4 py-3 text-text placeholder:text-muted focus:border-lime/50 focus:outline-none"
                disabled={ddLoading}
              />
            </div>
            <button type="submit" class="btn-lime w-full py-3" disabled={ddLoading || !ddCompanyName.trim()}>
              Run Due Diligence
            </button>
            {#if ddError}
              <div class="callout callout-error">
                <p class="text-sm text-err">{ddError}</p>
              </div>
            {/if}
          </form>
        </div>
      {/if}
    </div>
  </div>
{/if}
