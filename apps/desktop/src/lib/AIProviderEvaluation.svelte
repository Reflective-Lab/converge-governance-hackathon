<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { FlowPlayer, ReplayRunner } from "@reflective/helm-flow";
  import HitlGate from "@reflective/helm-flow/src/HitlGate.svelte";
  import type { RunState as HelmRunState } from "@reflective/helm-flow";
  import { invokeTauri } from "./tauri";
  import { randomVerb } from "./spinner";
  import { VendorSelectionReplayAdapter, statusFromReplaySession, emptyReplayStatus } from "./replay-adapter";
  import type {
    BootstrapMode,
    EvaluationMode,
    EvaluationDoc,
    ExpectedDoc,
    EvaluationStep,
    FormationAgent,
    VendorInput,
    ExperienceSnapshot,
    TodayRunResponse,
    TodayRecordedRun,
    TodayReplaySession,
    TodayReplayStatus,
  } from "./types";

  let {
    onBack = () => {},
    onApps = () => {},
    onSpecStudio = () => {},
  }: {
    onBack?: () => void;
    onApps?: () => void;
    onSpecStudio?: () => void;
  } = $props();

  // Vendor-selection-specific run mode (extends Helm's RunMode)
  type DemoRunMode = "mock" | "replay" | "live";

  // Vendor-selection-specific run state (extends Helm's RunState with 'commit-review')
  type RunState = HelmRunState | "commit-review";
  const PRESENTATION_STEP_DELAY_MS = 4_000;

  // Inline vendor arrays for adapter initialization (moved later in component for full definition)
  const todayVendorsForAdapter: Array<Record<string, any>> = [];
  const creativeVendorsForAdapter: Array<Record<string, any>> = [];

  // Replay adapter and runner (initialized before state, but vendor data comes later)
  const replayAdapter = new VendorSelectionReplayAdapter(
    todayVendorsForAdapter,
    creativeVendorsForAdapter,
  );
  const replayRunner = new ReplayRunner(replayAdapter as any);

  // Domain state
  let bootstrapMode = $state<BootstrapMode>("sample");
  let evaluationMode = $state<EvaluationMode>("today");
  let documents = $state<EvaluationDoc[]>([]);
  let executableReady = $state(true);
  let runMode = $state<DemoRunMode>("mock");
  let replayStatus = $state<TodayReplayStatus | null>(null);
  let replayStatusLoaded = $state(false);
  let delegateToCedar = $state(true);
  let approver = $state("procurement.review@buyer.example");
  let approvalNote = $state("Evidence package is sufficient for the demo threshold.");
  let installedCedar = $state(false);
  let expectedDocsOpen = $state(false);
  let flowError = $state("");
  let analysisRun = $state<TodayRunResponse | null>(null);
  let approvedRun = $state<TodayRunResponse | null>(null);
  let negativeControlRun = $state<TodayRunResponse | null>(null);
  let learningRuns = $state<TodayRunResponse[]>([]);
  let experience = $state<ExperienceSnapshot | null>(null);
  let spinnerInterval: ReturnType<typeof setInterval> | null = null;
  let presentationFrame: number | null = null;
  let presentationSequence = 0;
  let presentationProgressPercent = $state(8);
  let runningSegmentStart = $state(0);
  let runningSegmentEnd = $state(5);
  let runAnimationKey = $state(0);
  let autoStarted = false;
  let mockExperienceRunCount = 0;

  // Convenience aliases
  let recordingReplay = $derived(replayAdapter.recordingInProgress);
  let buildingOfflineReplay = $derived(replayAdapter.buildingOfflineBackup);

  const todayVendors: VendorInput[] = [
    {
      name: "Anthropic",
      score: 92,
      risk_score: 12,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 4800000,
      currency_code: "USD",
    },
    {
      name: "OpenAI",
      score: 90,
      risk_score: 18,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 5200000,
      currency_code: "USD",
    },
    {
      name: "Google DeepMind",
      score: 88,
      risk_score: 15,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR", "FedRAMP"],
      monthly_cost_minor: 4500000,
      currency_code: "USD",
    },
    {
      name: "Mistral",
      score: 82,
      risk_score: 22,
      compliance_status: "compliant",
      certifications: ["SOC2", "GDPR"],
      monthly_cost_minor: 2200000,
      currency_code: "USD",
    },
    {
      name: "Qwen (Alibaba Cloud)",
      score: 79,
      risk_score: 35,
      compliance_status: "pending",
      certifications: ["ISO27001"],
      monthly_cost_minor: 1800000,
      currency_code: "USD",
    },
  ];

  const creativeVendors: VendorInput[] = [
    {
      name: "Anthropic Claude Enterprise",
      score: 96,
      risk_score: 14,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 7600000,
      currency_code: "USD",
    },
    {
      name: "OpenAI Enterprise",
      score: 94,
      risk_score: 18,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 6800000,
      currency_code: "USD",
    },
    {
      name: "Google Vertex AI",
      score: 90,
      risk_score: 16,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR", "FedRAMP"],
      monthly_cost_minor: 6200000,
      currency_code: "USD",
    },
    {
      name: "Mistral EU",
      score: 84,
      risk_score: 20,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 2800000,
      currency_code: "USD",
    },
    {
      name: "Cohere Enterprise",
      score: 82,
      risk_score: 18,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 3600000,
      currency_code: "USD",
    },
    {
      name: "Fireworks AI",
      score: 78,
      risk_score: 24,
      compliance_status: "compliant",
      certifications: ["SOC2"],
      monthly_cost_minor: 1800000,
      currency_code: "USD",
    },
    {
      name: "Qwen Alibaba Cloud",
      score: 86,
      risk_score: 38,
      compliance_status: "pending",
      certifications: ["ISO27001"],
      monthly_cost_minor: 1600000,
      currency_code: "USD",
    },
    {
      name: "Kong AI Gateway",
      score: 80,
      risk_score: 12,
      compliance_status: "compliant",
      certifications: ["SOC2", "ISO27001", "GDPR"],
      monthly_cost_minor: 2400000,
      currency_code: "USD",
    },
    {
      name: "Brave Search API",
      score: 76,
      risk_score: 16,
      compliance_status: "compliant",
      certifications: ["SOC2", "GDPR"],
      monthly_cost_minor: 800000,
      currency_code: "USD",
    },
    {
      name: "Tavily Research API",
      score: 78,
      risk_score: 14,
      compliance_status: "compliant",
      certifications: ["SOC2", "GDPR"],
      monthly_cost_minor: 1200000,
      currency_code: "USD",
    },
  ];

  // Populate adapter with vendor data
  todayVendorsForAdapter.push(...todayVendors);
  creativeVendorsForAdapter.push(...creativeVendors);

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
    {
      name: "Cedar Gate",
      kind: "policy",
      purpose: "Authorize promotion, escalation, rejection, and commitment.",
      source: "Cedar policy",
    },
    {
      name: "Experience Store",
      kind: "memory",
      purpose: "Replay governed context without changing hard policy boundaries.",
      source: "Prior runs",
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
      step: "Compliance Screen",
      detail: "Screen declared compliance and certifications before ranking.",
      agent: "Compliance Agent",
      purpose: "Block pending or missing evidence before it can become a commitment.",
    },
    {
      step: "Price And Risk",
      detail: "Compare cost efficiency, operational risk, and certification coverage.",
      agent: "Price Optimizer + Risk Skeptic",
      purpose: "Make the trade-off explicit instead of hiding it in a scorecard.",
    },
    {
      step: "Shortlist And Synthesis",
      detail: "Rank feasible candidates and synthesize the recommendation.",
      agent: "Consensus Promoter",
      purpose: "Promote the recommendation only from facts that survived the gates.",
    },
    {
      step: "HITL Gate",
      detail: "Human approval required before promoting the shortlist recommendation.",
      agent: "Cedar Gate",
      purpose: "Capture authority now and optionally delegate the same pattern next time.",
    },
    {
      step: "Promote Commitment",
      detail: "Rerun with human approval present and let Cedar authorize the action.",
      agent: "Cedar Gate",
      purpose: "Separate recommendation from commitment.",
    },
    {
      step: "Negative Control",
      detail: "Rerun the same evidence with advisory authority.",
      agent: "Cedar Gate",
      purpose: "Prove policy can reject a good recommendation when authority is wrong.",
    },
    {
      step: "Learning Run 1",
      detail: "Replay the accepted pattern and expose prior context.",
      agent: "Experience Store",
      purpose: "Show that learning changes context, not hard policy boundaries.",
    },
    {
      step: "Learning Run 2",
      detail: "Add another governed run to the experience registry.",
      agent: "Experience Store",
      purpose: "Measure consistency without weakening compliance or risk gates.",
    },
    {
      step: "Learning Run 3",
      detail: "Confirm the recommendation stays stable under repeated execution.",
      agent: "Experience Store",
      purpose: "Make future delegation evidence-backed.",
    },
    {
      step: "Fixed Point",
      detail: "No new promotable facts remain under the current context, budget, and policy.",
      agent: "Consensus Promoter",
      purpose: "Stop when the governed record is stable, not when a model feels done.",
    },
  ];

  // Flow orchestration (Helm-owned). Keep this after pipelineSteps; Svelte
  // initializes component script top-to-bottom.
  const flowPlayer = new FlowPlayer({
    phases: [
      {
        name: "Analysis",
        steps: pipelineSteps.slice(0, 6).map((s, i) => ({
          id: `step-${i}`,
          label: s.step,
          detail: s.detail,
          agent: s.agent,
          purpose: s.purpose,
        })),
      },
      {
        name: "HITL Gate",
        steps: [
          {
            id: "step-6",
            label: pipelineSteps[6].step,
            detail: pipelineSteps[6].detail,
            agent: pipelineSteps[6].agent,
            purpose: pipelineSteps[6].purpose,
          },
        ],
        gateName: "hitl",
      },
      {
        name: "Promotion",
        steps: pipelineSteps.slice(7).map((s, i) => ({
          id: `step-${7 + i}`,
          label: s.step,
          detail: s.detail,
          agent: s.agent,
          purpose: s.purpose,
        })),
      },
    ],
    stepDelayMs: PRESENTATION_STEP_DELAY_MS,
    reviewPauseMs: 1_800,
  });

  let flowState = $state(flowPlayer.getState());
  let runState = $state<RunState>("bootstrap");
  let spinnerVerb = $state(randomVerb());
  let presentationStepIndex = $state(0);

  let documentPackageReady = $derived(documents.length >= 3);
  let canStart = $derived(bootstrapMode === "sample" || (documentPackageReady && executableReady));
  let activeDemoMode = $derived(evaluationMode);
  let activeTruthDemoMode = $derived(
    activeDemoMode === "creative" ? "pareto-breakout" : "governed",
  );
  let activeVendors = $derived(activeDemoMode === "creative" ? creativeVendors : todayVendors);
  let activeDemoLabel = $derived(
    activeDemoMode === "creative" ? "Creative Pareto Breakout" : "Today Governed Selection",
  );
  let activeDemoShortLabel = $derived(activeDemoMode === "creative" ? "Creative" : "Today");
  let canRunToday = $derived(
    canStart &&
      (runMode === "mock" ||
        runMode === "live" ||
        (activeDemoMode === "today" && replayStatus?.available === true)),
  );
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
  let highlightedStep = $state(pipelineSteps[0]);
  let progressPercent = $derived(presentationProgressPercent);
  let activeAgent = $derived(highlightedStep?.agent ?? "");
  let activeSuggestorNames = $derived(suggestorsForStep(presentationStepIndex));
  let activeSuggestorSet = $derived(new Set(activeSuggestorNames));
  let parallelSuggestorWork = $derived(activeSuggestorNames.length > 1);
  let runningSegmentSteps = $derived(
    pipelineSteps.slice(runningSegmentStart, runningSegmentEnd + 1),
  );
  let finalRun = $derived(approvedRun ?? analysisRun);
  let finalDetails = $derived(detailsFor(finalRun));
  let finalPolicy = $derived(policyFor(finalRun));
  let beforeHitlPolicy = $derived(policyFor(analysisRun));
  let selectedVendorName = $derived(stringAt(finalPolicy, "selected_vendor") || topShortlistName(finalRun));
  let rejectedRows = $derived(rejectedFor(finalRun));
  let shortlistRows = $derived(shortlistFor(finalRun));
  let liveCallCount = $derived(totalLlmCalls());
  let learningStatus = $derived(learningFor(learningRuns[learningRuns.length - 1] ?? finalRun));
  let replayIsLive = $derived(replayStatus?.mode === "recorded-live");
  let runModeLabel = $derived(
    runMode === "mock"
      ? "Mocked presentation flow"
      : runMode === "replay"
      ? replayIsLive
        ? "Recorded live replay"
        : "Offline presentation replay"
      : "Live provider run",
  );
  let runModeDetail = $derived(
    runMode === "mock"
      ? `Runs the ${activeDemoShortLabel} path locally with deterministic agents and presenter-paced timing.`
      : runMode === "replay"
      ? replayIsLive
        ? "Uses a previously captured session with real LLM telemetry and compressed thinking delays."
        : "Uses deterministic governed outputs with compressed thinking delays because provider quotas are exhausted."
      : "Calls configured providers now and fails if any model role falls back.",
  );
  let runModeVerb = $derived(runMode === "replay" ? "replayed" : "ran");

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

  // Spinner helper (vendor-selection-specific)
  function startSpinner() {
    if (spinnerInterval) {
      clearInterval(spinnerInterval);
    }
    spinnerVerb = randomVerb();
    spinnerInterval = setInterval(() => {
      spinnerVerb = randomVerb();
    }, 1800);
  }

  function stopSpinner() {
    if (spinnerInterval) {
      clearInterval(spinnerInterval);
      spinnerInterval = null;
    }
  }

  // Sync flowState when FlowPlayer updates
  function updateFlowState() {
    flowState = flowPlayer.getState();
  }

  function setPresentationStep(index: number) {
    presentationStepIndex = Math.min(Math.max(index, 0), pipelineSteps.length - 1);
    highlightedStep = pipelineSteps[presentationStepIndex];
    updateFlowState();
  }

  function clearPresentationTimers() {
    if (presentationFrame !== null) {
      cancelAnimationFrame(presentationFrame);
      presentationFrame = null;
    }
    presentationSequence += 1;
  }

  function schedulePresentationSegment(
    startIndex: number,
    endIndex: number,
    onComplete: () => void,
  ) {
    clearPresentationTimers();
    const sequence = presentationSequence;
    const startedAt = performance.now();
    const totalDuration = Math.max(1, endIndex - startIndex + 1) * PRESENTATION_STEP_DELAY_MS;

    setPresentationStep(startIndex);
    presentationProgressPercent = Math.max(8, ((startIndex + 1) / pipelineSteps.length) * 100);

    const animate = (now: number) => {
      if (sequence !== presentationSequence) {
        return;
      }

      const elapsed = Math.max(0, now - startedAt);
      const stepOffset = Math.min(endIndex - startIndex, Math.floor(elapsed / PRESENTATION_STEP_DELAY_MS));
      const nextStep = startIndex + stepOffset;
      const continuousStep = Math.min(endIndex + 1, startIndex + elapsed / PRESENTATION_STEP_DELAY_MS);

      if (nextStep !== presentationStepIndex) {
        setPresentationStep(nextStep);
      }

      presentationProgressPercent = Math.max(
        8,
        Math.min(100, (continuousStep / pipelineSteps.length) * 100),
      );

      if (elapsed >= totalDuration) {
        clearPresentationTimers();
        setPresentationStep(endIndex);
        presentationProgressPercent = Math.max(8, ((endIndex + 1) / pipelineSteps.length) * 100);
        onComplete();
        return;
      }

      presentationFrame = requestAnimationFrame(animate);
    };

    presentationFrame = requestAnimationFrame(animate);
  }

  function startEvaluation() {
    runMode = "mock";
    useSampleProcess();

    resetMockExperience();
    installedCedar = false;
    flowError = "";
    replayCursor = {};
    analysisRun = null;
    approvedRun = null;
    negativeControlRun = null;
    learningRuns = [];
    experience = null;
    beginMockSegment(0, 5);
  }

  function openHitlDecision() {
    runState = "hitl";
  }

  function approveHitl() {
    installedCedar = delegateToCedar;
    flowError = "";
    beginMockSegment(6, 11);
  }

  function continueAfterCommitReview() {
    flowError = "";
    beginMockSegment(7, 11);
  }

  function beginMockSegment(startIndex: number, endIndex: number) {
    clearPresentationTimers();
    runningSegmentStart = startIndex;
    runningSegmentEnd = endIndex;
    runAnimationKey += 1;
    runState = "running";
    setPresentationStep(startIndex);
    presentationProgressPercent = Math.max(8, ((startIndex + 1) / pipelineSteps.length) * 100);
    startSpinner();
  }

  function completeMockSegment() {
    if (runState !== "running") return;

    setPresentationStep(runningSegmentEnd);
    presentationProgressPercent = Math.max(8, ((runningSegmentEnd + 1) / pipelineSteps.length) * 100);
    stopSpinner();

    if (runningSegmentStart === 0) {
      const response = mockTodayStage("analysis");
      analysisRun = response;
      experience = analysisRun.experience;
      runState = "gate-review";
      return;
    }

    if (runningSegmentStart === 6) {
      approvedRun = mockTodayStage("approved");
      experience = approvedRun.experience;
      negativeControlRun = mockTodayStage("negative-control");
      experience = negativeControlRun.experience;
      learningRuns = [
        mockTodayStage("learning"),
        mockTodayStage("learning"),
        mockTodayStage("learning"),
      ];
      experience = learningRuns[learningRuns.length - 1].experience;
      runState = "finished";
      return;
    }

    if (runningSegmentStart === 7) {
      negativeControlRun = mockTodayStage("negative-control");
      experience = negativeControlRun.experience;
      learningRuns = [
        mockTodayStage("learning"),
        mockTodayStage("learning"),
        mockTodayStage("learning"),
      ];
      experience = learningRuns[learningRuns.length - 1].experience;
      runState = "finished";
    }
  }

  function resetEvaluation() {
    flowPlayer.reset();
    updateFlowState();
    runState = "bootstrap";
    clearPresentationTimers();
    setPresentationStep(0);
    presentationProgressPercent = 8;
    replayRunner.resetCursor();
    installedCedar = false;
    flowError = "";
    analysisRun = null;
    approvedRun = null;
    negativeControlRun = null;
    learningRuns = [];
    approvalNote = "Evidence package is sufficient for the demo threshold.";
    resetMockExperience();
  }


  async function loadReplayStatus() {
    try {
      replayStatus = await replayAdapter.getStatus();
      runMode = "mock";
    } catch (cause) {
      replayStatus = {
        ...emptyReplayStatus(),
        error: describeRunError(cause),
      };
      runMode = "mock";
    } finally {
      replayStatusLoaded = true;
    }
  }


  async function recordReplaySession() {
    flowError = "";
    try {
      await replayAdapter.recordSession();
      replayStatus = await replayAdapter.getStatus();
      runMode = "replay";
    } catch (cause) {
      flowError = describeRunError(cause);
    } finally {
      replayStatusLoaded = true;
    }
  }

  async function buildOfflineReplaySession() {
    flowError = "";
    try {
      await replayAdapter.buildOfflineSession();
      replayStatus = await replayAdapter.getStatus();
      runMode = "replay";
    } catch (cause) {
      flowError = describeRunError(cause);
    } finally {
      replayStatusLoaded = true;
    }
  }

  async function clearReplaySession() {
    await replayAdapter.clearSession();
    replayStatus = emptyReplayStatus();
    runMode = "mock";
  }

  async function resetTodayExperience() {
    await replayAdapter.resetExperience();
  }

  async function runTodayStage(stage: string): Promise<TodayRunResponse> {
    if (runMode === "mock") {
      return mockTodayStage(stage);
    }

    if (runMode === "replay") {
      const session = await replayRunner.ensureSession();
      const normalizedStage = replayAdapter.normalizeStage(stage);
      const recorded = (await replayRunner.takeRun(session, normalizedStage)) as unknown as TodayRecordedRun;
      await replayRunner.playDelay(recorded);

      const response: TodayRunResponse = {
        stage: recorded.stage,
        result: recorded.result,
        experience: recorded.experience,
      };
      if ((session as TodayReplaySession).mode === "recorded-live") {
        assertRealLlmCalls(response, "Recorded replay");
      }
      return response;
    }

    // Mock or Live mode
    const inputs = replayAdapter.formInputs(stage, runMode === "live", activeTruthDemoMode);
    const result = await replayAdapter.runStage(stage, inputs);

    if (runMode === "live") {
      assertRealLlmCalls(result as TodayRunResponse, "Live mode");
    }

    return result as TodayRunResponse;
  }

  function resetMockExperience() {
    mockExperienceRunCount = 0;
  }

  function mockTodayStage(stage: string): TodayRunResponse {
    const normalizedStage = replayAdapter.normalizeStage(stage);
    mockExperienceRunCount += 1;

    const creative = activeDemoMode === "creative";
    const selectedVendor = creative ? "Kong AI Gateway + Mistral EU" : "Mistral";
    const selectedAmountMajor = creative ? 52_000 : 22_000;
    const confidence = creative ? 0.88 : 0.91;
    const shortlist = mockShortlist(creative);
    const rejected = mockRejected(creative);
    const policy = mockPolicy(normalizedStage, selectedVendor, selectedAmountMajor);
    const learning = {
      prior_runs: Math.max(0, mockExperienceRunCount - 1),
      status:
        normalizedStage === "learning"
          ? "accepted pattern replayed without weakening gates"
          : "experience context available",
    };

    return {
      stage: normalizedStage,
      result: {
        converged: true,
        cycles: normalizedStage === "analysis" ? 3 : 2,
        stop_reason: normalizedStage === "negative-control" ? "policy_rejected" : "fixed_point",
        criteria_outcomes: [
          { criterion: "compliance_evidence", result: "satisfied" },
          { criterion: "risk_boundary", result: "satisfied" },
          { criterion: "cedar_authority", result: policy.outcome === "Reject" ? "rejected" : "satisfied" },
        ],
        projection: {
          events_emitted: normalizedStage === "analysis" ? 6 : 5,
          details: {
            formation: {
              assignments: formationAgents.map((agent) => ({
                agent: agent.name,
                source: agent.source,
              })),
            },
            recommendation: {
              recommendation: selectedVendor,
              confidence,
              rationale: creative
                ? "The governed mix keeps Kong at the policy perimeter and Mistral EU in the model slot."
                : "Mistral clears compliance, stays inside the risk boundary, and wins the cost trade-off.",
            },
            shortlist: {
              shortlist,
              rejected,
            },
            policy,
            learning,
          },
        },
        llm_calls: [],
      },
      experience: mockExperienceSnapshot(selectedVendor, confidence),
    };
  }

  function mockPolicy(stage: string, selectedVendor: string, selectedAmountMajor: number) {
    if (stage === "analysis") {
      return {
        outcome: "Escalate",
        selected_vendor: selectedVendor,
        selected_amount_major: selectedAmountMajor,
        hitl_threshold_major: 15_000,
        human_approval_present: false,
        principal_authority: "supervisory",
        reason: "Human approval is required before recommendation becomes a commitment.",
      };
    }

    if (stage === "negative-control") {
      return {
        outcome: "Reject",
        selected_vendor: selectedVendor,
        selected_amount_major: selectedAmountMajor,
        hitl_threshold_major: 15_000,
        human_approval_present: true,
        principal_authority: "advisory",
        reason: "Advisory authority cannot promote a vendor commitment.",
      };
    }

    return {
      outcome: "Promote",
      selected_vendor: selectedVendor,
      selected_amount_major: selectedAmountMajor,
      hitl_threshold_major: 15_000,
      human_approval_present: true,
      principal_authority: "supervisory",
      reason: "Human approval and supervisory authority are present.",
    };
  }

  function mockShortlist(creative: boolean): Array<Record<string, any>> {
    if (creative) {
      return [
        {
          rank: 1,
          vendor_name: "Kong AI Gateway + Mistral EU",
          score: 87,
          risk_score: 16,
          composite_score: "0.92",
        },
        {
          rank: 2,
          vendor_name: "OpenAI Enterprise",
          score: 94,
          risk_score: 18,
          composite_score: "0.89",
        },
        {
          rank: 3,
          vendor_name: "Google Vertex AI",
          score: 90,
          risk_score: 16,
          composite_score: "0.88",
        },
      ];
    }

    return [
      {
        rank: 1,
        vendor_name: "Mistral",
        score: 82,
        risk_score: 22,
        composite_score: "0.91",
      },
      {
        rank: 2,
        vendor_name: "Google DeepMind",
        score: 88,
        risk_score: 15,
        composite_score: "0.87",
      },
      {
        rank: 3,
        vendor_name: "OpenAI",
        score: 90,
        risk_score: 18,
        composite_score: "0.86",
      },
    ];
  }

  function mockRejected(creative: boolean): Array<Record<string, any>> {
    return [
      {
        vendor_name: creative ? "Qwen Alibaba Cloud" : "Qwen (Alibaba Cloud)",
        reasons: ["compliance evidence pending", "risk score exceeds governed threshold"],
      },
    ];
  }

  function mockExperienceSnapshot(selectedVendor: string, confidence: number): ExperienceSnapshot {
    const summaries = Array.from({ length: mockExperienceRunCount }, (_, index) => ({
      run_id: `mock-vendor-selection-${index + 1}`,
      cycles: index === 0 ? 3 : 2,
      elapsed_ms: 420 + index * 35,
      vendor_count: activeVendors.length,
      converged: true,
      confidence,
      recommended_vendor: selectedVendor,
      timestamp: new Date(2026, 3, 29, 12, index, 0).toISOString(),
    }));

    return {
      truth_key: "vendor-selection",
      run_count: mockExperienceRunCount,
      summaries,
      aggregate: {
        convergence_rate: 1,
        avg_cycles: summaries.length > 1 ? 2.2 : 3,
        avg_confidence: confidence,
        avg_elapsed_ms: 455,
        recommendation_frequencies: [
          {
            recommendation: selectedVendor,
            count: mockExperienceRunCount,
            share: 1,
          },
        ],
      },
    };
  }


  function emptyExperience(): ExperienceSnapshot {
    return {
      truth_key: "vendor-selection",
      run_count: 0,
      summaries: [],
      aggregate: {
        convergence_rate: 0,
        avg_cycles: 0,
        avg_confidence: 0,
        avg_elapsed_ms: 0,
        recommendation_frequencies: [],
      },
    };
  }

  function describeRunError(cause: unknown) {
    if (cause instanceof Error) return cause.message;
    if (typeof cause === "string") return cause;
    try {
      return JSON.stringify(cause);
    } catch {
      return `${activeDemoShortLabel} demo failed.`;
    }
  }

  function assertRealLlmCalls(response: TodayRunResponse, label: string) {
    const calls = response.result.llm_calls ?? [];
    if (calls.length === 0) {
      throw new Error(`${label} did not include any LLM calls. Check the recording or provider credentials.`);
    }

    const fallbacks = calls.filter((call) => {
      const metadata = call.metadata ?? {};
      return call.provider === "none" || call.model === "deterministic-fallback" || metadata.fallback === "true";
    });

    if (fallbacks.length > 0) {
      throw new Error(
        `${label} fell back instead of using real LLM calls: ${fallbacks
          .map((call) => call.context)
          .join(", ")}. Check provider credentials and model availability.`
      );
    }
  }

  function detailsFor(run: TodayRunResponse | null | undefined) {
    return run?.result.projection?.details ?? {};
  }

  function policyFor(run: TodayRunResponse | null | undefined) {
    return detailsFor(run).policy ?? {};
  }

  function recommendationFor(run: TodayRunResponse | null | undefined) {
    return detailsFor(run).recommendation ?? {};
  }

  function learningFor(run: TodayRunResponse | null | undefined) {
    return detailsFor(run).learning ?? {};
  }

  function shortlistFor(run: TodayRunResponse | null | undefined): Array<Record<string, any>> {
    const shortlist = detailsFor(run).shortlist?.shortlist;
    return Array.isArray(shortlist) ? shortlist : [];
  }

  function rejectedFor(run: TodayRunResponse | null | undefined): Array<Record<string, any>> {
    const rejected = detailsFor(run).shortlist?.rejected;
    return Array.isArray(rejected) ? rejected : [];
  }

  function topShortlistName(run: TodayRunResponse | null | undefined) {
    return String(shortlistFor(run)[0]?.vendor_name ?? "");
  }

  function stringAt(value: Record<string, any> | null | undefined, key: string) {
    const item = value?.[key];
    return typeof item === "string" ? item : "";
  }

  function numberAt(value: Record<string, any> | null | undefined, key: string) {
    const item = value?.[key];
    return typeof item === "number" ? item : null;
  }

  function money(value: unknown) {
    if (typeof value !== "number") return "-";
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(value);
  }

  function percent(value: unknown) {
    if (typeof value !== "number") return "-";
    return `${Math.round(value * 100)}%`;
  }

  function policyPillClass(outcome: unknown) {
    if (outcome === "Promote") return "pill-ok";
    if (outcome === "Reject") return "pill-err";
    return "pill-warn";
  }

  function totalLlmCalls() {
    const runs = [analysisRun, approvedRun, negativeControlRun, ...learningRuns].filter(Boolean) as TodayRunResponse[];
    return runs.reduce((total, run) => total + (run.result.llm_calls?.length ?? 0), 0);
  }

  function suggestorsForStep(index: number) {
    const byStep = [
      ["Bootstrap Mapper"],
      ["Formation Planner"],
      ["Compliance Agent", "Wide Evidence Agent", "Deep Evidence Agent"],
      ["Price Optimizer", "Risk Skeptic"],
      ["Consensus Promoter"],
      ["Cedar Gate"],
      ["Cedar Gate"],
      ["Cedar Gate", "Risk Skeptic"],
      ["Experience Store", "Consensus Promoter"],
      ["Experience Store", "Consensus Promoter"],
      ["Experience Store", "Consensus Promoter"],
      ["Consensus Promoter"],
    ];
    return byStep[index] ?? [];
  }

  function latestRunSummaries() {
    return [...(experience?.summaries ?? [])].reverse().slice(0, 5);
  }

  onMount(async () => {
    useSampleProcess();
    await loadReplayStatus();
    if (!autoStarted) {
      autoStarted = true;
      startEvaluation();
    }
  });
  onDestroy(() => {
    stopSpinner();
    clearPresentationTimers();
    flowPlayer.reset();
  });

  // For now, DocumentIntake fastLoadEnabled is not bindable
  // Keep using the local handlers (toggleFastLoad, handleFiles)
