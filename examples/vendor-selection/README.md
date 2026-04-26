# Vendor Selection Demo Data

This directory is the editable source pack for the current vendor-selection demo.

The presentation script reads from this directory by default:

```bash
just demo
just demo-live
just demo-today
just demo-creative
```

To point the demo at a copied or modified data pack:

```bash
DEMO_DATA_DIR=/path/to/vendor-selection-data just demo
DEMO_DATA_DIR=/path/to/vendor-selection-data just demo-live
```

Interactive runs pace result boxes line by line so the presenter can follow the flow and scroll back with context. Automated runs using `--no-pause` stay fast.

To override pacing:

```bash
DEMO_RESULT_PACE=off just demo-live
DEMO_RESULT_PACE=on DEMO_RESULT_LINE_DELAY=0.05 just demo-live
```

## Files Used By The Demo

| File | Purpose |
|---|---|
| `demo-ai-vendors.json` | The governed Step 3-6 vendor list used for the stable Mistral/Qwen story. |
| `demo-competition-vendors.json` | The richer Step 7 model/provider candidate list derived from the competition matrix. |
| `demo-ai-strategy-candidates.json` | Two-candidate strategy comparison: premium single-model vs governed router mix. |
| `demo-ai-strategy-candidates.md` | Business-facing explanation of the two strategy candidates. |
| `competition-matrix.json` | Role-level competition evidence used to explain the router/provider-mix breakout. |
| `demo-ai-provider-mix.json` | Earlier router/provider-mix scenario retained as a compact fallback scenario. |
| `buyer-brief.md` | Business problem, scope, stakeholders, constraints, and success criteria assumed before Helm starts. |
| `evaluation-model.md` | Criteria, hard gates, objective weights, and tuning guidance. |
| `downstream-actions.md` | What the decision package should trigger after the demo run. |
| `demo-source-pack.json` | Machine-readable manifest for the source pack. |

## How To Tune The Experience

Change vendor capability with `score`.

Change governance pressure with `risk_score` and `compliance_status`.

Change price sensitivity with `monthly_cost_minor`.

Change evidence coverage with `certifications`.

Change the future-state/router narrative by editing `competition-matrix.json` and `demo-competition-vendors.json`.

Use `demo-ai-strategy-candidates.json` when you want the shortest possible executive comparison: one premium model everywhere versus a governed multi-model router strategy.

The demo should still converge or honestly stop. If a data edit makes the story incoherent, that is useful feedback: the next step is to improve the source pack or the formation logic, not hide the contradiction.

## Live Model Routing

The live presenter scripts use OpenRouter model IDs by default:

| Role | Default model | Why |
|---|---|---|
| Compliance screening | `mistralai/mistral-small-2603` | Fast governance checks. |
| Cost / price analysis | `arcee-ai/trinity-large-preview` | Efficient secondary analysis. |
| Vendor risk | `mistralai/mistral-small-2603` | Fast governance risk calls. |
| Decision synthesis | `writer/palmyra-x5` | Business-facing recommendation narrative. |

The live compliance agent uses Brave Search for broad discovery and Tavily Search for deeper evidence follow-up.

Override models with environment variables:

```bash
DEMO_SYNTHESIS_MODEL=writer/palmyra-x5 just demo-today
DEMO_COST_MODEL=arcee-ai/trinity-large-preview just demo-creative
```
