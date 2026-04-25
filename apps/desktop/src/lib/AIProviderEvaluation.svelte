<script lang="ts">
  import { onDestroy } from "svelte";
  import { randomVerb } from "./spinner";

  let {
    onBack = () => {},
    onApps = () => {},
    onSpecStudio = () => {},
  }: {
    onBack?: () => void;
    onApps?: () => void;
    onSpecStudio?: () => void;
  } = $props();

  type BootstrapMode = "upload" | "sample";
  type EvaluationMode = "today" | "creative";
  type RunState = "bootstrap" | "running" | "hitl" | "finished";

  interface EvaluationDoc {
    name: string;
    kind: string;
    size: string;
    href?: string;
  }

  interface ExpectedDoc {
    title: string;
    purpose: string;
    requiredInformation: string;
    examples: string;
  }

  interface EvaluationStep {
    step: string;
    detail: string;
    agent: string;
    purpose: string;
    active: boolean;
  }

  interface FormationAgent {
    name: string;
    kind: string;
    purpose: string;
    source: string;
  }

  let runState = $state<RunState>("bootstrap");
  let bootstrapMode = $state<BootstrapMode>("upload");
  let evaluationMode = $state<EvaluationMode>("today");
  let documents = $state<EvaluationDoc[]>([]);
  let executableReady = $state(false);
  let steps = $state<EvaluationStep[]>([]);
  let spinnerVerb = $state(randomVerb());
  let delegateToCedar = $state(true);
  let approver = $state("procurement.review@buyer.example");
  let approvalNote = $state("Evidence package is sufficient for the demo threshold.");
  let installedCedar = $state(false);
  let expectedDocsOpen = $state(false);
  let timers: ReturnType<typeof setTimeout>[] = [];
  let spinnerInterval: ReturnType<typeof setInterval> | null = null;

  const expectedDocs: ExpectedDoc[] = [
    {
      title: "RFI or RFP",
      purpose: "Defines the buyer need and vendor response format.",
      requiredInformation: "Scope, success criteria, expected usage, deadlines, evaluation weights.",
      examples: "RFP PDF, RFI spreadsheet, requirements document",
    },
    {
      title: "AI Usage Profile",
      purpose: "Tells formation what the providers must support.",
      requiredInformation: "Workloads, latency tolerance, data sensitivity, regions, expected volume.",
      examples: "Architecture notes, prompt inventory, agent workflow map",
    },
    {
      title: "Security and Compliance Requirements",
      purpose: "Controls what can be promoted without escalation.",
      requiredInformation: "Data residency, retention, audit, certifications, vendor risk rules.",
      examples: "DPA template, SOC2 requirement, internal risk policy",
    },
    {
      title: "Pricing and Budget Constraints",
      purpose: "Lets optimization compare real cost envelopes instead of list-price guesses.",
      requiredInformation: "Budget ceiling, volume bands, token mix, gateway costs, discount assumptions.",
      examples: "Budget memo, pricing sheets, current spend extract",
    },
    {
      title: "Current Platform Context",
      purpose: "Shows whether a router or gateway is part of the correct answer.",
      requiredInformation: "Existing Kong/OpenRouter usage, observability needs, policy enforcement points.",
      examples: "System diagram, platform standard, integration constraints",
    },
  ];

  const sampleDocs: EvaluationDoc[] = [
    {
      name: "01-xyz-vendor-selection-process.md",
      kind: "RFI/RFP",
      size: "2.1 KB",
      href: "/demo/ai-provider-evaluation/fast-track/01-xyz-vendor-selection-process.md",
    },
    {
      name: "02-ai-workload-profile.md",
      kind: "Usage Profile",
      size: "1.5 KB",
      href: "/demo/ai-provider-evaluation/fast-track/02-ai-workload-profile.md",
    },
    {
      name: "03-security-compliance-requirements.md",
      kind: "Compliance",
      size: "1.4 KB",
      href: "/demo/ai-provider-evaluation/fast-track/03-security-compliance-requirements.md",
    },
    {
      name: "04-budget-and-token-forecast.csv",
      kind: "Pricing",
      size: "0.7 KB",
      href: "/demo/ai-provider-evaluation/fast-track/04-budget-and-token-forecast.csv",
    },
    {
      name: "05-gateway-architecture-notes.md",
      kind: "Platform",
      size: "1.3 KB",
      href: "/demo/ai-provider-evaluation/fast-track/05-gateway-architecture-notes.md",
    },
    {
      name: "06-executable-jtbd-and-converging-truth.md",
      kind: "Executable JTBD",
      size: "1.6 KB",
      href: "/demo/ai-provider-evaluation/fast-track/06-executable-jtbd-and-converging-truth.md",
    },
  ];

  const formationAgents: FormationAgent[] = [
    {
      name: "Bootstrap Mapper",
      kind: "knowledgebase + schema",
      purpose: "Extract required information and identify missing inputs.",
      source: "Uploaded documents",
    },
    {
      name: "Formation Planner",
      kind: "optimizer",
      purpose: "Select roles and delegation chain without hard-coding provider/model details.",
      source: "Needs and constraints",
    },
    {
      name: "Wide Evidence Agent",
      kind: "Brave search",
      purpose: "Find broad market, vendor, and incident signals.",
      source: "Web breadth",
    },
    {
      name: "Deep Evidence Agent",
      kind: "Tavily search",
      purpose: "Ground specific claims with deeper source retrieval.",
      source: "Web depth",
    },
    {
      name: "Compliance Agent",
      kind: "policy + LLM",
      purpose: "Map vendors against security, data, and audit requirements.",
      source: "Policy evidence",
    },
    {
      name: "Price Optimizer",
      kind: "math/optimization",
      purpose: "Compare model mix, router cost, and escalation budgets.",
      source: "Cost model",
    },
    {
      name: "Risk Skeptic",
      kind: "statistics + review",
      purpose: "Challenge weak evidence and detect unresolved ambiguity.",
      source: "Promoted facts",
    },
    {
      name: "Consensus Promoter",
      kind: "Converge",
      purpose: "Promote only evidence that survives gates into the record.",
      source: "Governed context",
    },
  ];

  const pipelineSteps: Omit<EvaluationStep, "active">[] = [
    {
      step: "Bootstrapping",
      detail: "Read the document package and derive the information model.",
      agent: "Bootstrap Mapper",
      purpose: "Turn documents into typed needs, constraints, and missing inputs.",
    },
    {
      step: "Formation",
      detail: "Create a governed team for compliance, price, risk, search, and consensus.",
      agent: "Formation Planner",
      purpose: "Express needs at the top level and let lower layers pick the right combo.",
    },
    {
      step: "Wide Search",
      detail: "Use Brave to discover providers, incidents, pricing pages, and integration signals.",
      agent: "Wide Evidence Agent",
      purpose: "Avoid local-minimum evidence by getting broad coverage first.",
    },
    {
      step: "Deep Search",
      detail: "Use Tavily to verify the strongest claims and source specific numbers.",
      agent: "Deep Evidence Agent",
      purpose: "Ground the candidate facts before they become shared evidence.",
    },
    {
      step: "Role Analysis",
      detail: "Score the workload against discussion, synthesis, polishing, escalation, and gateway needs.",
      agent: "Compliance Agent + Price Optimizer + Risk Skeptic",
      purpose: "Compare model mix and router architecture against the buyer constraints.",
    },
    {
      step: "HITL Gate",
      detail: "Human approval required before promoting the shortlist recommendation.",
      agent: "Cedar Gate",
      purpose: "Capture authority now and optionally delegate the same pattern next time.",
    },
    {
      step: "Fixed Point",
      detail: "No new promotable facts remain under the current context, budget, and policy.",
      agent: "Consensus Promoter",
      purpose: "Stop when the governed record is stable, not when a model feels done.",
    },
    {
      step: "Result",
      detail: "Present the evaluated provider setup and governance rationale.",
      agent: "Decision Projection",
      purpose: "Show the final recommendation from promoted evidence only.",
    },
  ];

  let documentPackageReady = $derived(documents.length >= 3);
  let canStart = $derived(documentPackageReady && executableReady);
  let bootstrapLabel = $derived(
    bootstrapMode === "sample"
      ? "Fast track package loaded"
      : `${documents.length}/6 documents loaded`,
  );
  let documentReadinessLabel = $derived(documentPackageReady ? "Documents ready" : "Needs 3 docs");
  let documentReadinessDetail = $derived(
    documentPackageReady
      ? "Minimum document package is present."
      : `${Math.max(0, 3 - documents.length)} more document${3 - documents.length === 1 ? "" : "s"} needed.`,
  );
  let documentProgressPercent = $derived(Math.min(100, (documents.length / 3) * 100));
  let executableReadinessLabel = $derived(
    executableReady ? "Executable JTBD ready" : "JTBD not executable yet",
  );
  let executableReadinessDetail = $derived(
    executableReady
      ? "A runnable job, authority boundary, evidence gates, and converging Truth are declared."
      : "Confirm that the package contains an executable job-to-be-done and a Truth that can converge.",
  );
  let progressPercent = $derived(
    Math.max(8, (steps.filter((step) => !step.active).length / pipelineSteps.length) * 100),
  );
  let activeAgent = $derived(steps.find((step) => step.active)?.agent ?? "");

  const cedarPreview = $derived(`permit(
  principal in Group::"procurement_review",
  action == Action::"promote_ai_provider_recommendation",
  resource
) when {
  resource.evaluation == "AI Provider Evaluation" &&
  resource.risk <= "medium" &&
  resource.budget_delta <= 0.15 &&
  resource.evidence.coverage >= 0.80
};`);

  function useSampleProcess() {
    bootstrapMode = "sample";
    documents = sampleDocs;
    executableReady = true;
    installedCedar = false;
  }

  function toggleFastLoad(enabled: boolean) {
    if (enabled) {
      useSampleProcess();
      return;
    }
    bootstrapMode = "upload";
    documents = [];
    executableReady = false;
    installedCedar = false;
  }

  function handleFiles(fileList: FileList | null) {
    if (!fileList) return;
    bootstrapMode = "upload";
    documents = Array.from(fileList)
      .slice(0, 5)
      .map((file) => ({
        name: file.name,
        kind: inferKind(file.name),
        size: formatBytes(file.size),
      }));
    executableReady = false;
    installedCedar = false;
  }

  function handleFileInput(event: Event) {
    handleFiles((event.currentTarget as HTMLInputElement).files);
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    handleFiles(event.dataTransfer?.files ?? null);
  }

  function inferKind(name: string) {
    const lower = name.toLowerCase();
    if (lower.includes("rfp") || lower.includes("rfi")) return "RFI/RFP";
    if (lower.includes("security") || lower.includes("compliance")) return "Compliance";
    if (lower.includes("price") || lower.includes("budget") || lower.includes("cost")) return "Pricing";
    if (lower.includes("architecture") || lower.includes("platform") || lower.includes("gateway")) return "Platform";
    if (lower.includes("usage") || lower.includes("workload")) return "Usage Profile";
    return "Supporting Evidence";
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function clearRunTimers() {
    timers.forEach(clearTimeout);
    timers = [];
    if (spinnerInterval) {
      clearInterval(spinnerInterval);
      spinnerInterval = null;
    }
  }

  function pushStep(index: number) {
    const step = pipelineSteps[index];
    steps = [
      ...steps.map((item) => ({ ...item, active: false })),
      { ...step, active: true },
    ];
  }

  function startEvaluation() {
    if (!canStart || evaluationMode !== "today") return;
    clearRunTimers();
    runState = "running";
    steps = [];
    installedCedar = false;
    spinnerVerb = randomVerb();
    spinnerInterval = setInterval(() => {
      spinnerVerb = randomVerb();
    }, 1800);

    for (let index = 0; index <= 5; index += 1) {
      const timer = setTimeout(() => {
        if (runState !== "running") return;
        pushStep(index);
        if (index === 5) {
          runState = "hitl";
          if (spinnerInterval) {
            clearInterval(spinnerInterval);
            spinnerInterval = null;
          }
        }
      }, index * 1300);
      timers.push(timer);
    }
  }

  function approveHitl() {
    installedCedar = delegateToCedar;
    runState = "running";
    spinnerVerb = randomVerb();
    spinnerInterval = setInterval(() => {
      spinnerVerb = randomVerb();
    }, 1800);

    [6, 7].forEach((index, offset) => {
      const timer = setTimeout(() => {
        if (runState !== "running") return;
        pushStep(index);
        if (index === 7) {
          steps = steps.map((step) => ({ ...step, active: false }));
          runState = "finished";
          clearRunTimers();
        }
      }, (offset + 1) * 1300);
      timers.push(timer);
    });
  }

  function resetEvaluation() {
    clearRunTimers();
    runState = "bootstrap";
    steps = [];
    installedCedar = false;
    approvalNote = "Evidence package is sufficient for the demo threshold.";
  }

  onDestroy(clearRunTimers);
</script>

<section class="min-h-screen bg-void">
  <header class="flex items-center justify-between border-b border-border px-8 py-4">
    <div class="flex items-center gap-4">
      <button class="btn-ghost text-sm" type="button" onclick={onBack}>&larr; Slides</button>
      <span class="font-mono text-xs tracking-widest text-muted uppercase">AI Provider Evaluation</span>
    </div>
    <div class="flex items-center gap-3">
      <button class="btn-ghost text-sm" type="button" onclick={onSpecStudio}>Spec Studio</button>
      <button class="btn-ghost text-sm" type="button" onclick={onApps}>Apps</button>
    </div>
  </header>

  {#if runState === "bootstrap"}
    <div class="mx-auto grid max-w-6xl gap-8 px-8 py-10 lg:grid-cols-[0.95fr_1.05fr]">
      <section>
        <p class="slide-eyebrow mb-3">Bootstrapping</p>
        <h1 class="slide-headline mb-5 text-4xl!">Start with the decision package.</h1>

        <div class="mb-0 flex items-end gap-1 border-b border-border">
          <div class="flex gap-1">
            <button
              type="button"
              class="-mb-px rounded-t-xl border border-border px-5 py-2 text-sm font-semibold transition"
              class:border-b-raised={evaluationMode === "today"}
              class:bg-raised={evaluationMode === "today"}
              class:bg-deep={evaluationMode !== "today"}
              class:text-bright={evaluationMode === "today"}
              class:text-subtle={evaluationMode !== "today"}
              onclick={() => (evaluationMode = "today")}
            >
              Today
            </button>
            <button
              type="button"
              class="-mb-px rounded-t-xl border border-border px-5 py-2 text-sm font-semibold transition"
              class:border-b-raised={evaluationMode === "creative"}
              class:bg-raised={evaluationMode === "creative"}
              class:bg-deep={evaluationMode !== "creative"}
              class:text-bright={evaluationMode === "creative"}
              class:text-subtle={evaluationMode !== "creative"}
              onclick={() => (evaluationMode = "creative")}
            >
              Future
            </button>
          </div>
        </div>
        <div class="mb-6 rounded-b-2xl rounded-tr-2xl border border-t-0 border-border bg-raised p-4">
          {#if evaluationMode === "today"}
            <p class="text-sm text-subtle">Governed selection inside the current RFI/RFP frame.</p>
          {:else}
            <p class="text-sm text-subtle">Preview: challenge the premise and explore Pareto breakouts.</p>
          {/if}
        </div>

        {#if evaluationMode === "creative"}
          <div class="mb-6 rounded-2xl border border-warn/25 bg-warn/5 p-4">
            <span class="card-label text-warn!">Preview Only</span>
            <h2 class="mt-1 font-display text-xl font-semibold text-bright">Creative mode is deliberately parked.</h2>
            <p class="mt-2 text-sm text-subtle">
              This will be the mode where the system can challenge the original vendor-only frame and propose a router, gateway, or multi-provider Pareto breakout. For now, the runnable path is As of today.
            </p>
          </div>
        {/if}

        <div class="grid gap-3">
          <div
            role="region"
            aria-label="Document dropbox"
            class="rounded-2xl border border-dashed border-border bg-raised p-5 transition hover:border-subtle"
            ondrop={handleDrop}
            ondragover={(event) => event.preventDefault()}
          >
            <div class="flex items-start justify-between gap-4">
              <label class="block flex-1 cursor-pointer">
                <input
                  type="file"
                  class="hidden"
                  multiple
                  accept=".pdf,.doc,.docx,.md,.txt,.csv,.xlsx,.json"
                  onchange={handleFileInput}
                />
                <span class="card-label">Document Dropbox</span>
                <h2 class="mt-1 font-display text-xl font-semibold text-bright">Drop 3-5 buyer documents</h2>
                <p class="mt-1 text-sm text-subtle">RFI/RFP, workload profile, security requirements, pricing constraints, platform context.</p>
                <p class="mt-3 font-mono text-xs text-muted">Click to browse or drag files here. This demo records names and document roles only.</p>
              </label>
              <button
                type="button"
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-border bg-deep font-mono text-sm text-subtle transition hover:border-lime/50 hover:text-lime"
                aria-label="Show expected document set"
                onclick={() => (expectedDocsOpen = true)}
              >
                ?
              </button>
            </div>

            <div class="mt-5 grid gap-3 md:grid-cols-2">
              <label class="flex items-center gap-3 rounded-xl border border-border bg-deep px-3 py-2">
                <input
                  class="accent-lime"
                  type="checkbox"
                  checked={bootstrapMode === "sample"}
                  onchange={(event) => toggleFastLoad((event.currentTarget as HTMLInputElement).checked)}
                />
                <span class="text-sm text-text">Fast load</span>
              </label>

              <label class="flex items-start gap-3 rounded-xl border border-border bg-deep px-3 py-2">
                <input class="mt-1 accent-lime" type="checkbox" bind:checked={executableReady} />
                <span>
                  <span class="block text-sm text-text">Executable JTBD + converging Truth</span>
                  <span class="block text-xs text-muted">Ready to run as a governed job.</span>
                </span>
              </label>
            </div>

            <div class="mt-3 grid gap-3 md:grid-cols-2">
              <div class="rounded-xl border border-border bg-deep px-3 py-2">
                <div class="flex items-center gap-2">
                  <span class="h-2.5 w-2.5 rounded-full" class:bg-ok={documentPackageReady} class:bg-warn={!documentPackageReady}></span>
                  <span class="font-display text-sm font-semibold text-bright">{documentReadinessLabel}</span>
                </div>
                <p class="mt-1 text-xs text-muted">{documentReadinessDetail}</p>
              </div>

              <div class="rounded-xl border border-border bg-deep px-3 py-2">
                <div class="flex items-center gap-2">
                  <span class="h-2.5 w-2.5 rounded-full" class:bg-ok={executableReady} class:bg-warn={!executableReady}></span>
                  <span class="font-display text-sm font-semibold text-bright">{executableReadinessLabel}</span>
                </div>
                <p class="mt-1 text-xs text-muted">{executableReadinessDetail}</p>
              </div>
            </div>
          </div>
        </div>

        <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
          <div class="mb-3 flex items-center justify-between gap-4">
            <span class="card-label">Uploaded Documents</span>
            <span class="pill" class:pill-ok={documentPackageReady} class:pill-warn={!documentPackageReady}>{bootstrapLabel}</span>
          </div>
          {#if documents.length > 0}
            <div class="space-y-2">
              {#each documents as document}
                <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-raised px-3 py-2">
                  <div class="min-w-0">
                    <p class="truncate text-sm text-bright">{document.name}</p>
                    <p class="text-xs text-muted">{document.kind}</p>
                  </div>
                  <div class="flex shrink-0 items-center gap-2">
                    {#if document.href}
                      <a class="font-mono text-xs text-lime hover:underline" href={document.href} target="_blank" rel="noreferrer">Open</a>
                    {/if}
                    <span class="font-mono text-xs text-subtle">{document.size}</span>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-sm text-subtle">No uploaded documents yet.</p>
          {/if}
        </div>

        <button class="btn-lime mt-5 w-full justify-center py-3" type="button" disabled={!canStart || evaluationMode !== "today"} onclick={startEvaluation}>
          Create Formation
        </button>
      </section>

      <section class="rounded-[28px] border border-border bg-panel p-5">
        <span class="card-label">Readiness</span>
        <h2 class="mt-1 font-display text-2xl font-semibold text-bright">Bootstrap status</h2>
        <div class="mt-5 rounded-2xl border border-border bg-raised p-5">
          <div class="flex items-center justify-between gap-4">
            <div>
              <p class="font-display text-xl font-semibold text-bright">{documentReadinessLabel}</p>
              <p class="mt-1 text-sm text-subtle">{documentReadinessDetail}</p>
            </div>
            <span class="pill" class:pill-ok={documentPackageReady} class:pill-warn={!documentPackageReady}>{bootstrapLabel}</span>
          </div>
          <div class="mt-5 h-2 overflow-hidden rounded-full bg-border">
            <div class="h-full rounded-full bg-lime transition-all duration-500" style="width: {documentProgressPercent}%"></div>
          </div>
          <button class="btn-ghost mt-5 w-full justify-center" type="button" onclick={() => (expectedDocsOpen = true)}>
            What documents are expected?
          </button>
        </div>

        <div class="mt-4 rounded-2xl border border-border bg-raised p-5">
          <div class="flex items-start gap-3">
            <input class="mt-1 accent-lime" type="checkbox" bind:checked={executableReady} />
            <div>
              <p class="font-display text-xl font-semibold text-bright">{executableReadinessLabel}</p>
              <p class="mt-1 text-sm text-subtle">{executableReadinessDetail}</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  {:else}
    <div class="mx-auto grid max-w-7xl gap-6 px-8 py-8 lg:grid-cols-[0.9fr_1.1fr]">
      <section class="rounded-[28px] border border-border bg-panel p-5">
        <div class="mb-5 flex items-start justify-between gap-4">
          <div>
            <p class="slide-eyebrow mb-2">{runState === "finished" ? "Fixed Point" : runState === "hitl" ? "Human Gate" : "Formation In Work"}</p>
            <h1 class="slide-headline text-3xl!">AI Provider Evaluation</h1>
            <p class="mt-2 text-sm text-subtle">{bootstrapLabel}</p>
          </div>
          {#if runState === "finished"}
            <span class="pill pill-ok">Converged</span>
          {:else if runState === "hitl"}
            <span class="pill pill-warn">Approval Needed</span>
          {:else}
            <span class="pill pill-info">{activeAgent || "Starting"}</span>
          {/if}
        </div>

        <div class="mb-6 space-y-3">
          {#each steps as step}
            <div class="flex items-start gap-3 fade-in">
              <div
                class="mt-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-full"
                class:bg-lime-glow={step.active}
                class:text-lime={step.active}
                style={!step.active ? "background: rgba(52,211,153,0.15); color: var(--color-ok)" : ""}
              >
                {#if step.active}
                  <span class="inline-block h-2 w-2 animate-pulse rounded-full bg-lime"></span>
                {:else}
                  <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                {/if}
              </div>
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <span class="text-sm font-semibold" class:text-bright={step.active} class:text-subtle={!step.active}>{step.step}</span>
                  <span class="rounded-full border border-border px-2 py-0.5 font-mono text-[0.65rem] text-muted">{step.agent}</span>
                </div>
                <p class="mt-1 text-xs text-subtle">{step.detail}</p>
                <p class="mt-1 text-xs text-muted">{step.purpose}</p>
              </div>
            </div>
          {/each}
        </div>

        {#if runState === "running"}
          <div class="h-1 overflow-hidden rounded-full bg-border">
            <div class="h-full rounded-full bg-lime transition-all duration-1000" style="width: {progressPercent}%"></div>
          </div>
          <p class="mt-4 text-center text-sm text-muted">{spinnerVerb}...</p>
        {/if}

        {#if runState === "hitl"}
          <form class="mt-5 rounded-2xl border border-warn/30 bg-warn/5 p-4" onsubmit={(event) => { event.preventDefault(); approveHitl(); }}>
            <span class="card-label text-warn!">HITL Form</span>
            <h2 class="mt-1 font-display text-xl font-semibold text-bright">Promote shortlist recommendation?</h2>
            <p class="mt-2 text-sm text-subtle">
              The system has enough evidence for a demo-threshold recommendation, but authority still requires review.
            </p>

            <div class="mt-4 grid gap-3">
              <label class="block">
                <span class="card-label mb-1 block">Approver</span>
                <input class="w-full rounded-xl border border-border bg-deep px-3 py-2 text-sm text-text focus:border-lime/50 focus:outline-none" bind:value={approver} />
              </label>
              <label class="block">
                <span class="card-label mb-1 block">Decision Note</span>
                <textarea class="min-h-20 w-full rounded-xl border border-border bg-deep px-3 py-2 text-sm text-text focus:border-lime/50 focus:outline-none" bind:value={approvalNote}></textarea>
              </label>
              <label class="flex items-start gap-3 rounded-xl border border-lime/20 bg-lime-glow p-3">
                <input class="mt-1 accent-lime" type="checkbox" bind:checked={delegateToCedar} />
                <span>
                  <strong class="block text-sm text-bright">Auto-approve next time with Cedar</strong>
                  <span class="text-xs text-subtle">Replace this human gate when the same threshold, evidence coverage, and risk class recur.</span>
                </span>
              </label>
            </div>

            {#if delegateToCedar}
              <pre class="mt-4 overflow-auto rounded-xl border border-border bg-deep p-3 font-mono text-xs leading-relaxed text-subtle">{cedarPreview}</pre>
            {/if}

            <button class="btn-lime mt-4 w-full justify-center" type="submit">Approve And Continue</button>
          </form>
        {/if}

        {#if runState === "finished"}
          <div class="mt-5 flex gap-3">
            <button class="btn-lime" type="button" onclick={resetEvaluation}>Run Again</button>
            <button class="btn-ghost" type="button" onclick={onSpecStudio}>Open Spec Studio</button>
          </div>
        {/if}
      </section>

      <aside class="space-y-5">
        {#if runState !== "finished"}
          <section class="rounded-[28px] border border-border bg-panel p-5">
            <span class="card-label">Formation</span>
            <h2 class="mt-1 font-display text-2xl font-semibold text-bright">Agents and purpose</h2>
            <div class="mt-4 grid gap-3 md:grid-cols-2">
              {#each formationAgents as agent}
                <article class="rounded-2xl border border-border bg-raised p-4" class:ring-1={activeAgent === agent.name} class:ring-lime={activeAgent === agent.name}>
                  <div class="flex items-start justify-between gap-3">
                    <h3 class="font-display text-sm font-semibold text-bright">{agent.name}</h3>
                    <span class="pill pill-info">{agent.kind}</span>
                  </div>
                  <p class="mt-2 text-xs text-subtle">{agent.purpose}</p>
                  <p class="mt-2 font-mono text-[0.68rem] text-muted">{agent.source}</p>
                </article>
              {/each}
            </div>
          </section>
        {:else}
          <section class="rounded-[28px] border border-lime/30 bg-lime-glow p-5">
            <span class="card-label text-lime!">Result</span>
            <h2 class="mt-1 font-display text-3xl font-semibold text-bright">Governed provider mix behind a router.</h2>
            <p class="mt-3 text-sm text-subtle">
              The evaluation did not choose one model to do everything. It promoted a mixed setup because different roles want different latency, cost, and reasoning profiles.
            </p>

            <div class="mt-5 grid gap-3 md:grid-cols-2">
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Primary</span>
                <p class="mt-2 text-sm text-bright">Arcee Trinity or Mistral Small</p>
                <p class="mt-1 text-xs text-subtle">Discussion, extraction, and synthesis for most runs.</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Polishing</span>
                <p class="mt-2 text-sm text-bright">Writer Palmyra</p>
                <p class="mt-1 text-xs text-subtle">Stakeholder-ready language and executive summaries.</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Escalation</span>
                <p class="mt-2 text-sm text-bright">Claude or GPT Pro</p>
                <p class="mt-1 text-xs text-subtle">Reserved for high-risk or high-ambiguity decisions.</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Gateway</span>
                <p class="mt-2 text-sm text-bright">Kong or OpenRouter</p>
                <p class="mt-1 text-xs text-subtle">Routing, audit, rate limits, cost control, and provider flexibility.</p>
              </div>
            </div>

            <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
              <span class="card-label">Governance Record</span>
              <div class="mt-3 grid gap-2">
                <p class="text-sm text-text">Formation selected 8 acting agents.</p>
                <p class="text-sm text-text">HITL approval captured from {approver}.</p>
                <p class="text-sm text-text">{installedCedar ? "Cedar delegation installed for the next matching run." : "Human approval remains required for the next run."}</p>
                <p class="text-sm text-text">Fixed point reached after promoted evidence stopped changing.</p>
              </div>
            </div>
          </section>
        {/if}
      </aside>
    </div>
  {/if}

  {#if expectedDocsOpen}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-void/84 px-4 py-8 backdrop-blur-sm"
      role="presentation"
      onclick={() => (expectedDocsOpen = false)}
      onkeydown={(event) => event.key === "Escape" && (expectedDocsOpen = false)}
    >
      <div
        class="max-h-[90vh] w-full max-w-5xl overflow-auto rounded-[28px] border border-border bg-deep p-6 shadow-2xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="expected-docs-title"
        tabindex="-1"
        onclick={(event) => event.stopPropagation()}
        onkeydown={(event) => event.stopPropagation()}
      >
        <div class="mb-5 flex items-start justify-between gap-4">
          <div>
            <span class="card-label">Expected Document Set</span>
            <h2 id="expected-docs-title" class="mt-1 font-display text-2xl font-semibold text-bright">What information must exist?</h2>
            <p class="mt-2 text-sm text-subtle">The system can start with three documents, but the best run has all five input categories.</p>
          </div>
          <button class="btn-ghost text-sm" type="button" onclick={() => (expectedDocsOpen = false)}>Close</button>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          {#each expectedDocs as doc}
            <article class="rounded-2xl border border-border bg-raised p-4">
              <div class="flex items-start justify-between gap-3">
                <h3 class="font-display text-base font-semibold text-bright">{doc.title}</h3>
                <span class="pill pill-info">Input</span>
              </div>
              <p class="mt-2 text-sm text-subtle">{doc.purpose}</p>
              <p class="mt-3 text-xs text-text"><span class="text-muted">Needed:</span> {doc.requiredInformation}</p>
              <p class="mt-1 text-xs text-muted">Examples: {doc.examples}</p>
            </article>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</section>