</script>

<section class="min-h-screen bg-void">
  <header class="flex items-center justify-between border-b border-border px-8 py-4">
    <div class="flex items-center gap-4">
      <button class="btn-ghost text-sm" type="button" onclick={onBack}>&larr; Slides</button>
      <span class="font-mono text-xs tracking-widest text-muted uppercase">{activeDemoLabel}</span>
    </div>
    <div class="flex items-center gap-3">
      <button class="btn-ghost text-sm" type="button" onclick={onSpecStudio}>Spec Studio</button>
      <button class="btn-ghost text-sm" type="button" onclick={onApps}>Apps</button>
    </div>
  </header>

  {#if runState === "bootstrap"}
    <div class="mx-auto grid max-w-7xl gap-6 px-8 py-8 lg:grid-cols-[0.85fr_1.15fr]">
      <section class="rounded-[28px] border border-border bg-panel p-5">
        <p class="slide-eyebrow mb-3">Vendor Selection Demo</p>
        <h1 class="slide-headline mb-4 text-4xl!">{activeDemoLabel}</h1>

        <div class="grid gap-3">
          <button
            type="button"
            class="rounded-2xl border p-4 text-left transition"
            class:border-lime={evaluationMode === "today"}
            class:bg-lime-glow={evaluationMode === "today"}
            class:border-border={evaluationMode !== "today"}
            class:bg-raised={evaluationMode !== "today"}
            onclick={() => (evaluationMode = "today")}
          >
            <span class="block font-display text-xl font-semibold text-bright">Today</span>
            <span class="mt-1 block text-sm text-subtle">Governed selection inside the current RFI/RFP frame.</span>
          </button>
          <button
            type="button"
            class="rounded-2xl border p-4 text-left transition"
            class:border-lime={evaluationMode === "creative"}
            class:bg-lime-glow={evaluationMode === "creative"}
            class:border-border={evaluationMode !== "creative"}
            class:bg-raised={evaluationMode !== "creative"}
            onclick={() => (evaluationMode = "creative")}
          >
            <span class="block font-display text-xl font-semibold text-bright">Creative</span>
            <span class="mt-1 block text-sm text-subtle">Pareto breakout that may prefer a router or provider mix.</span>
          </button>
        </div>

        <div class="mt-5 rounded-2xl border border-border bg-raised p-4">
          <span class="card-label">Run Mode</span>
          <div class="mt-3 rounded-xl border border-lime/30 bg-lime-glow px-3 py-3">
            <span class="block text-sm font-semibold text-bright">Mock presentation flow</span>
            <span class="block text-xs text-subtle">No providers, no replay, no backend calls during the walkthrough.</span>
          </div>
        </div>

        <button class="btn-lime mt-5 w-full justify-center py-3" type="button" onclick={startEvaluation}>
          Run {activeDemoShortLabel}
        </button>

        {#if flowError}
          <div class="callout callout-error mt-4">
            <strong>Run failed</strong>
            <p>{flowError}</p>
          </div>
        {/if}
      </section>

      <aside class="space-y-5">
        <section class="rounded-[28px] border border-border bg-panel p-5">
          <div class="flex items-start justify-between gap-4">
            <div>
              <span class="card-label">Source Pack</span>
              <h2 class="mt-1 font-display text-2xl font-semibold text-bright">Fast track package loaded</h2>
            </div>
            <span class="pill pill-ok">{documents.length || sampleDocs.length} docs</span>
          </div>
          <div class="mt-4 grid gap-2 md:grid-cols-2">
            {#each (documents.length > 0 ? documents : sampleDocs) as document}
              <a class="rounded-xl border border-border bg-raised px-3 py-2 transition hover:border-lime/50" href={document.href} target="_blank" rel="noreferrer">
                <span class="block truncate text-sm text-bright">{document.name}</span>
                <span class="block text-xs text-muted">{document.kind} / {document.size}</span>
              </a>
            {/each}
          </div>
        </section>

        <section class="rounded-[28px] border border-border bg-panel p-5">
          <div class="flex items-center justify-between gap-4">
            <span class="card-label">{activeDemoShortLabel} Candidates</span>
            <span class="pill pill-info">{activeVendors.length} candidates</span>
          </div>
          <div class="mt-3 grid gap-2 md:grid-cols-2">
            {#each activeVendors as vendor}
              <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-raised px-3 py-2">
                <div class="min-w-0">
                  <p class="truncate text-sm text-bright">{vendor.name}</p>
                  <p class="text-xs text-muted">score {vendor.score} / risk {vendor.risk_score}</p>
                </div>
                <span
                  class="pill"
                  class:pill-ok={vendor.compliance_status === "compliant"}
                  class:pill-warn={vendor.compliance_status !== "compliant"}
                >
                  {vendor.compliance_status}
                </span>
              </div>
            {/each}
          </div>
        </section>
      </aside>
    </div>
  {:else}
    <div class="mx-auto grid max-w-7xl gap-6 px-8 py-8 lg:grid-cols-[0.9fr_1.1fr]">
      <section class="rounded-[28px] border border-border bg-panel p-5">
        <div class="mb-5 flex items-start justify-between gap-4">
          <div>
            <p class="slide-eyebrow mb-2">
              {runState === "finished"
                ? "Fixed Point"
                : runState === "commit-review"
                  ? "Commitment Review"
                  : runState === "gate-review" || runState === "hitl"
                    ? "Human Gate"
                    : "Formation In Work"}
            </p>
            <h1 class="slide-headline text-3xl!">{activeDemoLabel}</h1>
            <p class="mt-2 text-sm text-subtle">{bootstrapLabel}</p>
          </div>
          {#if runState === "finished"}
            <span class="pill pill-ok">Converged</span>
          {:else if runState === "commit-review"}
            <span class="pill pill-ok">Authorized</span>
          {:else if runState === "gate-review"}
            <span class="pill pill-warn">Review Gate</span>
          {:else if runState === "hitl"}
            <span class="pill pill-warn">Approval Needed</span>
          {:else}
            <span class="pill pill-info">{activeAgent || "Starting"}</span>
          {/if}
        </div>

        <div class="mb-6 grid gap-2">
          {#key runAnimationKey}
            {#if runState === "running"}
              {#each runningSegmentSteps as step, localIndex}
                {@const globalIndex = runningSegmentStart + localIndex}
                <div
                  class="mock-step-row flex items-center gap-3 rounded-xl border px-3 py-2"
                  style={`--step-index: ${localIndex}; --step-duration: ${PRESENTATION_STEP_DELAY_MS}ms;`}
                >
                  <span class="mock-step-badge flex h-7 w-7 shrink-0 items-center justify-center rounded-full font-mono text-[0.7rem]">
                    {globalIndex + 1}
                  </span>
                  <span class="min-w-0 truncate text-sm font-semibold">
                    {step.step}
                  </span>
                  <span class="mock-step-pulse ml-auto h-2 w-2 shrink-0 rounded-full"></span>
                </div>
              {/each}
            {:else}
              {#each pipelineSteps.slice(0, presentationStepIndex + 1) as step, index}
                <div
                  class="flex items-center gap-3 rounded-xl border px-3 py-2 transition-all"
                  class:border-warn={index === presentationStepIndex}
                  class:bg-warn={index === presentationStepIndex}
                  class:bg-opacity-10={index === presentationStepIndex}
                  class:border-border={index < presentationStepIndex}
                  class:bg-deep={index < presentationStepIndex}
                  class:opacity-50={index < presentationStepIndex}
                >
                  <span
                    class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full font-mono text-[0.7rem]"
                    class:bg-warn={index === presentationStepIndex}
                    class:text-deep={index === presentationStepIndex}
                    class:bg-raised={index < presentationStepIndex}
                    class:text-muted={index !== presentationStepIndex}
                  >
                    {index + 1}
                  </span>
                  <span
                    class="min-w-0 truncate text-sm font-semibold"
                    class:text-bright={index === presentationStepIndex}
                    class:text-subtle={index !== presentationStepIndex}
                  >
                    {step.step}
                  </span>
                </div>
              {/each}
            {/if}
          {/key}
        </div>

        {#if runState === "running"}
          <div class="h-1.5 overflow-hidden rounded-full bg-border">
            {#key runAnimationKey}
              <div
                class="mock-progress-fill h-full rounded-full bg-warn"
                style={`--segment-duration: ${(runningSegmentEnd - runningSegmentStart + 1) * PRESENTATION_STEP_DELAY_MS}ms;`}
                onanimationend={completeMockSegment}
              ></div>
            {/key}
          </div>
          <p class="mt-4 flex items-center justify-center gap-2 text-center text-sm text-muted">
            <span class="inline-block h-2.5 w-2.5 animate-pulse rounded-full bg-warn"></span>
            <span>
              {spinnerVerb}... {runMode === "mock" ? "mocked governed agents" : runMode === "replay" ? "recorded live LLM thinking" : "live provider path"}
            </span>
          </p>
          <button class="btn-ghost mt-3 w-full justify-center text-sm" type="button" onclick={completeMockSegment}>
            Continue
          </button>
        {/if}

        {#if flowError && runState !== "bootstrap"}
          <div class="callout callout-error mt-5">
            <strong>Run failed</strong>
            <p>{flowError}</p>
          </div>
        {/if}

        {#if runState === "gate-review"}
          <section class="mt-5 rounded-2xl border border-warn/30 bg-warn/5 p-4">
            <span class="card-label text-warn!">Pause Before HITL</span>
            <h2 class="mt-1 font-display text-xl font-semibold text-bright">The recommendation is not a commitment yet.</h2>
            <p class="mt-2 text-sm text-subtle">
              The {runMode === "mock" ? "mocked agents" : runMode === "replay" ? "recorded live agents" : "live agents"} selected a candidate and Cedar escalated because approval is missing. Review the evidence before opening the human decision form.
            </p>

            <div class="mt-4 grid gap-3 md:grid-cols-3">
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Candidate</span>
                <p class="mt-1 text-sm text-bright">{stringAt(beforeHitlPolicy, "selected_vendor")}</p>
                <p class="mt-1 text-xs text-muted">{money(numberAt(beforeHitlPolicy, "selected_amount_major"))} / mo</p>
              </div>
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Cedar Says</span>
                <p class="mt-1 text-sm text-bright">{stringAt(beforeHitlPolicy, "outcome")}</p>
                <p class="mt-1 text-xs text-muted">{stringAt(beforeHitlPolicy, "reason") || "Human approval is missing."}</p>
              </div>
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">{runMode === "mock" ? "Mode" : "LLM Calls"}</span>
                <p class="mt-1 text-sm text-bright">{runMode === "mock" ? "Mocked" : analysisRun?.result.llm_calls?.length ?? 0}</p>
                <p class="mt-1 text-xs text-muted">{runMode === "mock" ? "deterministic stage output" : "provider calls recorded for this stage"}</p>
              </div>
            </div>

            {#if rejectedRows.length > 0}
              <div class="mt-4 rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Blocked Inputs</span>
                <div class="mt-2 space-y-2">
                  {#each rejectedRows as vendor}
                    <p class="text-xs text-subtle">
                      <span class="text-bright">{String(vendor.vendor_name)}</span>: {Array.isArray(vendor.reasons) ? vendor.reasons.join("; ") : ""}
                    </p>
                  {/each}
                </div>
              </div>
            {/if}

            <button class="btn-lime mt-4 w-full justify-center" type="button" onclick={openHitlDecision}>
              Open HITL Decision
            </button>
          </section>
        {/if}

        {#if runState === "hitl"}
          <HitlGate
            decisionSummary={{
              candidate: stringAt(recommendationFor(analysisRun), "recommendation"),
              reason: stringAt(beforeHitlPolicy, "reason") || "Human approval is missing.",
              threshold: money(numberAt(beforeHitlPolicy, "hitl_threshold_major")),
            }}
            bind:approverName={approver}
            bind:approvalNote
            bind:delegateToPolicy={delegateToCedar}
            policyPreview={delegateToCedar ? cedarPreview : ""}
            onApprove={approveHitl}
          />
        {/if}

        {#if runState === "commit-review"}
          <section class="mt-5 rounded-2xl border border-lime/30 bg-lime-glow p-4">
            <span class="card-label text-lime!">Pause After HITL</span>
            <h2 class="mt-1 font-display text-xl font-semibold text-bright">Human approval changed the policy outcome.</h2>
            <p class="mt-2 text-sm text-subtle">
              Cedar can now promote the commitment. The next step intentionally reruns controls to prove authority still matters, then grows the experience store.
            </p>

            <div class="mt-4 grid gap-3 md:grid-cols-3">
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Cedar Outcome</span>
                <p class="mt-1 text-sm text-bright">{stringAt(finalPolicy, "outcome")}</p>
                <p class="mt-1 text-xs text-muted">approval {finalPolicy.human_approval_present ? "present" : "missing"}</p>
              </div>
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Commitment</span>
                <p class="mt-1 text-sm text-bright">{stringAt(finalPolicy, "selected_vendor")}</p>
                <p class="mt-1 text-xs text-muted">{money(numberAt(finalPolicy, "selected_amount_major"))} / mo</p>
              </div>
              <div class="rounded-xl border border-border bg-deep p-3">
                <span class="card-label">Next Check</span>
                <p class="mt-1 text-sm text-bright">Advisory authority</p>
                <p class="mt-1 text-xs text-muted">should reject the same candidate</p>
              </div>
            </div>

            <button class="btn-lime mt-4 w-full justify-center" type="button" onclick={continueAfterCommitReview}>
              Run Negative Control And Learning
            </button>
          </section>
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
            <span class="card-label">Suggestors</span>
            <h2 class="mt-1 font-display text-2xl font-semibold text-bright">Who is active now?</h2>
            {#if runState === "running"}
              <p class="mt-2 text-sm text-subtle">Suggestor groups light up as the mocked flow progresses.</p>
            {:else if parallelSuggestorWork}
              <p class="mt-2 text-sm text-subtle">Parallel suggestors are running for this step.</p>
            {:else}
              <p class="mt-2 text-sm text-subtle">One suggestor is carrying the current step.</p>
            {/if}
            {#if runState === "running"}
              {#key runAnimationKey}
                <div class="mt-4 grid gap-3">
                  {#each runningSegmentSteps as step, localIndex}
                    {@const globalIndex = runningSegmentStart + localIndex}
                    <article
                      class="mock-suggestor-row rounded-2xl border p-3"
                      style={`--step-index: ${localIndex}; --step-duration: ${PRESENTATION_STEP_DELAY_MS}ms;`}
                    >
                      <div class="flex items-center justify-between gap-3">
                        <h3 class="font-display text-sm font-semibold text-bright">{step.step}</h3>
                        <span class="pill pill-info">{suggestorsForStep(globalIndex).length > 1 ? "Parallel" : "Focused"}</span>
                      </div>
                      <div class="mt-3 flex flex-wrap gap-2">
                        {#each suggestorsForStep(globalIndex) as suggestor}
                          <span class="rounded-full border border-warn/30 bg-warn/5 px-3 py-1 font-mono text-[0.68rem] text-warn">
                            {suggestor}
                          </span>
                        {/each}
                      </div>
                    </article>
                  {/each}
                </div>
              {/key}
            {:else}
              <div class="mt-4 grid gap-3 md:grid-cols-2">
                {#each formationAgents as agent}
                  <article
                    class="rounded-2xl border p-3 transition-all"
                    class:border-lime={activeSuggestorSet.has(agent.name)}
                    class:bg-lime-glow={activeSuggestorSet.has(agent.name)}
                    class:border-border={!activeSuggestorSet.has(agent.name)}
                    class:bg-raised={!activeSuggestorSet.has(agent.name)}
                    class:ring-1={activeSuggestorSet.has(agent.name)}
                    class:ring-lime={activeSuggestorSet.has(agent.name)}
                  >
                    <div class="flex items-start justify-between gap-3">
                      <h3 class="font-display text-sm font-semibold text-bright">{agent.name}</h3>
                      <span class="pill" class:pill-ok={activeSuggestorSet.has(agent.name)} class:pill-info={!activeSuggestorSet.has(agent.name)}>
                        {activeSuggestorSet.has(agent.name) ? "Active" : agent.kind}
                      </span>
                    </div>
                    <p class="mt-2 text-xs text-subtle">{agent.purpose}</p>
                    <p class="mt-2 font-mono text-[0.68rem] text-muted">{agent.source}</p>
                  </article>
                {/each}
              </div>
            {/if}
          </section>

          {#if analysisRun}
            <section class="rounded-[28px] border border-border bg-panel p-5">
              <div class="flex items-start justify-between gap-3">
                <div>
                  <span class="card-label">Observed Run</span>
                  <h2 class="mt-1 font-display text-2xl font-semibold text-bright">Decision package before approval</h2>
                </div>
                <span class="pill {policyPillClass(stringAt(beforeHitlPolicy, "outcome"))}">{stringAt(beforeHitlPolicy, "outcome")}</span>
              </div>

              <div class="mt-4 grid gap-3 md:grid-cols-2">
                <div class="rounded-2xl border border-border bg-raised p-4">
                  <span class="card-label">Selected Vendor</span>
                  <p class="mt-2 font-display text-2xl text-bright">{stringAt(beforeHitlPolicy, "selected_vendor")}</p>
                  <p class="mt-1 text-xs text-muted">{analysisRun.result.cycles} cycles, {analysisRun.result.converged ? "converged" : "stopped"}</p>
                </div>
                <div class="rounded-2xl border border-border bg-raised p-4">
                  <span class="card-label">Experience</span>
                  <p class="mt-2 font-display text-2xl text-bright">{analysisRun.experience.run_count}</p>
                  <p class="mt-1 text-xs text-muted">run summaries in this desktop session</p>
                </div>
              </div>

              {#if rejectedRows.length > 0}
                <div class="mt-4 rounded-2xl border border-border bg-raised p-4">
                  <span class="card-label">Rejected Or Blocked</span>
                  <div class="mt-3 space-y-2">
                    {#each rejectedRows as vendor}
                      <div class="rounded-xl border border-border bg-deep px-3 py-2">
                        <p class="text-sm text-bright">{String(vendor.vendor_name)}</p>
                        <p class="mt-1 text-xs text-muted">{Array.isArray(vendor.reasons) ? vendor.reasons.join("; ") : ""}</p>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </section>
          {/if}
        {:else}
          <section class="rounded-[28px] border border-lime/30 bg-lime-glow p-5">
            <span class="card-label text-lime!">Result</span>
            <h2 class="mt-1 font-display text-3xl font-semibold text-bright">{selectedVendorName || "Recommendation"} promoted through governance.</h2>
            <p class="mt-3 text-sm text-subtle">
              The desktop {runModeVerb} the {activeDemoShortLabel} path: first it stopped at HITL, then it promoted the same recommendation after approval, rejected the advisory-authority negative control, and grew the learning context.
            </p>

            <div class="mt-5 grid gap-3 md:grid-cols-2">
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Cedar Outcome</span>
                <p class="mt-2 text-sm text-bright">{stringAt(finalPolicy, "outcome")}</p>
                <p class="mt-1 text-xs text-subtle">{stringAt(finalPolicy, "principal_authority")} authority, approval {finalPolicy.human_approval_present ? "present" : "missing"}.</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Commitment</span>
                <p class="mt-2 text-sm text-bright">{money(numberAt(finalPolicy, "selected_amount_major"))} / mo</p>
                <p class="mt-1 text-xs text-subtle">HITL threshold {money(numberAt(finalPolicy, "hitl_threshold_major"))}.</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Negative Control</span>
                <p class="mt-2 text-sm text-bright">{stringAt(policyFor(negativeControlRun), "outcome") || "not run"}</p>
                <p class="mt-1 text-xs text-subtle">{stringAt(policyFor(negativeControlRun), "reason") || "Advisory authority should not commit."}</p>
              </div>
              <div class="rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Learning Context</span>
                <p class="mt-2 text-sm text-bright">{String(learningStatus.prior_runs ?? experience?.run_count ?? 0)} prior runs</p>
                <p class="mt-1 text-xs text-subtle">{String(learningStatus.status ?? "updated")}</p>
              </div>
            </div>

            <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
              <span class="card-label">Shortlist</span>
              <div class="mt-3 space-y-2">
                {#each shortlistRows as vendor}
                  <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-raised px-3 py-2">
                    <div class="min-w-0">
                      <p class="truncate text-sm text-bright">#{String(vendor.rank)} {String(vendor.vendor_name)}</p>
                      <p class="text-xs text-muted">capability {String(vendor.score)} / risk {String(vendor.risk_score)}</p>
                    </div>
                    <span class="font-mono text-xs text-lime">{String(vendor.composite_score)}</span>
                  </div>
                {/each}
              </div>
            </div>

            <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
              <span class="card-label">Governance Record</span>
              <div class="mt-3 grid gap-2">
                <p class="text-sm text-text">Formation selected {finalDetails.formation?.assignments?.length ?? 0} role assignments.</p>
                <p class="text-sm text-text">HITL approval captured from {approver}.</p>
                <p class="text-sm text-text">{installedCedar ? "Cedar delegation candidate recorded for the next matching run." : "Human approval remains required for the next run."}</p>
                <p class="text-sm text-text">Fixed point reached after {approvedRun?.result.cycles ?? "-"} cycles.</p>
                <p class="text-sm text-text">LLM telemetry entries: {liveCallCount}</p>
              </div>
            </div>

            {#if latestRunSummaries().length > 0}
              <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
                <span class="card-label">Experience Store</span>
                <div class="mt-3 space-y-2">
                  {#each latestRunSummaries() as run}
                    <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-raised px-3 py-2">
                      <span class="truncate text-sm text-bright">{run.recommended_vendor}</span>
                      <span class="font-mono text-xs text-muted">{run.cycles} cycles</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
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

<style>
  .mock-step-row,
  .mock-suggestor-row {
    opacity: 0;
    max-height: 0;
    overflow: hidden;
    transform: translateY(-0.35rem);
    border-color: var(--color-border);
    background: var(--color-deep);
    color: var(--color-subtle);
    animation:
      mock-row-reveal 180ms ease forwards calc(var(--step-index) * var(--step-duration)),
      mock-row-active var(--step-duration) linear forwards calc(var(--step-index) * var(--step-duration));
  }

  .mock-step-badge {
    background: var(--color-raised);
    color: var(--color-muted);
    animation: mock-badge-active var(--step-duration) linear forwards calc(var(--step-index) * var(--step-duration));
  }

  .mock-step-pulse {
    background: var(--color-warn);
    opacity: 0;
    animation: mock-pulse-active var(--step-duration) linear forwards calc(var(--step-index) * var(--step-duration));
  }

  .mock-progress-fill {
    width: 8%;
    animation: mock-progress var(--segment-duration) linear forwards;
  }

  @keyframes mock-row-reveal {
    to {
      opacity: 1;
      max-height: 4rem;
      transform: translateY(0);
    }
  }

  @keyframes mock-row-active {
    0%,
    88% {
      border-color: var(--color-warn);
      background: rgba(251, 191, 36, 0.12);
      color: var(--color-bright);
      opacity: 1;
    }
    100% {
      border-color: var(--color-border);
      background: var(--color-deep);
      color: var(--color-subtle);
      opacity: 0.5;
    }
  }

  @keyframes mock-badge-active {
    0%,
    88% {
      background: var(--color-warn);
      color: var(--color-deep);
    }
    100% {
      background: var(--color-raised);
      color: var(--color-muted);
    }
  }

  @keyframes mock-pulse-active {
    0%,
    88% {
      opacity: 1;
      transform: scale(1);
    }
    44% {
      opacity: 0.35;
      transform: scale(0.7);
    }
    100% {
      opacity: 0;
      transform: scale(0.7);
    }
  }

  @keyframes mock-progress {
    to {
      width: 100%;
    }
  }
</style>
