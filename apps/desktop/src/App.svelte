<script>
  import { onDestroy } from "svelte";
  import { invokeTauri } from "./lib/tauri.js";

  // ─── Phases ───
  let phase = $state("slides"); // slides | demo
  let currentSlide = $state(0);

  const slides = [
    {
      eyebrow: "The Problem",
      headline: "Vendor decisions\nare a black box.",
      body: "Enterprises evaluate AI vendors with spreadsheets, email chains, and gut feel. No audit trail, no reproducibility, no governance.",
      image: "/images/slides/hero.jpg",
    },
    {
      eyebrow: "The Enterprise Reality",
      headline: "Towers of process.\nZero transparency.",
      body: "Procurement committees, legal review boards, security checklists — layers of approval that produce paper trails instead of machine-readable decisions.",
      image: "/images/slides/towers.jpg",
    },
    {
      eyebrow: "Why It Matters",
      headline: "Compliance fails\nwhen process is invisible.",
      body: "Regulators ask for evidence. Boards ask for rationale. Without machine-readable governance, you are rebuilding the story after the fact.",
      image: "/images/slides/problem.jpg",
    },
    {
      eyebrow: "The Scale",
      headline: "Every organization.\nEvery border.\nEvery decision.",
      body: "Vendor governance is not a local problem. Enterprises operate across jurisdictions, regulations, and risk profiles. The rules must travel with the data.",
      image: "/images/slides/earth.jpg",
    },
    {
      eyebrow: "The Converge Way",
      headline: "Governance\nas code.",
      body: "A Truth is a machine-readable governance spec. Intent, authority, constraints, and evidence — declared up front, validated automatically, auditable forever.",
      image: "/images/slides/converge.jpg",
    },
    {
      eyebrow: "The Pattern",
      headline: "Structure that\nemerges from flow.",
      body: "Governance is not a gate you pass through once. It is a continuous flow — living constraints that adapt as context shifts, not static checklists that rot.",
      image: "/images/slides/flow.jpg",
    },
    {
      eyebrow: "How It Works",
      headline: "Intent. Authority.\nConstraint. Evidence.",
      body: "Every vendor decision declares what outcome it seeks, who can approve it, what limits apply, and what proof is required. Agents propose. Humans promote.",
      image: "/images/slides/howit.jpg",
    },
    {
      eyebrow: "Deep Architecture",
      headline: "Layers that\ncompose cleanly.",
      body: "Domain packs, policy engines, promotion gates, and agent runtimes — each layer has a single responsibility. Compose them to build governance that fits your org.",
      image: "/images/slides/layers.jpg",
    },
    {
      eyebrow: "The Stack",
      headline: "Converge platform.\nOrganism runtime.\nHelm desktop.",
      body: "Write a truth. Validate it against policy. Act on it with confidence. Governance that learns from every decision — processes that strengthen under stress and keep your organization safe by default.",
      image: "/images/slides/stack.jpg",
    },
    {
      eyebrow: "Under The Hood",
      headline: "Silicon-level\nconfidence.",
      body: "From circuit board to policy decision — every layer is typed, validated, and auditable. Zero unsafe code. Zero runtime surprises. The machine earns trust.",
      image: "/images/slides/circuit.jpg",
    },
    {
      eyebrow: "The Grid",
      headline: "Structured decisions\nat enterprise scale.",
      body: "Not a single vendor scorecard — a composable grid of evaluations, constraints, and approvals that scales across teams, business units, and geographies.",
      image: "/images/slides/grid.jpg",
    },
    {
      eyebrow: "Living Systems",
      headline: "Governance that\nevolves organically.",
      body: "Organisms adapt. So should your vendor governance. As markets shift and regulations change, Truths update — and the audit trail shows exactly what changed and why.",
      image: "/images/slides/organic.jpg",
    },
    {
      eyebrow: "Corporate Trust",
      headline: "The board sees\nwhat the machine sees.",
      body: "No more translating between technical artifacts and executive summaries. One Truth spec serves the engineer, the auditor, and the board — same artifact, same source of truth.",
      image: "/images/slides/corporate.jpg",
    },
    {
      eyebrow: "Your Mission",
      headline: "Write a governance\ntruth that holds.",
      body: "Today you will write a vendor-selection Truth spec, validate it live against policy, and see how structured governance enables trustworthy AI decisions.",
      image: "/images/slides/students.jpg",
    },
  ];

  function nextSlide() {
    if (currentSlide < slides.length - 1) currentSlide++;
    else phase = "demo";
  }

  function prevSlide() {
    if (currentSlide > 0) currentSlide--;
  }

  function goToDemo() {
    phase = "demo";
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

  async function validate() {
    busy = true;
    error = "";
    simulation = null;
    try {
      // Step 1: Simulate (pre-flight)
      simulation = await invokeTauri("simulate_truth", { spec });

      // Step 2: Validate (full compiler)
      validation = await invokeTauri("validate_gherkin", { spec });

      // Step 3: Extract policy (Cedar preview)
      policy = await invokeTauri("extract_policy", { spec });
    } catch (cause) {
      validation = null;
      error = describeError(cause);
    } finally {
      busy = false;
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
  });
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
      <div class="flex items-center justify-between">
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Converge Governance</span>
        <button class="btn-ghost text-sm" onclick={goToDemo}>
          Skip to Demo &rarr;
        </button>
      </div>

      <!-- Main content -->
      {#key currentSlide}
        <div class="fade-in max-w-3xl">
          <p class="slide-eyebrow mb-4">{slides[currentSlide].eyebrow}</p>
          <h1 class="slide-headline mb-6 whitespace-pre-line">{slides[currentSlide].headline}</h1>
          <p class="slide-body">{slides[currentSlide].body}</p>

          {#if currentSlide === slides.length - 1}
            <div class="mt-10 flex gap-4">
              <button class="btn-lime" onclick={goToDemo}>
                Launch the Editor
              </button>
            </div>
          {/if}
        </div>
      {/key}

      <!-- Bottom nav -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          {#each slides as _, i}
            <button
              class="nav-dot"
              class:active={i === currentSlide}
              onclick={() => (currentSlide = i)}
              aria-label="Go to slide {i + 1}"
            ></button>
          {/each}
        </div>

        <div class="flex items-center gap-3">
          <span class="font-mono text-xs text-muted">
            {currentSlide + 1} / {slides.length}
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
        </div>
      </div>
    </div>
  </div>

<!-- ═══════════════════════════════════════════════
     DEMO PHASE
     ═══════════════════════════════════════════════ -->
{:else}
  <div class="min-h-screen bg-void">
    <!-- Top bar -->
    <header class="flex items-center justify-between border-b border-border px-8 py-4">
      <div class="flex items-center gap-4">
        <button class="btn-ghost text-sm" onclick={goToSlides}>
          &larr; Slides
        </button>
        <span class="font-mono text-xs tracking-widest text-muted uppercase">Spec Studio</span>
      </div>
      <button class="btn-ghost" type="button" onclick={openExampleOverlay}>Browse Examples</button>
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
          {#if error}
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
{/if}
