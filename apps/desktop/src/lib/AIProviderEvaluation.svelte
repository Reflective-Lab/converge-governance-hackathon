<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { FlowPlayer, ReplayRunner } from "@reflective/helm-flow";
  import HitlGate from "@reflective/helm-flow/src/HitlGate.svelte";
  import DocumentIntake from "@reflective/helm-flow/src/DocumentIntake.svelte";
  import type { RunState as HelmRunState, FlowPhase, FlowStep } from "@reflective/helm-flow";
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
    TruthResult,
    RunSummary,
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

  // Inline todayVendors for adapter initialization (moved later in component for full definition)
  const todayVendorsForAdapter: Array<Record<string, any>> = [];

  // Replay adapter and runner (initialized before state, but vendor data comes later)
  const replayAdapter = new VendorSelectionReplayAdapter(todayVendorsForAdapter);
  const replayRunner = new ReplayRunner(replayAdapter as any);

  // Flow orchestration (Helm-owned)
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
    stepDelayMs: 1650,
    reviewPauseMs: 1100,
  });

  let flowState = $state(flowPlayer.getState());
  let spinnerVerb = $state(randomVerb());

  // Domain state
  let bootstrapMode = $state<BootstrapMode>("upload");
  let evaluationMode = $state<EvaluationMode>("today");
  let documents = $state<EvaluationDoc[]>([]);
  let executableReady = $state(false);
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

  // Convenience aliases
  let runState = $derived(flowState.runState as RunState);
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

  // Populate adapter with vendor data
  todayVendorsForAdapter.push(...todayVendors);

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

  let documentPackageReady = $derived(documents.length >= 3);
  let canStart = $derived(documentPackageReady && executableReady);
  let canRunToday = $derived(
    canStart && (runMode === "mock" || runMode === "live" || replayStatus?.available === true),
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
  // Build steps array from pipeline steps and FlowPlayer state
  let steps = $derived.by(() => {
    return pipelineSteps.map((step, index) => ({
      step: step.step,
      detail: step.detail,
      agent: step.agent,
      purpose: step.purpose,
      active: index === flowState.activeStepIndex,
    }));
  });

  let progressPercent = $derived(flowState.progressPercent);
  let activeAgent = $derived(steps.find((step) => step.active)?.agent ?? "");
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
      ? "Runs the governed Today path locally with deterministic agents and presenter-paced timing."
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

  async function startEvaluation() {
    if (!canRunToday || evaluationMode !== "today") return;

    // Initialize flow
    flowPlayer.start();
    updateFlowState();
    installedCedar = false;
    flowError = "";
    replayCursor = {};
    analysisRun = null;
    approvedRun = null;
    negativeControlRun = null;
    learningRuns = [];
    experience = null;
    startSpinner();

    try {
      // Schedule first 6 steps (Bootstrapping → Compliance Screen)
      flowPlayer.scheduleSteps(6, (index) => {
        updateFlowState();
      });

      if (runMode === "mock") {
        await resetTodayExperience();
      } else if (runMode === "replay") {
        await ensureReplaySession();
      } else {
        await resetTodayExperience();
      }

      // Run analysis while steps progress
      const [response] = await Promise.all([
        runTodayStage("analysis"),
        wait(9_200),
      ]);
      analysisRun = response;
      experience = analysisRun.experience;
      await pauseForReview(1_800);

      // Transition to gate review
      flowPlayer.pauseAtGate("hitl", () => {
        updateFlowState();
      });
    } catch (cause) {
      flowError = describeRunError(cause);
      flowPlayer.reset();
      updateFlowState();
    } finally {
      stopSpinner();
    }
  }

  function openHitlDecision() {
    flowState = { ...flowState, runState: "hitl" as RunState };
  }

  async function approveHitl() {
    installedCedar = delegateToCedar;
    flowPlayer.approveGate();
    updateFlowState();
    flowError = "";
    startSpinner();

    try {
      // Step 6: Promote Commitment
      flowPlayer.scheduleSteps(1, () => updateFlowState());
      const [approved] = await Promise.all([
        runTodayStage("approved"),
        wait(2_400),
      ]);
      approvedRun = approved;
      experience = approvedRun.experience;

      // Step 7: Negative Control
      flowPlayer.scheduleSteps(1, () => updateFlowState());
      const [negative] = await Promise.all([
        runTodayStage("negative-control"),
        wait(2_400),
      ]);
      negativeControlRun = negative;
      experience = negativeControlRun.experience;

      // Steps 8-10: Learning Runs
      for (let index = 0; index < 3; index += 1) {
        flowPlayer.scheduleSteps(1, () => updateFlowState());
        const [learningRun] = await Promise.all([
          runTodayStage("learning"),
          wait(1_650),
        ]);
        learningRuns = [...learningRuns, learningRun];
        experience = learningRun.experience;
      }

      // Step 11: Fixed Point
      await pauseForReview(1_300);
      flowPlayer.finish();
      updateFlowState();
    } catch (cause) {
      flowError = describeRunError(cause);
      flowState = { ...flowState, runState: "hitl" as RunState };
    } finally {
      stopSpinner();
    }
  }

  async function continueAfterCommitReview() {
    flowState = { ...flowState, runState: "running" };
    flowError = "";
    startSpinner();

    try {
      negativeControlRun = await runTodayStage("negative-control");
      experience = negativeControlRun.experience;

      for (let index = 0; index < 3; index += 1) {
        const learningRun = await runTodayStage("learning");
        learningRuns = [...learningRuns, learningRun];
        experience = learningRun.experience;
      }

      flowPlayer.finish();
      updateFlowState();
    } catch (cause) {
      flowError = describeRunError(cause);
      flowState = { ...flowState, runState: "commit-review" };
    } finally {
      stopSpinner();
    }
  }

  function resetEvaluation() {
    flowPlayer.reset();
    updateFlowState();
    replayRunner.resetCursor();
    installedCedar = false;
    flowError = "";
    analysisRun = null;
    approvedRun = null;
    negativeControlRun = null;
    learningRuns = [];
    approvalNote = "Evidence package is sufficient for the demo threshold.";
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
    const inputs = replayAdapter.formInputs(stage, runMode === "live");
    const result = await replayAdapter.runStage(stage, inputs);

    if (runMode === "live") {
      assertRealLlmCalls(result as TodayRunResponse, "Live mode");
    }

    return result as TodayRunResponse;
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
      return "Today demo failed.";
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

  function wait(ms: number) {
    return new Promise<void>((resolve) => {
      setTimeout(resolve, ms);
    });
  }

  function pauseForReview(ms = 1_100) {
    return wait(ms);
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

  function latestRunSummaries() {
    return [...(experience?.summaries ?? [])].reverse().slice(0, 5);
  }

  onMount(loadReplayStatus);
  onDestroy(() => {
    stopSpinner();
    flowPlayer.reset();
  });

  // For now, DocumentIntake fastLoadEnabled is not bindable
  // Keep using the local handlers (toggleFastLoad, handleFiles)
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
    <div class="mx-auto max-w-5xl px-8 py-10">
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
            <p class="text-sm text-subtle">Governed selection inside the current RFI/RFP frame, using the same source pack and policy path as <code class="font-mono text-lime">just demo-today-live</code>.</p>
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
          <div class="grid gap-3">
            <DocumentIntake
              bind:documents
              fastLoadEnabled={bootstrapMode === "sample"}
              bind:executableReady
              expectedDocs={expectedDocs}
              onFilesSelected={(files) => handleFiles(files)}
            />

            <!-- Vendor-selection-specific: Fast-load toggle -->
            <button
              class="rounded-xl border px-3 py-2 text-left transition"
              class:border-lime={bootstrapMode === "sample"}
              class:bg-lime-glow={bootstrapMode === "sample"}
              class:border-border={bootstrapMode !== "sample"}
              class:bg-deep={bootstrapMode !== "sample"}
              type="button"
              onclick={() => toggleFastLoad(bootstrapMode === "upload")}
            >
              <span class="block text-sm font-semibold text-bright">
                {bootstrapMode === "sample" ? "Using Sample Package" : "Load Sample Package"}
              </span>
              <span class="block text-xs text-muted">AI provider evaluation demo data</span>
            </button>
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

        {#if bootstrapMode === "sample"}
          <div class="mt-5 rounded-2xl border border-border bg-deep p-4">
            <div class="mb-3 flex items-center justify-between gap-4">
              <span class="card-label">As Of Today</span>
              <span class="pill pill-info">Mocked flow</span>
            </div>
            <div class="grid gap-2 md:grid-cols-2">
              {#each todayVendors as vendor}
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
          </div>
        {/if}

        <button class="btn-lime mt-5 w-full justify-center py-3" type="button" disabled={!canRunToday || evaluationMode !== "today"} onclick={startEvaluation}>
          Run
        </button>

        {#if flowError}
          <div class="callout callout-error mt-4">
            <strong>Run failed</strong>
            <p>{flowError}</p>
          </div>
        {/if}
      </section>

      <section class="hidden rounded-[28px] border border-border bg-panel p-5">
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
          <div class="flex items-start justify-between gap-4">
            <div>
              <span class="card-label">Demo Run Mode</span>
              <p class="mt-1 font-display text-xl font-semibold text-bright">{runModeLabel}</p>
              <p class="mt-1 text-sm text-subtle">{runModeDetail}</p>
            </div>
            <span
              class="pill"
              class:pill-ok={runMode === "replay" && replayStatus?.available}
              class:pill-info={runMode === "live"}
              class:pill-warn={runMode === "replay" && !replayStatus?.available}
            >
              {runMode === "replay" ? "Replay" : "Live"}
            </span>
          </div>

          <div class="mt-4 grid gap-2 md:grid-cols-2">
            <button
              type="button"
              class="rounded-xl border px-3 py-2 text-left transition"
              class:border-lime={runMode === "replay"}
              class:bg-lime-glow={runMode === "replay"}
              class:border-border={runMode !== "replay"}
              class:bg-deep={runMode !== "replay"}
              class:opacity-50={!replayStatus?.available}
              disabled={!replayStatus?.available}
              onclick={() => (runMode = "replay")}
            >
              <span class="block text-sm font-semibold text-bright">Use replay</span>
              <span class="block text-xs text-muted">{replayStatus?.available ? `${replayStatus.run_count} recorded stages` : "No recording found"}</span>
            </button>
            <button
              type="button"
              class="rounded-xl border px-3 py-2 text-left transition"
              class:border-lime={runMode === "live"}
              class:bg-lime-glow={runMode === "live"}
              class:border-border={runMode !== "live"}
              class:bg-deep={runMode !== "live"}
              onclick={() => (runMode = "live")}
            >
              <span class="block text-sm font-semibold text-bright">Run live</span>
              <span class="block text-xs text-muted">Uses provider credentials now</span>
            </button>
          </div>

          <div class="mt-4 rounded-xl border border-border bg-deep p-3">
            {#if !replayStatusLoaded}
              <p class="text-sm text-subtle">Checking for a recorded live session...</p>
            {:else if replayStatus?.available}
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm text-bright">
                    {replayStatus.mode === "recorded-live" ? "Recording ready" : "Offline backup ready"}
                  </p>
                  <p class="mt-1 font-mono text-xs text-muted">{replayStatus.recorded_at} / {replayStatus.source_hash}</p>
                </div>
                <span class="pill" class:pill-ok={replayStatus.source_matches} class:pill-warn={!replayStatus.source_matches}>
                  {replayStatus.source_matches ? "Source match" : "Source changed"}
                </span>
              </div>
              {#if replayStatus.model_summary.length > 0}
                <div class="mt-3 space-y-1">
                  {#each replayStatus.model_summary.slice(0, 4) as model}
                    <p class="truncate font-mono text-[0.68rem] text-muted">{model}</p>
                  {/each}
                </div>
              {/if}
            {:else}
              <p class="text-sm text-subtle">No recording is available yet. Record once with credentials, then present from replay mode.</p>
              {#if replayStatus?.error}
                <p class="mt-1 text-xs text-warn">{replayStatus.error}</p>
              {/if}
            {/if}
          </div>

          <div class="mt-4 flex flex-wrap gap-2">
            <button class="btn-ghost text-sm" type="button" disabled={recordingReplay} onclick={recordReplaySession}>
              {recordingReplay ? "Recording..." : "Record New Live Session"}
            </button>
            <button class="btn-ghost text-sm" type="button" disabled={buildingOfflineReplay || recordingReplay} onclick={buildOfflineReplaySession}>
              {buildingOfflineReplay ? "Building..." : "Build Offline Backup"}
            </button>
            {#if replayStatus?.available}
              <button class="btn-ghost text-sm" type="button" disabled={recordingReplay} onclick={clearReplaySession}>
                Clear Recording
              </button>
            {/if}
          </div>
        </div>

        <div class="mt-4 rounded-2xl border border-border bg-raised p-5">
          <span class="card-label">Today Candidates</span>
          <div class="mt-3 space-y-2">
            {#each todayVendors as vendor}
              <div class="flex items-center justify-between gap-3 rounded-xl border border-border bg-deep px-3 py-2">
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
        </div>
      </section>
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
            <h1 class="slide-headline text-3xl!">AI Provider Evaluation</h1>
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
          <p class="mt-4 text-center text-sm text-muted">
            {spinnerVerb}... {runMode === "mock" ? "mocked governed agents" : runMode === "replay" ? "recorded live LLM thinking" : "live provider path"}
          </p>
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
              The desktop {runModeVerb} the governed Today path: first it stopped at HITL, then it promoted the same recommendation after approval, rejected the advisory-authority negative control, and grew the learning context.
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
