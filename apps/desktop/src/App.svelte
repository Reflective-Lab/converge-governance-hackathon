<script>
  import { onDestroy } from "svelte";
  import { invokeTauri } from "./lib/tauri.js";

  const exampleSpec = `Truth: Enterprise AI vendor selection is auditable, constrained, and approval-gated
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
`;

  let spec = exampleSpec;
  let validation = null;
  let error = "";
  let busy = false;
  let truthGuidance = null;
  let truthGuidanceError = "";
  let truthGuidanceBusy = false;
  let textareaEl;
  let highlightEl;
  let guidanceTimer = null;
  let lastGuidanceSpec = "";
  let guidanceSequence = 0;

  function describeError(cause) {
    if (cause instanceof Error) {
      return cause.message;
    }

    if (typeof cause === "string") {
      return cause;
    }

    if (cause && typeof cause === "object") {
      if (typeof cause.message === "string" && cause.message.length > 0) {
        return cause.message;
      }

      try {
        return JSON.stringify(cause);
      } catch {
        return "Validation failed for an unknown reason.";
      }
    }

    return "Validation failed for an unknown reason.";
  }

  async function validate() {
    busy = true;
    error = "";

    try {
      validation = await invokeTauri("validate_gherkin", { spec });
    } catch (cause) {
      validation = null;
      error = describeError(cause);
    } finally {
      busy = false;
    }
  }

  function loadExample() {
    spec = exampleSpec;
    error = "";
    validation = null;
  }

  function clearGuidanceTimer() {
    if (guidanceTimer) {
      clearTimeout(guidanceTimer);
      guidanceTimer = null;
    }
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
      if (requestId !== guidanceSequence) {
        return;
      }
      truthGuidance = response;
    } catch (cause) {
      if (requestId !== guidanceSequence) {
        return;
      }
      truthGuidance = null;
      truthGuidanceError = describeError(cause);
    } finally {
      if (requestId === guidanceSequence) {
        truthGuidanceBusy = false;
      }
    }
  }

  function scheduleTruthGuidance(nextSpec) {
    if (nextSpec === lastGuidanceSpec) {
      return;
    }

    lastGuidanceSpec = nextSpec;
    clearGuidanceTimer();

    if (!extractTruthHeading(nextSpec)) {
      truthGuidance = null;
      truthGuidanceError = "";
      truthGuidanceBusy = false;
      return;
    }

    guidanceTimer = setTimeout(() => {
      requestTruthGuidance(nextSpec);
    }, 650);
  }

  function applyTruthSuggestion() {
    if (!truthGuidance?.shouldRewrite || !truthGuidance?.suggestedTitle) {
      return;
    }

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
    return value
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;");
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
    return rawLine
      .trim()
      .split(/\s+/)
      .map((tag) => `<span class="token tag">${escapeHtml(tag)}</span>`)
      .join(" ");
  }

  function renderTable(rawLine, lineNumber) {
    const cells = rawLine.split("|").map((cell) => cell.trim());
    const rendered = cells
      .map((cell, index) => {
        if (index === 0 || index === cells.length - 1) {
          return '<span class="token punctuation">|</span>';
        }
        return `<span class="token table-cell">${highlightInline(` ${cell} `)}</span><span class="token punctuation">|</span>`;
      })
      .join("");

    return wrapLine(lineNumber, "line-table", rendered);
  }

  function renderStructuredLine(rawLine, index) {
    const lineNumber = index + 1;

    if (rawLine.length === 0) {
      return wrapLine(lineNumber, "line-empty", "");
    }

    if (/^\s*#/.test(rawLine)) {
      return wrapLine(
        lineNumber,
        "line-comment",
        `<span class="token comment">${escapeHtml(rawLine)}</span>`
      );
    }

    const featureMatch = rawLine.match(/^(\s*)(Truth|Feature)(:)(.*)$/);
    if (featureMatch) {
      const [, indent, keyword, punctuation, rest] = featureMatch;
      return wrapLine(
        lineNumber,
        "line-feature",
        `${indent}<span class="token keyword keyword-feature">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`
      );
    }

    const governanceMatch = rawLine.match(
      /^(\s*)(Intent|Authority|Constraint|Evidence|Exception)(:)(.*)$/
    );
    if (governanceMatch) {
      const [, indent, keyword, punctuation, rest] = governanceMatch;
      return wrapLine(
        lineNumber,
        "line-governance",
        `${indent}<span class="token keyword keyword-governance">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`
      );
    }

    const scenarioMatch = rawLine.match(/^(\s*)(Scenario(?: Outline)?)(:)(.*)$/);
    if (scenarioMatch) {
      const [, indent, keyword, punctuation, rest] = scenarioMatch;
      return wrapLine(
        lineNumber,
        "line-scenario",
        `${indent}<span class="token keyword keyword-scenario">${keyword}</span><span class="token punctuation">${punctuation}</span><span class="token title">${highlightInline(rest)}</span>`
      );
    }

    const stepMatch = rawLine.match(/^(\s*)(Given|When|Then|And|But)(\s+)(.*)$/);
    if (stepMatch) {
      const [, indent, keyword, spacer, rest] = stepMatch;
      return wrapLine(
        lineNumber,
        "line-step",
        `${indent}<span class="token keyword keyword-step">${keyword}</span>${spacer}${highlightInline(rest)}`
      );
    }

    if (/^\s*@/.test(rawLine)) {
      const indent = rawLine.match(/^\s*/)?.[0] ?? "";
      return wrapLine(lineNumber, "line-tags", `${indent}${renderTags(rawLine)}`);
    }

    if (/^\s*\|/.test(rawLine)) {
      return renderTable(rawLine, lineNumber);
    }

    return wrapLine(lineNumber, "line-plain", highlightInline(rawLine));
  }

  function renderHighlightedSpec(value) {
    return value
      .split("\n")
      .map((line, index) => renderStructuredLine(line, index))
      .join("");
  }

  $: issues = validation?.issues ?? [];
  $: errors = issues.filter((issue) => issue.severity === "error");
  $: warnings = issues.filter((issue) => issue.severity === "warning");
  $: highlightedSpec = renderHighlightedSpec(spec);
  $: scheduleTruthGuidance(spec);

  onDestroy(() => {
    guidanceSequence += 1;
    clearGuidanceTimer();
  });
