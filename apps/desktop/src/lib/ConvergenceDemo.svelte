<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { randomVerb } from "./spinner";

  export let onBack: () => void = () => {};
  export let onApps: () => void = () => {};
  export let onSpecStudio: () => void = () => {};

  interface VendorInput {
    name: string;
    score: number;
    risk_score: number;
    compliance_status: string;
    certifications: string[];
    monthly_cost_minor: number;
    currency_code: string;
  }

  interface LoopPattern {
    id: string;
    label: string;
    topology: string;
    summary: string;
    hypothesis: string;
    rootIntent: string;
    decisionPressure: string;
    inputs: Record<string, string>;
  }

  interface FactView {
    key: string;
    id: string;
    content: unknown;
    promotion?: unknown;
  }

  interface AgentView {
    id: string;
    pack: string;
    class: string;
    role: string;
    model: string;
    output: string;
  }

  interface ProjectionDetails {
    recommendation?: Record<string, any>;
    shortlist?: {
      shortlist?: Array<Record<string, any>>;
      rejected?: Array<Record<string, any>>;
    };
    policy?: Record<string, any>;
    formation?: {
      request_id?: string;
      assignments?: Array<{ role: string; suggestor: string }>;
      unmatched_roles?: string[];
      coverage_ratio?: number;
    };
    agents?: AgentView[];
    context?: {
      strategies?: FactView[];
      seeds?: FactView[];
      evaluations?: FactView[];
      proposals?: FactView[];
    };
    root_intent?: Record<string, any>;
    resources?: Record<string, any>;
    invariants?: Array<Record<string, any>>;
    optimization?: {
      solver?: string;
      objective?: string;
      hard_constraints?: string[];
      rows?: OptimizationRow[];
    };
    fixed_point?: {
      definition?: string;
      fact_counts?: Record<string, number>;
      terminal_facts?: string[];
    };
    learning?: Record<string, any>;
    stack_pressure?: StackPressure[];
  }

  interface TruthResult {
    converged: boolean;
    cycles: number;
    stop_reason: string;
    criteria_outcomes: { criterion: string; result: string }[];
    projection: {
      events_emitted: number;
      details: ProjectionDetails | null;
    } | null;
    llm_calls?: unknown[] | null;
  }

  interface OptimizationRow {
    vendor: string;
    feasible: boolean;
    score: number;
    risk: number;
    cost_major: number;
    cost_score: number;
    certification_score: number;
    objective_score: number;
    pareto_frontier: boolean;
  }

  interface RunSummary {
    run_id: string;
    cycles: number;
    elapsed_ms: number;
    vendor_count: number;
    converged: boolean;
    confidence: number;
    recommended_vendor: string;
    timestamp: string;
  }

  interface ExperienceSnapshot {
    truth_key: string;
    run_count: number;
    summaries: RunSummary[];
    aggregate: {
      convergence_rate: number;
      avg_cycles: number;
      avg_confidence: number;
      avg_elapsed_ms: number;
      recommendation_frequencies: Array<{
        recommendation: string;
        count: number;
        share: number;
      }>;
    };
  }

  interface StackPressure {
    layer: string;
    version: string;
    contract: string;
    demo_signal: string;
    pressure: string;
  }

  interface Stage {
    id: string;
    label: string;
    thesis: string;
    agentIds: string[];
    contextKeys: Array<"strategies" | "seeds" | "evaluations" | "proposals">;
  }

  const balancedVendors: VendorInput[] = [
    {
      name: "UiPath",
      score: 84,
      risk_score: 21,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 4100000,
      currency_code: "USD",
    },
    {
      name: "Blue Prism",
      score: 77,
      risk_score: 28,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001"],
      monthly_cost_minor: 3600000,
      currency_code: "USD",
    },
    {
      name: "Tungsten Automation",
      score: 91,
      risk_score: 24,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 4700000,
      currency_code: "USD",
    },
    {
      name: "DataBridge",
      score: 82,
      risk_score: 41,
      compliance_status: "pending",
      certifications: ["SOC2"],
      monthly_cost_minor: 2900000,
      currency_code: "USD",
    },
  ];

  const optimizerVendors: VendorInput[] = [
    {
      name: "Acme AI",
      score: 86,
      risk_score: 16,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 3200000,
      currency_code: "USD",
    },
    {
      name: "Nova Models",
      score: 94,
      risk_score: 19,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001"],
      monthly_cost_minor: 6900000,
      currency_code: "USD",
    },
    {
      name: "Epsilon AI",
      score: 88,
      risk_score: 13,
      compliance_status: "compliant",
      certifications: ["SOC2", "GDPR"],
      monthly_cost_minor: 3900000,
      currency_code: "USD",
    },
    {
      name: "Gamma LLM",
      score: 92,
      risk_score: 34,
      compliance_status: "pending",
      certifications: ["ISO27001"],
      monthly_cost_minor: 6500000,
      currency_code: "USD",
    },
  ];

  const policyStressVendors: VendorInput[] = [
    {
      name: "Gamma LLM",
      score: 92,
      risk_score: 12,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001"],
      monthly_cost_minor: 6500000,
      currency_code: "USD",
    },
    {
      name: "Acme AI",
      score: 84,
      risk_score: 16,
      compliance_status: "compliant",
      certifications: ["SOC2", "GDPR"],
      monthly_cost_minor: 4200000,
      currency_code: "USD",
    },
  ];

  const honestStopVendors: VendorInput[] = [
    {
      name: "Opaque Systems",
      score: 71,
      risk_score: 46,
      compliance_status: "pending",
      certifications: ["SOC2"],
      monthly_cost_minor: 2800000,
      currency_code: "USD",
    },
    {
      name: "Frontier Stack",
      score: 89,
      risk_score: 51,
      compliance_status: "pending",
      certifications: ["ISO27001"],
      monthly_cost_minor: 6100000,
      currency_code: "USD",
    },
    {
      name: "CheapGen",
      score: 69,
      risk_score: 31,
      compliance_status: "compliant",
      certifications: ["SOC2"],
      monthly_cost_minor: 1200000,
      currency_code: "USD",
    },
  ];

  const loopPatterns: LoopPattern[] = [
    {
      id: "panel",
      label: "Curated Panel",
      topology: "Panel",
      summary: "Expert roles score the same evidence base, then a synthesis agent writes one decision record.",
      hypothesis: "Works when the rubric is stable and the buyer wants a board-readable recommendation.",
      rootIntent: "Select an automation vendor for claims and invoice exceptions without losing auditability.",
      decisionPressure: "Balanced score, risk, compliance, and cost.",
      inputs: {
        vendors_json: JSON.stringify(balancedVendors),
        min_score: "75",
        max_risk: "30",
        max_vendors: "3",
      },
    },
    {
      id: "optimizer",
      label: "Constraint Optimizer",
      topology: "Self-organizing -> Optimizer",
      summary: "Let facts accumulate, then apply hard constraints and a weighted objective over feasible vendors.",
      hypothesis: "Works when score alone is misleading and cost/risk trade-offs need a mathematical surface.",
      rootIntent: "Find the best feasible vendor, not merely the highest scoring vendor.",
      decisionPressure: "CP/SAT style hard constraints plus OR-style objective scoring.",
      inputs: {
        vendors_json: JSON.stringify(optimizerVendors),
        min_score: "80",
        max_risk: "25",
        max_vendors: "2",
      },
    },
    {
      id: "huddle",
      label: "Policy Huddle",
      topology: "Huddle",
      summary: "Force the same committee through strict authority and HITL gates before commitment.",
      hypothesis: "Works for high-value commitments where the best answer may be escalation, not approval.",
      rootIntent: "Decide whether a high-scoring but expensive vendor can be committed without override.",
      decisionPressure: "Human approval and spending authority are intentionally stressed.",
      inputs: {
        vendors_json: JSON.stringify(policyStressVendors),
        min_score: "75",
        max_risk: "30",
        max_vendors: "2",
        human_approval_present: "false",
      },
    },
    {
      id: "adversarial",
      label: "Adversarial Review",
      topology: "Huddle -> Panel",
      summary: "Use the loop to surface contradictions and prove when no vendor should be promoted.",
      hypothesis: "Works when the correct governance outcome is honest stopping or manual review.",
      rootIntent: "Prevent a weak vendor field from becoming a fake recommendation.",
      decisionPressure: "Score floor, risk ceiling, and compliance invariant are deliberately hard.",
      inputs: {
        vendors_json: JSON.stringify(honestStopVendors),
        min_score: "80",
        max_risk: "25",
        max_vendors: "2",
      },
    },
  ];

  const stages: Stage[] = [
    {
      id: "intent",
      label: "Intent",
      thesis: "Declare the outcome, authority, constraints, and success criteria before work begins.",
      agentIds: ["planning-seed"],
      contextKeys: ["strategies"],
    },
    {
      id: "formation",
      label: "Formation",
      thesis: "Assemble the right roles for the root intent rather than assuming one fixed flow.",
      agentIds: ["planning-seed"],
      contextKeys: ["strategies"],
    },
    {
      id: "screening",
      label: "Screen",
      thesis: "LLM/search-backed or deterministic agents propose compliance and eligibility facts.",
      agentIds: ["compliance-screener"],
      contextKeys: ["seeds"],
    },
    {
      id: "evaluation",
      label: "Evaluate",
      thesis: "Cost analytics and risk models convert vendor claims into comparable evidence.",
      agentIds: ["cost-analysis", "vendor-risk"],
      contextKeys: ["evaluations"],
    },
    {
      id: "optimize",
      label: "Optimize",
      thesis: "Hard constraints filter infeasible choices; the objective ranks the remaining search space.",
      agentIds: ["vendor-shortlist"],
      contextKeys: ["proposals"],
    },
    {
      id: "synthesis",
      label: "Synthesize",
      thesis: "LLM reasoning explains the trade-offs without bypassing facts or policy.",
      agentIds: ["decision-synthesis"],
      contextKeys: ["evaluations"],
    },
    {
      id: "policy",
      label: "Policy",
      thesis: "Cedar decides whether the recommendation can become a commitment.",
      agentIds: ["policy-gate"],
      contextKeys: ["evaluations"],
    },
    {
      id: "fixed-point",
      label: "Fixed Point",
      thesis: "The loop stops only when no new promotable fact can change the decision.",
      agentIds: [],
      contextKeys: ["strategies", "seeds", "evaluations", "proposals"],
    },
  ];

  let selectedPatternId = loopPatterns[0].id;
  let result: TruthResult | null = null;
  let experience: ExperienceSnapshot | null = null;
  let loading = false;
  let error = "";
  let experienceError = "";
  let activeStage = 0;
  let revealedStages = 0;
  let spinnerVerb = randomVerb();
  let spinnerInterval: ReturnType<typeof setInterval> | null = null;
  let timers: ReturnType<typeof setTimeout>[] = [];

  $: selectedPattern = loopPatterns.find((pattern) => pattern.id === selectedPatternId) ?? loopPatterns[0];
  $: details = result?.projection?.details ?? null;
  $: agents = details?.agents ?? [];
  $: context = details?.context ?? {};
  $: selectedStage = stages[activeStage] ?? stages[0];
  $: selectedFacts = factsForStage(selectedStage);
  $: selectedAgents = agents.filter((agent) => selectedStage.agentIds.includes(agent.id));
  $: candidateVendors = vendorsForPattern(selectedPattern);
  $: optimizationRows = rowsForDisplay();
  $: foundationPressure = foundationPressureRows();
  $: contextCounts = {
    strategies: context.strategies?.length ?? 0,
    seeds: context.seeds?.length ?? 0,
    evaluations: context.evaluations?.length ?? 0,
    proposals: context.proposals?.length ?? 0,
  };

  onMount(() => {
    void fetchExperience();
  });

  onDestroy(() => {
    clearTimers();
    stopSpinner();
  });

  function selectPattern(patternId: string) {
    selectedPatternId = patternId;
    result = null;
    error = "";
    activeStage = 0;
    revealedStages = 0;
    clearTimers();
  }

  function vendorsForPattern(pattern: LoopPattern): VendorInput[] {
    try {
      const parsed = JSON.parse(pattern.inputs.vendors_json ?? "[]");
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  }

  function clearTimers() {
    timers.forEach(clearTimeout);
    timers = [];
  }

  function stopSpinner() {
    if (spinnerInterval) {
      clearInterval(spinnerInterval);
      spinnerInterval = null;
    }
  }

  async function fetchExperience() {
    experienceError = "";
    try {
      const response = await fetch("http://127.0.0.1:8080/v1/experience/vendor-selection");
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      experience = await response.json();
    } catch (cause) {
      experience = null;
      experienceError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function runConvergence() {
    clearTimers();
    stopSpinner();
    loading = true;
    error = "";
    result = null;
    revealedStages = 0;
    activeStage = 0;
    spinnerVerb = randomVerb();
    spinnerInterval = setInterval(() => {
      spinnerVerb = randomVerb();
    }, 1800);

    try {
      const response = await fetch("http://127.0.0.1:8080/v1/truths/vendor-selection/execute", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          inputs: {
            ...selectedPattern.inputs,
            loop_pattern: selectedPattern.id,
          },
          persist_projection: true,
        }),
      });

      if (!response.ok) {
        const body = await response.text();
        throw new Error(body || `HTTP ${response.status}`);
      }

      result = await response.json();
      loading = false;
      stopSpinner();
      animateStages();
      void fetchExperience();
    } catch (cause) {
      loading = false;
      stopSpinner();
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function animateStages() {
    clearTimers();
    stages.forEach((_, index) => {
      const timer = setTimeout(() => {
        activeStage = index;
        revealedStages = index + 1;
      }, index * 520);
      timers.push(timer);
    });
  }

  function factsForStage(stage: Stage): FactView[] {
    if (!details?.context) return [];
    const allFacts = stage.contextKeys.flatMap((key) => details.context?.[key] ?? []);
    if (stage.id === "intent") {
      return allFacts.filter((fact) => fact.id.startsWith("strategy:vendor-sel:"));
    }
    if (stage.id === "formation") {
      return allFacts.filter((fact) => fact.id.startsWith("formation:plan:"));
    }
    if (stage.id === "evaluation") {
      return allFacts.filter((fact) => fact.id.startsWith("cost:") || fact.id.startsWith("risk:"));
    }
    if (stage.id === "synthesis") {
      return allFacts.filter((fact) => fact.id === "decision:recommendation");
    }
    if (stage.id === "policy") {
      return allFacts.filter((fact) => fact.id.startsWith("policy:decision:"));
    }
    return allFacts;
  }

  function canInspect(index: number) {
    return Boolean(result && index < revealedStages);
  }

  function inspectStage(index: number) {
    if (!canInspect(index)) return;
    activeStage = index;
  }

  function stageTone(index: number) {
    if (loading && index === activeStage) return "active";
    if (result && index < revealedStages) return "done";
    return "pending";
  }

  function rowsForDisplay(): OptimizationRow[] {
    const rows = details?.optimization?.rows;
    if (Array.isArray(rows) && rows.length > 0) return rows;
    return localOptimizationRows(candidateVendors);
  }

  function foundationPressureRows(): StackPressure[] {
    const rows = details?.stack_pressure;
    if (Array.isArray(rows) && rows.length > 0) return rows;
    return [
      {
        layer: "Helm",
        version: "0.1.0",
        contract: "Operator workbench for truth execution and evidence inspection.",
        demo_signal: "Waiting for a run.",
        pressure: "Run a loop pattern to see which product surfaces the demo stresses.",
      },
      {
        layer: "Axiom",
        version: "0.7.0",
        contract: "Normative truth contract, invariants, examples, and policy lens.",
        demo_signal: "Planned invariants are score, risk, compliance, HITL, and provenance.",
        pressure: "Editable truth artifacts should compile into visible diagnostics.",
      },
      {
        layer: "Organism",
        version: "1.4.0",
        contract: "Intent, planning seed, formation assembly, and topology choice.",
        demo_signal: selectedPattern.topology,
        pressure: "Topology labels should become typed plan bundles.",
      },
      {
        layer: "Converge",
        version: "3.7.4",
        contract: "Engine cycles, context partitions, promotion, policy, and fixed points.",
        demo_signal: "Context partitions will populate after execution.",
        pressure: "Participants need richer criterion evidence and promotion traces.",
      },
      {
        layer: "Ferrox",
        version: "0.3.12",
        contract: "Optimization substrate for feasible sets and Pareto frontier decisions.",
        demo_signal: `${candidateVendors.length} candidate rows previewed locally.`,
        pressure: "The local optimizer should graduate into a Ferrox-backed suggestor.",
      },
    ];
  }

  function localOptimizationRows(vendors: VendorInput[]): OptimizationRow[] {
    const costs = vendors.map((vendor) => moneyMajor(vendor.monthly_cost_minor));
    const minCost = Math.min(...costs, 0);
    const maxCost = Math.max(...costs, minCost);
    const minScore = Number(selectedPattern.inputs.min_score ?? 0);
    const maxRisk = Number(selectedPattern.inputs.max_risk ?? 100);

    return vendors
      .map((vendor) => {
        const costMajor = moneyMajor(vendor.monthly_cost_minor);
        const costScore = costEfficiencyScore(costMajor, minCost, maxCost);
        const certificationScore = certificationScoreFor(vendor);
        const objectiveScore = round1(
          0.35 * vendor.score +
            0.25 * Math.max(0, 100 - vendor.risk_score) +
            0.2 * costScore +
            0.2 * certificationScore
        );
        return {
          vendor: vendor.name,
          feasible:
            vendor.compliance_status === "compliant" &&
            vendor.score >= minScore &&
            vendor.risk_score <= maxRisk,
          score: vendor.score,
          risk: vendor.risk_score,
          cost_major: costMajor,
          cost_score: costScore,
          certification_score: certificationScore,
          objective_score: objectiveScore,
          pareto_frontier: isParetoFrontier(vendor, vendors),
        };
      })
      .sort((a, b) => b.objective_score - a.objective_score);
  }

  function moneyMajor(minor: number) {
    return Math.ceil(Math.max(0, minor) / 100);
  }

  function costEfficiencyScore(cost: number, minCost: number, maxCost: number) {
    if (maxCost <= minCost) return 100;
    return round1((1 - (cost - minCost) / (maxCost - minCost)) * 100);
  }

  function certificationScoreFor(vendor: VendorInput) {
    const required = ["SOC2", "ISO27001", "GDPR"];
    const matched = required.filter((requiredCert) =>
      vendor.certifications.some((cert) => cert.toLowerCase() === requiredCert.toLowerCase())
    ).length;
    return round1((matched / required.length) * 100);
  }

  function isParetoFrontier(vendor: VendorInput, vendors: VendorInput[]) {
    const cost = moneyMajor(vendor.monthly_cost_minor);
    return !vendors.some((other) => {
      if (other.name === vendor.name) return false;
      const otherCost = moneyMajor(other.monthly_cost_minor);
      const atLeastAsGood =
        other.score >= vendor.score &&
        other.risk_score <= vendor.risk_score &&
        otherCost <= cost;
      const strictlyBetter =
        other.score > vendor.score ||
        other.risk_score < vendor.risk_score ||
        otherCost < cost;
      return atLeastAsGood && strictlyBetter;
    });
  }

  function round1(value: number) {
    return Math.round(value * 10) / 10;
  }

  function formatJson(value: unknown) {
    return JSON.stringify(value, null, 2);
  }

  function formatMoney(value: unknown) {
    if (typeof value !== "number") return "-";
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(value);
  }

  function pct(value: unknown) {
    if (typeof value !== "number") return "-";
    return `${Math.round(value * 100)}%`;
  }

  function number(value: unknown, fallback = "-") {
    return typeof value === "number" ? value.toLocaleString("en-US") : fallback;
  }

  function policyOutcome() {
    const outcome = details?.policy?.outcome;
    return typeof outcome === "string" ? outcome : "Pending";
  }

  function policyTone() {
    const outcome = policyOutcome();
    if (outcome === "Promote") return "ok";
    if (outcome === "Escalate") return "warn";
    if (outcome === "Reject") return "err";
    return "info";
  }

  function resultTone(value: string) {
    if (value.includes("Met")) return "ok";
    if (value.includes("Blocked")) return "warn";
    return "err";
  }

  function bestPriorRecommendation() {
    return experience?.aggregate.recommendation_frequencies[0]?.recommendation ?? "No prior winner";
  }

  function latestRuns() {
    return [...(experience?.summaries ?? [])].reverse().slice(0, 4);
  }
</script>

<div class="min-h-screen overflow-hidden bg-void">
  <header class="flex items-center justify-between border-b border-border bg-void/90 px-6 py-4 backdrop-blur">
    <div class="flex items-center gap-4">
      <button class="btn-ghost text-sm" type="button" onclick={onBack}>&larr; Slides</button>
      <div>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Vendor Selection Lab</span>
        <p class="mt-0.5 text-xs text-subtle">Converge + Organism + Cedar + experience</p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <button class="btn-ghost" type="button" onclick={onSpecStudio}>Spec Studio</button>
      <button class="btn-ghost" type="button" onclick={onApps}>Apps</button>
    </div>
  </header>

  <main class="relative mx-auto max-w-[1800px] px-5 py-5">
    <div class="pointer-events-none absolute inset-x-0 top-0 h-[460px] bg-[radial-gradient(circle_at_20%_0%,rgba(204,255,0,0.12),transparent_32%),radial-gradient(circle_at_85%_12%,rgba(96,165,250,0.14),transparent_28%)]"></div>

    <section class="relative mb-5 grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(460px,0.8fr)]">
      <div class="rounded-[28px] border border-border bg-panel p-6 shadow-2xl">
        <p class="slide-eyebrow mb-3">Root Intent</p>
        <h1 class="font-display text-4xl font-semibold leading-tight text-bright md:text-5xl">
          Vendor selection is not a scorecard. It is a governed search for a stable decision.
        </h1>
        <p class="mt-4 max-w-3xl text-base leading-relaxed text-subtle">
          {details?.root_intent?.statement ?? selectedPattern.rootIntent}
        </p>

        <div class="mt-6 grid gap-3 md:grid-cols-4">
          <div class="rounded-2xl border border-border bg-raised p-4">
            <span class="card-label">Candidates</span>
            <div class="mt-2 font-display text-3xl text-bright">{candidateVendors.length}</div>
          </div>
          <div class="rounded-2xl border border-border bg-raised p-4">
            <span class="card-label">Authority</span>
            <div class="mt-2 text-sm text-bright">{String(details?.root_intent?.authority?.["level"] ?? "supervisory")}</div>
          </div>
          <div class="rounded-2xl border border-border bg-raised p-4">
            <span class="card-label">Budget Gate</span>
            <div class="mt-2 text-sm text-bright">{formatMoney(details?.resources?.financial_boundary?.["hitl_threshold_major"] ?? 50000)}</div>
          </div>
          <div class="rounded-2xl border border-border bg-raised p-4">
            <span class="card-label">Prior Runs</span>
            <div class="mt-2 font-display text-3xl text-bright">{experience?.run_count ?? 0}</div>
          </div>
        </div>
      </div>

      <aside class="rounded-[28px] border border-lime/20 bg-lime-glow p-5">
        <div class="flex items-start justify-between gap-3">
          <div>
            <p class="card-label">Loop Pattern</p>
            <h2 class="mt-2 font-display text-2xl font-semibold text-bright">{selectedPattern.label}</h2>
          </div>
          <span class="pill pill-info">{selectedPattern.topology}</span>
        </div>
        <p class="mt-3 text-sm leading-relaxed text-subtle">{selectedPattern.summary}</p>
        <div class="mt-4 rounded-2xl border border-border bg-deep/80 p-4">
          <span class="card-label">Hypothesis Under Test</span>
          <p class="mt-2 text-sm text-text">{selectedPattern.hypothesis}</p>
        </div>
        <button class="btn-lime mt-4 w-full justify-center" type="button" onclick={runConvergence} disabled={loading}>
          {#if loading}{spinnerVerb}...{:else}Run This Loop{/if}
        </button>
      </aside>
    </section>

    <section class="relative mb-5 grid gap-3 xl:grid-cols-4">
      {#each loopPatterns as pattern}
        <button
          type="button"
          class="group rounded-2xl border bg-raised p-4 text-left transition hover:-translate-y-0.5 hover:border-subtle hover:bg-surface"
          class:border-lime={selectedPatternId === pattern.id}
          class:border-border={selectedPatternId !== pattern.id}
          onclick={() => selectPattern(pattern.id)}
          disabled={loading}
        >
          <div class="mb-3 flex items-center justify-between gap-3">
            <strong class="font-display text-base text-bright">{pattern.label}</strong>
            <span class="pill" class:pill-ok={selectedPatternId === pattern.id} class:pill-info={selectedPatternId !== pattern.id}>
              {pattern.topology}
            </span>
          </div>
          <p class="text-xs leading-relaxed text-subtle">{pattern.decisionPressure}</p>
        </button>
      {/each}
    </section>

    {#if error}
      <div class="callout callout-error relative mb-5">
        <strong>Server unavailable</strong>
        <p>{error}</p>
      </div>
    {/if}

    <section class="relative grid gap-5 xl:grid-cols-[330px_minmax(0,1fr)_450px]">
      <aside class="space-y-4">
        <section class="rounded-2xl border border-border bg-deep p-4">
          <div class="flex items-center justify-between">
            <span class="card-label">Available Resources</span>
            <span class="pill pill-info">{loading ? "Running" : result ? "Observed" : "Planned"}</span>
          </div>
          <div class="mt-4 space-y-2">
            <div class="flex items-center justify-between rounded-xl border border-border bg-raised px-3 py-2">
              <span class="text-xs text-muted">Evidence channels</span>
              <span class="text-sm text-bright">5</span>
            </div>
            <div class="flex items-center justify-between rounded-xl border border-border bg-raised px-3 py-2">
              <span class="text-xs text-muted">Agent roles</span>
              <span class="text-sm text-bright">8</span>
            </div>
            <div class="flex items-center justify-between rounded-xl border border-border bg-raised px-3 py-2">
              <span class="text-xs text-muted">Max cycles</span>
              <span class="text-sm text-bright">{String(details?.resources?.compute_budget?.["max_cycles"] ?? 10)}</span>
            </div>
            <div class="flex items-center justify-between rounded-xl border border-border bg-raised px-3 py-2">
              <span class="text-xs text-muted">LLM mode</span>
              <span class="text-sm text-bright">{result?.llm_calls?.length ? "live" : "fallback-ready"}</span>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border border-border bg-deep p-4">
          <span class="card-label mb-3 block">Invariants</span>
          <div class="space-y-2">
            {#each (details?.invariants ?? []) as invariant}
              <article class="rounded-xl border border-border bg-raised px-3 py-2">
                <div class="flex items-center justify-between gap-3">
                  <strong class="font-mono text-xs text-lime">{String(invariant.id ?? "invariant")}</strong>
                  <span class="text-[0.68rem] uppercase tracking-wide text-muted">{String(invariant.owned_by ?? "")}</span>
                </div>
                <p class="mt-1 text-xs leading-relaxed text-subtle">{String(invariant.statement ?? "")}</p>
              </article>
            {:else}
              <article class="rounded-xl border border-border bg-raised px-3 py-3">
                <strong class="text-sm text-bright">Pre-run invariant set</strong>
                <p class="mt-1 text-xs text-subtle">Score floor, risk ceiling, compliance, authority, and provenance will be materialized by the run.</p>
              </article>
            {/each}
          </div>
        </section>

        <section class="rounded-2xl border border-border bg-deep p-4">
          <div class="flex items-center justify-between">
            <span class="card-label">Experience Store</span>
            <button class="text-xs text-lime" type="button" onclick={fetchExperience}>Refresh</button>
          </div>
          {#if experienceError}
            <p class="mt-3 text-xs text-warn">No snapshot yet: {experienceError}</p>
          {:else}
            <div class="mt-3 grid grid-cols-2 gap-2">
              <div class="rounded-xl border border-border bg-raised p-3">
                <span class="text-xs text-muted">Convergence</span>
                <div class="font-display text-2xl text-bright">{pct(experience?.aggregate.convergence_rate ?? 0)}</div>
              </div>
              <div class="rounded-xl border border-border bg-raised p-3">
                <span class="text-xs text-muted">Avg cycles</span>
                <div class="font-display text-2xl text-bright">{number(experience?.aggregate.avg_cycles ?? 0)}</div>
              </div>
            </div>
            <div class="mt-3 rounded-xl border border-border bg-surface p-3">
              <span class="card-label">Most Recalled</span>
              <p class="mt-2 line-clamp-3 text-xs text-subtle">{bestPriorRecommendation()}</p>
            </div>
            {#if latestRuns().length > 0}
              <div class="mt-3 space-y-2">
                {#each latestRuns() as run}
                  <div class="rounded-xl border border-border bg-raised px-3 py-2">
                    <div class="flex items-center justify-between gap-2">
                      <span class="truncate text-xs text-bright">{run.recommended_vendor || "manual review"}</span>
                      <span class="text-xs text-muted">{run.cycles} cycles</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          {/if}
        </section>
      </aside>

      <section class="min-w-0 space-y-4">
        <div class="grid gap-3 md:grid-cols-4">
          {#each Object.entries(contextCounts) as [key, count]}
            <div class="rounded-2xl border border-border bg-raised px-4 py-3">
              <span class="card-label">{key}</span>
              <div class="mt-2 font-display text-3xl text-bright">{count}</div>
            </div>
          {/each}
        </div>

        <section class="rounded-[28px] border border-border bg-deep p-4">
          <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
            <div>
              <span class="card-label">Converging Loop</span>
              <h2 class="mt-1 font-display text-2xl font-semibold text-bright">{selectedStage.label}</h2>
              <p class="mt-1 max-w-2xl text-sm text-subtle">{selectedStage.thesis}</p>
            </div>
            <div class="flex items-center gap-2">
              <span class="pill" class:pill-ok={result?.converged} class:pill-warn={result && !result.converged} class:pill-info={!result}>
                {result ? (result.converged ? "Converged" : "Stopped") : "Ready"}
              </span>
              <span class="pill" class:pill-ok={policyTone() === "ok"} class:pill-warn={policyTone() === "warn"} class:pill-err={policyTone() === "err"} class:pill-info={policyTone() === "info"}>
                {policyOutcome()}
              </span>
            </div>
          </div>

          <div class="mb-4 grid gap-2 lg:grid-cols-8">
            {#each stages as stage, index}
              <button
                type="button"
                class="rounded-2xl border border-border bg-raised px-3 py-3 text-left transition"
                class:border-lime={index === activeStage}
                class:bg-surface={index === activeStage}
                class:opacity-45={!canInspect(index) && !loading}
                onclick={() => inspectStage(index)}
                disabled={!canInspect(index) && !loading}
              >
                <div
                  class="mb-2 flex h-7 w-7 items-center justify-center rounded-full border border-border text-xs"
                  class:bg-lime={stageTone(index) === "done"}
                  class:text-void={stageTone(index) === "done"}
                  class:text-lime={stageTone(index) === "active"}
                >
                  {index + 1}
                </div>
                <span class="block truncate text-xs font-semibold text-bright">{stage.label}</span>
              </button>
            {/each}
          </div>

          <div class="grid gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(280px,0.9fr)]">
            <div class="min-w-0">
              <span class="card-label mb-3 block">Promoted Facts</span>
              {#if selectedFacts.length > 0}
                <div class="space-y-3">
                  {#each selectedFacts as fact}
                    <article class="rounded-2xl border border-border bg-raised">
                      <div class="flex flex-wrap items-center justify-between gap-2 border-b border-border px-3 py-2">
                        <span class="font-mono text-xs text-lime">{fact.id}</span>
                        <span class="text-xs text-muted">{fact.key}</span>
                      </div>
                      <pre class="max-h-52 overflow-auto p-3 font-mono text-xs leading-relaxed text-subtle">{formatJson(fact.content)}</pre>
                    </article>
                  {/each}
                </div>
              {:else if loading}
                <div class="callout callout-neutral">
                  <strong>{spinnerVerb}...</strong>
                  <p>Agents are proposing facts into governed context partitions.</p>
                </div>
              {:else}
                <div class="callout callout-neutral">
                  <strong>Run a loop pattern</strong>
                  <p>The inspector will show exactly which facts were promoted and why the loop stopped.</p>
                </div>
              {/if}
            </div>

            <div class="min-w-0 space-y-4">
              <section>
                <span class="card-label mb-3 block">Active Roles</span>
                {#if selectedAgents.length > 0}
                  <div class="space-y-2">
                    {#each selectedAgents as agent}
                      <article class="rounded-2xl border border-border bg-surface px-3 py-3">
                        <div class="flex items-start justify-between gap-3">
                          <div>
                            <strong class="block text-sm text-bright">{agent.id}</strong>
                            <span class="text-xs text-muted">{agent.pack}</span>
                          </div>
                          <span class="pill pill-info">{agent.class}</span>
                        </div>
                        <p class="mt-2 text-xs text-subtle">{agent.role}</p>
                        <p class="mt-1 font-mono text-xs text-muted">{agent.model}</p>
                      </article>
                    {/each}
                  </div>
                {:else}
                  <div class="rounded-2xl border border-border bg-surface px-3 py-3 text-sm text-subtle">
                    Criteria evaluator and fixed-point detector
                  </div>
                {/if}
              </section>

              {#if selectedStage.id === "formation" || selectedStage.id === "fixed-point"}
                <section>
                  <span class="card-label mb-3 block">Formation</span>
                  <div class="rounded-2xl border border-border bg-surface px-3 py-3">
                    <div class="mb-3 flex items-center justify-between">
                      <span class="text-sm text-bright">{details?.formation?.request_id ?? "vendor-selection"}</span>
                      <span class="pill pill-ok">{Math.round((details?.formation?.coverage_ratio ?? 0) * 100)}%</span>
                    </div>
                    <div class="space-y-2">
                      {#each details?.formation?.assignments ?? [] as assignment}
                        <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-deep px-3 py-2">
                          <span class="text-xs text-subtle">{assignment.role}</span>
                          <span class="font-mono text-xs text-lime">{assignment.suggestor}</span>
                        </div>
                      {/each}
                    </div>
                  </div>
                </section>
              {/if}

              {#if selectedStage.id === "policy" || selectedStage.id === "fixed-point"}
                <section>
                  <span class="card-label mb-3 block">Policy Gate</span>
                  <div class="rounded-2xl border border-border bg-surface px-3 py-3">
                    <div class="flex items-center justify-between">
                      <strong class="text-sm text-bright">{policyOutcome()}</strong>
                      <span class="text-xs text-muted">{formatMoney(details?.policy?.selected_amount_major)}</span>
                    </div>
                    <p class="mt-2 text-xs text-subtle">{String(details?.policy?.reason ?? "No policy reason emitted yet.")}</p>
                    <div class="mt-3 grid grid-cols-2 gap-2 text-xs text-subtle">
                      <div class="rounded-xl border border-border bg-deep px-2 py-2">Authority: {String(details?.policy?.principal_authority ?? "-")}</div>
                      <div class="rounded-xl border border-border bg-deep px-2 py-2">Approval: {details?.policy?.human_approval_present ? "present" : "missing"}</div>
                    </div>
                  </div>
                </section>
              {/if}

              {#if selectedStage.id === "fixed-point"}
                <section>
                  <span class="card-label mb-3 block">Fixed Point</span>
                  <div class="rounded-2xl border border-border bg-surface px-3 py-3">
                    <p class="text-xs leading-relaxed text-subtle">{details?.fixed_point?.definition ?? "No fixed point proof emitted yet."}</p>
                    <div class="mt-3 grid grid-cols-2 gap-2">
                      <div class="rounded-xl border border-border bg-deep px-3 py-2">
                        <span class="text-xs text-muted">Cycles</span>
                        <div class="font-display text-2xl text-bright">{result?.cycles ?? "-"}</div>
                      </div>
                      <div class="rounded-xl border border-border bg-deep px-3 py-2">
                        <span class="text-xs text-muted">Events</span>
                        <div class="font-display text-2xl text-bright">{result?.projection?.events_emitted ?? "-"}</div>
                      </div>
                    </div>
                  </div>
                </section>
              {/if}
            </div>
          </div>
        </section>
      </section>

      <aside class="space-y-4">
        <section class="rounded-2xl border border-border bg-deep p-4">
          <div class="flex items-start justify-between gap-3">
            <div>
              <span class="card-label">Optimization Surface</span>
              <h3 class="mt-1 font-display text-xl font-semibold text-bright">Feasible set and objective</h3>
            </div>
            <span class="pill pill-info">{details?.optimization?.solver ?? "local preview"}</span>
          </div>
          <p class="mt-3 text-xs leading-relaxed text-subtle">
            {details?.optimization?.objective ?? "0.35*capability + 0.25*risk_adjusted + 0.20*cost_efficiency + 0.20*certification_coverage"}
          </p>
          <div class="mt-4 space-y-2">
            {#each optimizationRows as row}
              <article class="rounded-2xl border border-border bg-raised px-3 py-3">
                <div class="flex items-center justify-between gap-3">
                  <strong class="text-sm text-bright">{row.vendor}</strong>
                  <span class="pill" class:pill-ok={row.feasible} class:pill-err={!row.feasible}>
                    {row.feasible ? "feasible" : "blocked"}
                  </span>
                </div>
                <div class="mt-3 grid grid-cols-4 gap-2 text-xs">
                  <div class="rounded-lg bg-deep px-2 py-2 text-subtle">Score <b class="block text-bright">{row.score}</b></div>
                  <div class="rounded-lg bg-deep px-2 py-2 text-subtle">Risk <b class="block text-bright">{row.risk}</b></div>
                  <div class="rounded-lg bg-deep px-2 py-2 text-subtle">Cost <b class="block text-bright">{formatMoney(row.cost_major)}</b></div>
                  <div class="rounded-lg bg-deep px-2 py-2 text-subtle">Obj <b class="block text-lime">{row.objective_score}</b></div>
                </div>
                {#if row.pareto_frontier}
                  <p class="mt-2 text-xs text-lime">Pareto frontier candidate</p>
                {/if}
              </article>
            {/each}
          </div>
        </section>

        <section class="rounded-2xl border border-border bg-deep p-4">
          <span class="card-label mb-3 block">Shortlist</span>
          {#if details?.shortlist?.shortlist?.length}
            <div class="space-y-2">
              {#each details.shortlist.shortlist as vendor}
                <div class="rounded-2xl border border-border bg-raised px-3 py-3">
                  <div class="flex items-center justify-between gap-3">
                    <strong class="text-sm text-bright">{String(vendor.vendor_name)}</strong>
                    <span class="pill pill-ok">#{String(vendor.rank)}</span>
                  </div>
                  <div class="mt-2 grid grid-cols-3 gap-2 text-xs text-subtle">
                    <span>Score {String(vendor.score)}</span>
                    <span>Risk {String(vendor.risk_score)}</span>
                    <span>Obj {String(vendor.composite_score)}</span>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-sm text-muted">Run result will populate the shortlist.</p>
          {/if}
        </section>

        <section class="rounded-2xl border border-border bg-deep p-4">
          <span class="card-label mb-3 block">Why It Stopped</span>
          {#if result}
            <div class="space-y-2">
              {#each result.criteria_outcomes as vote}
                <div class="rounded-xl border border-border bg-surface px-3 py-2">
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-xs text-subtle">{vote.criterion}</span>
                    <span
                      class="pill"
                      class:pill-ok={resultTone(vote.result) === "ok"}
                      class:pill-warn={resultTone(vote.result) === "warn"}
                      class:pill-err={resultTone(vote.result) === "err"}
                    >
                      {vote.result.split(" ")[0]}
                    </span>
                  </div>
                </div>
              {/each}
              <div class="rounded-xl border border-border bg-surface px-3 py-3">
                <span class="card-label">Stop Reason</span>
                <p class="mt-1 text-sm text-bright">{result.stop_reason}</p>
              </div>
            </div>
          {:else}
            <div class="callout callout-neutral">
              <strong>Four honest exits</strong>
              <p>Converged, budget exhausted, policy blocked, or escalated to a human.</p>
            </div>
          {/if}
        </section>

        <section class="rounded-2xl border border-border bg-deep p-4">
          <div class="mb-3 flex items-start justify-between gap-3">
            <div>
              <span class="card-label">Foundation Pressure</span>
              <h3 class="mt-1 font-display text-xl font-semibold text-bright">What this run asks from the stack</h3>
            </div>
            <span class="pill pill-info">{result ? "Observed" : "Planned"}</span>
          </div>
          <div class="space-y-2">
            {#each foundationPressure as row}
              <article class="rounded-xl border border-border bg-raised px-3 py-2">
                <div class="flex items-center justify-between gap-3">
                  <strong class="text-sm text-bright">{row.layer}</strong>
                  <span class="font-mono text-[0.68rem] text-muted">{row.version}</span>
                </div>
                <p class="mt-1 text-xs leading-relaxed text-subtle">{row.contract}</p>
                <p class="mt-2 rounded-lg border border-border bg-deep px-2 py-2 text-xs text-text">{row.demo_signal}</p>
                <p class="mt-2 text-xs leading-relaxed text-lime">{row.pressure}</p>
              </article>
            {/each}
          </div>
        </section>
      </aside>
    </section>
  </main>
</div>