</script>

<svelte:head>
  <title>Converge Governance Desktop</title>
  <meta
    name="description"
    content="Desktop editor for Converge vendor-selection Gherkin validation."
  />
</svelte:head>

<main class="shell">
  <section class="hero">
    <div class="hero-copy">
      <p class="eyebrow">Vendor Selection Spec Studio</p>
      <h1>Write a Converge Truth with live guidance, not just after-the-fact validation.</h1>
      <p class="lede">
        The editor now evaluates the current <code>Truth:</code> heading as you type and suggests
        a stronger formulation in the browser before you validate the full spec.
      </p>
    </div>

    <div class="hero-actions">
      <button class="ghost" type="button" on:click={loadExample}>Load Example</button>
      <button class="primary" type="button" on:click={validate} disabled={busy}>
        {#if busy}Validating…{:else}Validate Spec{/if}
      </button>
    </div>
  </section>

  <section class="workspace">
    <label class="editor-card">
      <span class="card-label">Gherkin / Truth Spec</span>
      <div class="editor-surface">
        <div class="editor-stack">
          <div class="highlight-layer" bind:this={highlightEl} aria-hidden="true">
            {@html highlightedSpec}
          </div>
          {#if !spec.trim()}
            <div class="editor-empty">Paste vendor-selection Gherkin or Truth syntax here.</div>
          {/if}
          <textarea bind:this={textareaEl} bind:value={spec} spellcheck="false" on:scroll={syncScroll}></textarea>
        </div>
      </div>
    </label>

    <aside class="results-card">
      <section class="guidance-panel">
        <div class="results-header">
          <span class="card-label">Truth Guidance</span>
          {#if truthGuidance}
            <span class:source-pill={truthGuidance.source === "kong-llm"} class:fallback-pill={truthGuidance.source !== "kong-llm"}>
              {truthGuidance.source === "kong-llm" ? "Kong LLM" : "Local Fallback"}
            </span>
          {/if}
        </div>

        {#if truthGuidanceBusy}
          <div class="callout neutral-callout">
            <strong>Evaluating heading</strong>
            <p>The editor is checking whether the current Truth line reads like a governed rule.</p>
          </div>
        {:else if truthGuidanceError}
          <div class="callout error-callout">
            <strong>Guidance error</strong>
            <p>{truthGuidanceError}</p>
          </div>
        {:else if truthGuidance}
          <div class="callout neutral-callout">
            <strong>{truthGuidance.shouldRewrite ? "Suggested rewrite available" : "Heading already reads like a rule"}</strong>
            <p>{truthGuidance.note}</p>
          </div>

          <div class="truth-preview-grid">
            <article class="truth-preview-card">
              <span>Current</span>
              <strong>Truth: {truthGuidance.currentTitle}</strong>
            </article>
            <article class="truth-preview-card">
              <span>Suggested</span>
              <strong>Truth: {truthGuidance.suggestedTitle}</strong>
            </article>
          </div>

          {#if truthGuidance.shouldRewrite}
            <button class="ghost secondary-action" type="button" on:click={applyTruthSuggestion}>
              Apply Suggested Title
            </button>
          {/if}

          <div class="guidance-stack">
            <h2>Why</h2>
            {#each truthGuidance.rationale as reason}
              <p class="guidance-item">{reason}</p>
            {/each}
          </div>

          {#if truthGuidance.descriptionHints.length > 0}
            <div class="guidance-stack">
              <h2>Description Hints</h2>
              {#each truthGuidance.descriptionHints as hint}
                <p class="guidance-item">{hint}</p>
              {/each}
            </div>
          {/if}
        {:else}
          <div class="callout neutral-callout">
            <strong>Ready to guide</strong>
            <p>Add a <code>Truth:</code> line and the editor will critique the heading as you type.</p>
          </div>
        {/if}
      </section>

      <section class="validation-panel">
        <div class="results-header">
          <span class="card-label">Validation</span>
          {#if validation}
            <span class:valid-pill={validation.isValid} class:invalid-pill={!validation.isValid}>
              {validation.isValid ? "Valid" : "Needs Work"}
            </span>
          {/if}
        </div>

        {#if error}
          <div class="callout error-callout">
            <strong>Command error</strong>
            <p>{error}</p>
          </div>
        {:else if validation}
          <div class="metrics">
            <article>
              <span>Scenarios</span>
              <strong>{validation.scenarioCount}</strong>
            </article>
            <article>
              <span>Errors</span>
              <strong>{errors.length}</strong>
            </article>
            <article>
              <span>Warnings</span>
              <strong>{warnings.length}</strong>
            </article>
          </div>

          <div class="callout neutral-callout">
            <strong>{validation.summary}</strong>
            <p>{validation.notes[0]}</p>
          </div>

          <div class="governance-grid">
            <div>
              <span>Intent</span>
              <strong>{validation.governance.intent ? "Present" : "Missing"}</strong>
            </div>
            <div>
              <span>Authority</span>
              <strong>{validation.governance.authority ? "Present" : "Missing"}</strong>
            </div>
            <div>
              <span>Constraint</span>
              <strong>{validation.governance.constraint ? "Present" : "Missing"}</strong>
            </div>
            <div>
              <span>Evidence</span>
              <strong>{validation.governance.evidence ? "Present" : "Missing"}</strong>
            </div>
          </div>

          {#if validation.scenarios.length > 0}
            <div class="scenario-list">
              <h2>Scenarios</h2>
              {#each validation.scenarios as scenario}
                <div class="scenario-item">
                  <strong>{scenario.name}</strong>
                  <span>{scenario.kind ?? "untagged"}</span>
                </div>
              {/each}
            </div>
          {/if}

          <div class="issue-list">
            <h2>Findings</h2>
            {#if issues.length === 0}
              <p class="empty">No findings. The local validator accepted this spec.</p>
            {:else}
              {#each issues as issue}
                <article class="issue" data-severity={issue.severity}>
                  <div class="issue-topline">
                    <strong>{issue.message}</strong>
                    <span>{issue.severity}</span>
                  </div>
                  <p>{issue.location} · {issue.category}</p>
                  {#if issue.suggestion}
                    <p class="suggestion">{issue.suggestion}</p>
                  {/if}
                </article>
              {/each}
            {/if}
          </div>
        {:else}
          <div class="callout neutral-callout">
            <strong>Ready to validate</strong>
            <p>
              The validator runs locally in Tauri. Paste vendor-selection Gherkin and click
              <em>Validate Spec</em>.
            </p>
          </div>
        {/if}
      </section>
    </aside>
  </section>
</main>
