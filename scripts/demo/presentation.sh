#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# Converge Governance — AI Vendor Selection Presentation Demo
# ============================================================================
#
# Usage:
#   ./scripts/demo/presentation.sh           # Full walkthrough, mock Providers, default HITL reply
#   ./scripts/demo/presentation.sh step N    # Jump to step N (1-7)
#   ./scripts/demo/presentation.sh -l        # Use real LLM/search Providers
#   ./scripts/demo/presentation.sh --live    # Use real LLM/search Providers
#   ./scripts/demo/presentation.sh -v        # Show demo diagnostics
#   ./scripts/demo/presentation.sh --verbose # Show demo diagnostics
#   ./scripts/demo/presentation.sh --hitl    # Ask for a real HITL reply
#   ./scripts/demo/presentation.sh --nohitl  # Use default HITL reply
#   ./scripts/demo/presentation.sh --today   # Today's governed-selection track
#   ./scripts/demo/presentation.sh --creative # Creative Pareto-breakout track
#   ./scripts/demo/presentation.sh --doc PATH --static-facts PATH
#   ./scripts/demo/presentation.sh --no-pause
#
# Prerequisites:
#   cargo build -p governance-server
#   (for --live: OPENROUTER_API_KEY or a direct LLM key, plus BRAVE_API_KEY/TAVILY_API_KEY in .env)

LIVE_FLAG=""
STEP=""
TRACK="full"
NO_PAUSE="false"
VERBOSE="false"
HITL_MODE="nohitl"
DEMO_DATA_DIR="${DEMO_DATA_DIR:-examples/vendor-selection}"
VENDORS_FILE="${VENDORS_FILE:-$DEMO_DATA_DIR/demo-ai-vendors.json}"
ROUTER_FILE="${ROUTER_FILE:-$DEMO_DATA_DIR/demo-ai-provider-mix.json}"
COMPETITION_VENDORS_FILE="${COMPETITION_VENDORS_FILE:-$DEMO_DATA_DIR/demo-competition-vendors.json}"
COMPETITION_MATRIX_FILE="${COMPETITION_MATRIX_FILE:-$DEMO_DATA_DIR/competition-matrix.json}"
SOURCE_PACK_FILE="${SOURCE_PACK_FILE:-$DEMO_DATA_DIR/demo-source-pack.json}"
BUYER_BRIEF_FILE="${BUYER_BRIEF_FILE:-$DEMO_DATA_DIR/buyer-brief.md}"
EVALUATION_MODEL_FILE="${EVALUATION_MODEL_FILE:-$DEMO_DATA_DIR/evaluation-model.md}"
DOWNSTREAM_ACTIONS_FILE="${DOWNSTREAM_ACTIONS_FILE:-$DEMO_DATA_DIR/downstream-actions.md}"
RESULT_PACE="${DEMO_RESULT_PACE:-auto}"
RESULT_LINE_DELAY="${DEMO_RESULT_LINE_DELAY:-0.035}"
RESULT_BOX_DELAY="${DEMO_RESULT_BOX_DELAY:-0.18}"
COMPLIANCE_MODEL="${DEMO_COMPLIANCE_MODEL:-mistralai/mistral-small-2603}"
COST_MODEL="${DEMO_COST_MODEL:-arcee-ai/trinity-large-preview}"
RISK_MODEL="${DEMO_RISK_MODEL:-mistralai/mistral-small-2603}"
SYNTHESIS_MODEL="${DEMO_SYNTHESIS_MODEL:-writer/palmyra-x5}"
SPINNER_SOURCE="apps/desktop/src/lib/spinner.ts"
WORK_DIR="${TMPDIR:-/tmp}/governance-presentation-demo"
EXPERIENCE_PATH="$WORK_DIR/demo_presentation_experience_store.json"
STEP3_ANALYSIS_JSON="$WORK_DIR/step3-analysis.json"
STEP3_APPROVED_JSON="$WORK_DIR/step3-approved.json"
HITL_CHOICE="promote"
STOP_AFTER_HITL="false"
SPINNER_VERBS=("Thinking" "Processing" "Synthesizing" "Converging")
STATIC_FACTS_FILES=()

usage() {
  cat <<'EOF'
Helm AI vendor-selection demo

Core variants:
  just demo-today                  Today flow, mock Providers, default HITL reply
  just demo-today-live             Today flow, live Providers, default HITL reply
  just demo-creative               Creative flow, mock Providers, default HITL reply
  just demo-creative-live          Creative flow, live Providers, default HITL reply

Flags:
  --today                          Run the governed today flow
  --creative                       Run the creative Pareto flow
  -l, --live                       Use live remote Providers
  --mock                           Use deterministic Provider mocks
  --hitl                           Ask for a human HITL reply
  --nohitl, --no-hitl              Use the default HITL reply
  -v, --verbose, --verbode         Show diagnostics and source-pack details
  --doc PATH                       Buyer/source document to display and pass through
  --vendors PATH                   Governed vendor JSON
  --creative-vendors PATH          Creative/competition vendor JSON
  --criteria PATH                  Evaluation model document
  --static-facts PATH              Static facts file; may be repeated
  --data-dir PATH                  Source pack directory
  --no-pause                       Disable pacing pauses
  step N                           Run only one step
EOF
}

args=("$@")
index=0
while [ "$index" -lt "${#args[@]}" ]; do
  arg="${args[$index]}"
  case "$arg" in
    -h|--help)
      usage
      exit 0
      ;;
    -l|--live) LIVE_FLAG="--live" ;;
    --mock) LIVE_FLAG="" ;;
    -v|--verbose|--verbode) VERBOSE="true" ;;
    --hitl) HITL_MODE="hitl" ;;
    --nohitl|--no-hitl) HITL_MODE="nohitl" ;;
    --today|--today-only) TRACK="today" ;;
    --creative|--creative-only) TRACK="creative" ;;
    --track=today) TRACK="today" ;;
    --track=creative) TRACK="creative" ;;
    --track=full) TRACK="full" ;;
    --data-dir=*)
      DEMO_DATA_DIR="${arg#*=}"
      VENDORS_FILE="$DEMO_DATA_DIR/demo-ai-vendors.json"
      ROUTER_FILE="$DEMO_DATA_DIR/demo-ai-provider-mix.json"
      COMPETITION_VENDORS_FILE="$DEMO_DATA_DIR/demo-competition-vendors.json"
      COMPETITION_MATRIX_FILE="$DEMO_DATA_DIR/competition-matrix.json"
      SOURCE_PACK_FILE="$DEMO_DATA_DIR/demo-source-pack.json"
      BUYER_BRIEF_FILE="$DEMO_DATA_DIR/buyer-brief.md"
      EVALUATION_MODEL_FILE="$DEMO_DATA_DIR/evaluation-model.md"
      DOWNSTREAM_ACTIONS_FILE="$DEMO_DATA_DIR/downstream-actions.md"
      ;;
    --data-dir)
      index=$((index + 1))
      DEMO_DATA_DIR="${args[$index]}"
      VENDORS_FILE="$DEMO_DATA_DIR/demo-ai-vendors.json"
      ROUTER_FILE="$DEMO_DATA_DIR/demo-ai-provider-mix.json"
      COMPETITION_VENDORS_FILE="$DEMO_DATA_DIR/demo-competition-vendors.json"
      COMPETITION_MATRIX_FILE="$DEMO_DATA_DIR/competition-matrix.json"
      SOURCE_PACK_FILE="$DEMO_DATA_DIR/demo-source-pack.json"
      BUYER_BRIEF_FILE="$DEMO_DATA_DIR/buyer-brief.md"
      EVALUATION_MODEL_FILE="$DEMO_DATA_DIR/evaluation-model.md"
      DOWNSTREAM_ACTIONS_FILE="$DEMO_DATA_DIR/downstream-actions.md"
      ;;
    --doc=*|--document=*|--input-doc=*) BUYER_BRIEF_FILE="${arg#*=}" ;;
    --doc|--document|--input-doc)
      index=$((index + 1))
      BUYER_BRIEF_FILE="${args[$index]}"
      ;;
    --criteria=*|--evaluation-model=*) EVALUATION_MODEL_FILE="${arg#*=}" ;;
    --criteria|--evaluation-model)
      index=$((index + 1))
      EVALUATION_MODEL_FILE="${args[$index]}"
      ;;
    --vendors=*|--vendors-json=*) VENDORS_FILE="${arg#*=}" ;;
    --vendors|--vendors-json)
      index=$((index + 1))
      VENDORS_FILE="${args[$index]}"
      ;;
    --creative-vendors=*|--competition-vendors=*) COMPETITION_VENDORS_FILE="${arg#*=}" ;;
    --creative-vendors|--competition-vendors)
      index=$((index + 1))
      COMPETITION_VENDORS_FILE="${args[$index]}"
      ;;
    --static-facts=*) STATIC_FACTS_FILES+=("${arg#*=}") ;;
    --static-facts)
      index=$((index + 1))
      STATIC_FACTS_FILES+=("${args[$index]}")
      ;;
    --no-pause) NO_PAUSE="true" ;;
    --) : ;;
    step)
      index=$((index + 1))
      STEP="${args[$index]}"
      ;;
    [0-9]*) STEP="$arg" ;;
    *)
      echo "Unknown argument: $arg" >&2
      usage >&2
      exit 1
      ;;
  esac
  index=$((index + 1))
done

if [ "${#STATIC_FACTS_FILES[@]}" -eq 0 ] && [ -f "$DEMO_DATA_DIR/static-facts.json" ]; then
  STATIC_FACTS_FILES+=("$DEMO_DATA_DIR/static-facts.json")
fi

DEMO_BIN="cargo run -q -p governance-server --bin vendor-selection-demo --"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
NEON_GREEN='\033[38;5;46m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'
CLEAR_LINE='\033[2K'
HELM_VERSION="0.1.0"
AXIOM_VERSION="0.7.0"
ORGANISM_VERSION="1.4.0"
FERROX_VERSION="0.3.12"
CONVERGE_VERSION="3.7.4"
ACCENT_COLOR="$CYAN"
if [ "$TRACK" = "creative" ]; then
  ACCENT_COLOR="$NEON_GREEN"
fi

interrupt_demo() {
  echo ""
  echo -e "${DIM}Demo interrupted.${RESET}"
  exit 0
}

trap interrupt_demo INT

pause() {
  if [ "$NO_PAUSE" = "true" ]; then
    return
  fi
  echo ""
  echo -e "${DIM}Press Enter to continue...${RESET}"
  read -r
}

banner() {
  echo ""
  echo -e "${BOLD}${ACCENT_COLOR}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  $1${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

narrate() {
  echo -e "${YELLOW}  $1${RESET}"
}

pace_enabled() {
  case "$RESULT_PACE" in
    off|false|0) return 1 ;;
    on|true|1) return 0 ;;
    auto|"") [ "$NO_PAUSE" != "true" ] ;;
    *) [ "$NO_PAUSE" != "true" ] ;;
  esac
}

pace_line() {
  if pace_enabled; then
    sleep "$RESULT_LINE_DELAY"
  fi
}

pace_box() {
  if pace_enabled; then
    sleep "$RESULT_BOX_DELAY"
  fi
}

emit_line() {
  echo -e "$1"
  pace_line
}

emit_plain_line() {
  printf "%s\n" "$1"
  pace_line
}

pace_stream() {
  while IFS= read -r line || [ -n "$line" ]; do
    emit_plain_line "$line"
  done
  pace_box
}

masthead() {
  local short_pwd="${PWD/#$HOME/~}"
  echo ""
  echo -e "${BOLD}${ACCENT_COLOR}  ██╗  ██╗███████╗██╗     ███╗   ███╗${RESET}  ${DIM}Helm     v$HELM_VERSION${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  ██║  ██║██╔════╝██║     ████╗ ████║${RESET}  ${DIM}Axiom    v$AXIOM_VERSION${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  ███████║█████╗  ██║     ██╔████╔██║${RESET}  ${DIM}Organism v$ORGANISM_VERSION${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  ██╔══██║██╔══╝  ██║     ██║╚██╔╝██║${RESET}  ${DIM}Converge v$CONVERGE_VERSION${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  ██║  ██║███████╗███████╗██║ ╚═╝ ██║${RESET}  ${DIM}Ferrox   v$FERROX_VERSION${RESET}"
  echo -e "${BOLD}${ACCENT_COLOR}  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚═╝${RESET}  ${DIM}$short_pwd${RESET}"
}

box_text() {
  local title="$1"
  local text="$2"
  echo ""
  emit_line "${BOLD}${ACCENT_COLOR}┌─ $title${RESET}"
  emit_line "${ACCENT_COLOR}├────────────────────────────────────────────────────────────────────────────${RESET}"
  printf "%s\n" "$text" | while IFS= read -r line || [ -n "$line" ]; do
    emit_plain_line "│ $line"
  done
  emit_line "${ACCENT_COLOR}└────────────────────────────────────────────────────────────────────────────${RESET}"
  pace_box
}

box_cmd() {
  local title="$1"
  shift
  local output
  output=$("$@")
  echo ""
  emit_line "${BOLD}${ACCENT_COLOR}┌─ $title${RESET}"
  emit_line "${ACCENT_COLOR}├────────────────────────────────────────────────────────────────────────────${RESET}"
  printf "%s\n" "$output" | while IFS= read -r line || [ -n "$line" ]; do
    emit_plain_line "│ $line"
  done
  emit_line "${ACCENT_COLOR}└────────────────────────────────────────────────────────────────────────────${RESET}"
  pace_box
}

box_file_excerpt() {
  local title="$1"
  local file="$2"
  local max_lines="${3:-30}"

  if [ ! -f "$file" ]; then
    box_text "$title" "Source document missing: $file"
    return
  fi

  box_cmd "$title" sed -n "1,${max_lines}p" "$file"
}

require_demo_data() {
  local missing=0

  if [ ! -d "$DEMO_DATA_DIR" ]; then
    echo -e "${RED}Missing demo data directory: $DEMO_DATA_DIR${RESET}" >&2
    return 1
  fi

  for file in "$VENDORS_FILE" "$COMPETITION_VENDORS_FILE" "$COMPETITION_MATRIX_FILE"; do
    if [ ! -s "$file" ]; then
      echo -e "${RED}Missing required demo data file: $file${RESET}" >&2
      missing=1
    fi
  done
  for file in "${STATIC_FACTS_FILES[@]}"; do
    if [ ! -s "$file" ]; then
      echo -e "${RED}Missing static facts file: $file${RESET}" >&2
      missing=1
    fi
  done

  return "$missing"
}

data_pack_summary() {
  box_text "Demo data directory" "Source directory: $DEMO_DATA_DIR
Governed vendor data: $VENDORS_FILE
Competition vendor data: $COMPETITION_VENDORS_FILE
Competition evidence matrix: $COMPETITION_MATRIX_FILE

This is intentionally editable. Changing the JSON data should change the governed outcome or make the system honestly stop."

  if [ -f "$SOURCE_PACK_FILE" ]; then
    box_cmd "Source pack manifest" jq -r '
      "Documents:",
      (.documents[] | "• \(.path) — \(.purpose)"),
      "",
      "Data inputs:",
      (.data[] | "• \(.path) — steps \((.used_in_steps // []) | join(",")) — \(.purpose)")
    ' "$SOURCE_PACK_FILE"
  fi
}

static_facts_summary() {
  if [ "${#STATIC_FACTS_FILES[@]}" -eq 0 ]; then
    box_text "Static facts" "No static facts file was provided.
Run with --static-facts PATH to add buyer constraints, standing policies, or known organizational facts to this demo run."
    return
  fi

  for file in "${STATIC_FACTS_FILES[@]}"; do
    if jq empty "$file" >/dev/null 2>&1; then
      box_cmd "Static facts: $file" jq -r '
        if type == "array" then
          .[] | "• \(.id // .fact_id // "fact"): \(.statement // .value // .description // tostring)"
        elif has("facts") and (.facts | type == "array") then
          .facts[] | "• \(.id // .fact_id // "fact"): \(.statement // .value // .description // tostring)"
        else
          tostring
        end
      ' "$file"
    else
      box_file_excerpt "Static facts: $file" "$file" 36
    fi
  done
}

provider_plan() {
  if [ -n "$LIVE_FLAG" ]; then
    box_text "Real Provider plan" "Model routing is forced through OpenRouter model IDs when OPENROUTER_API_KEY is available:
• Compliance screening: $COMPLIANCE_MODEL — fast governance checks
• Cost / price analysis: $COST_MODEL — efficient value analysis
• Vendor risk: $RISK_MODEL — fast governance risk calls
• Decision synthesis: $SYNTHESIS_MODEL — business-facing recommendation narrative

Web evidence:
• Brave Search — broad market and compliance discovery
• Tavily Search — deeper canonical evidence and regulatory follow-up"
    return
  fi

  box_text "Mock Provider plan" "This run uses deterministic local Provider mocks.
No external API keys, search calls, or model calls are required.

The same truth runtime still executes:
• Compliance screening from declared vendor evidence
• Cost/value analysis from structured prices
• Risk scoring from configured inputs
• Multi-criteria optimization over capability, risk, cost, and certification coverage
• Synthesis and Cedar/HITL commitment gating"
}

demo_problem_statement() {
  box_text "User problem: complex AI vendor evaluation" "The user is not asking for a simple scorecard.
They need to evaluate AI vendors against criteria that pull in different directions:
• capability and business fit
• compliance posture and certifications
• operational, lock-in, and regulatory risk
• monthly cost and value
• authority, budget, HITL, and Cedar policy gates

Helm's job in this demo is to make that weighting explicit, combine the evidence through governed execution, and show why the decision converged or honestly stopped."
}

demo_options_summary() {
  box_text "Demo variants" "Same script, four common variants:
1. Today + Mock:      just demo-today
2. Today + Live:      just demo-today-live
3. Creative + Mock:   just demo-creative
4. Creative + Live:   just demo-creative-live

Modifiers:
• -v / --verbose / --verbode for diagnostics
• --hitl for interactive human approval, --nohitl for default approval
• --doc PATH, --vendors PATH, --criteria PATH, --static-facts PATH to test another source pack"
}

data_moat_principle() {
  box_text "Data moat principle" "The agent is not the moat.
The data around the agent is the moat.
Teams that understand this do not just move faster; they get better with every cycle.

In this demo, that data is explicit:
• source documents and static facts
• promoted context facts with provenance
• policy and HITL outcomes
• ExperienceStore run summaries that influence the next decision"
}

verbose_diagnostics() {
  if [ "$VERBOSE" != "true" ]; then
    return
  fi

  local provider_mode="mock"
  if [ -n "$LIVE_FLAG" ]; then
    provider_mode="live"
  fi

  box_text "Verbose diagnostics" "Track: $TRACK
Step filter: ${STEP:-all}
Provider mode: $provider_mode
HITL mode: $HITL_MODE
Pacing: $RESULT_PACE
No pause: $NO_PAUSE
Work directory: $WORK_DIR
Experience store: $EXPERIENCE_PATH
Demo binary: $DEMO_BIN

Data files:
• source document: $BUYER_BRIEF_FILE
• governed vendors: $VENDORS_FILE
• competition vendors: $COMPETITION_VENDORS_FILE
• competition matrix: $COMPETITION_MATRIX_FILE
• evaluation model: $EVALUATION_MODEL_FILE
• source manifest: $SOURCE_PACK_FILE
• static facts: ${STATIC_FACTS_FILES[*]:-none}

Models when live:
• compliance: $COMPLIANCE_MODEL
• cost: $COST_MODEL
• risk: $RISK_MODEL
• synthesis: $SYNTHESIS_MODEL"
}

upstream_assumptions() {
  box_text "Upstream assumptions: already true before Helm starts" "1. Management has defined the business problem and why it matters.
2. Procurement or the owning division has assigned an accountable owner.
3. Stakeholders are known: business, IT, security, legal, finance, and likely users.
4. A market scan or RFI has produced the candidate set.
5. The RFP package has been sent, responses have come back, and first-pass criteria exist.
6. The buyer has a budget envelope, timeline, and non-negotiable gates."

  box_text "What Helm receives in this demo" "A structured document pack:
• vendors and declared capabilities
• compliance status and certifications
• cost inputs
• risk inputs
• evaluation thresholds
• authority and HITL policy context

In the real product, Helm should help create and clean this pack. In today's demo, the pack already exists so we can focus on governed evaluation."

  box_text "What this means for RFI/RFP experts" "We are not skipping problem definition, stakeholder alignment, RFI discovery, vendor Q&A, demos, PoC, negotiation, contracting, or onboarding.
We are choosing a narrow, high-value slice: once the vendor responses and criteria exist, make the evaluation auditable, policy-bound, and easier to defend."
}

downstream_actions() {
  box_file_excerpt "Downstream actions from source pack" "$DOWNSTREAM_ACTIONS_FILE" 42

  box_text "Downstream actions after this run" "The demo does not end with a magic purchase order.
It creates a decision package that can move into the normal procurement path."

  box_text "What the decision package contains" "• recommended vendor or router strategy
• ranked alternatives and rejected candidates
• compliance, price, and risk evidence
• assumptions and hard constraints
• HITL/Cedar gate outcome
• open issues for legal, security, finance, or implementation
• audit trail showing which facts were promoted and why"

  box_text "What happens next in a real RFI/RFP process" "1. Management reviews the recommendation and alternatives.
2. Procurement runs clarification or best-and-final-offer if needed.
3. Legal/security validate contract terms, data handling, SLAs, and regulatory exposure.
4. Finance validates budget and commercial impact.
5. A PoC or pilot may be required before final award.
6. Contracting formalizes the vendor selection.
7. Onboarding turns the selected option into an implementation plan.
8. Outcomes feed back into the learning registry for the next decision."
}

load_spinner_verbs() {
  if [ ! -f "$SPINNER_SOURCE" ] || ! command -v perl >/dev/null 2>&1; then
    return
  fi

  local old_ifs="$IFS"
  local verbs=()
  IFS=$'\n'
  verbs=($(perl -0ne 'if (/SPINNER_VERBS\s*=\s*\[(.*?)\]/s) { my $body = $1; while ($body =~ /"([^"]+)"|'\''([^'\'']+)'\''/g) { print defined($1) ? "$1\n" : "$2\n"; } }' "$SPINNER_SOURCE"))
  IFS="$old_ifs"

  if [ "${#verbs[@]}" -gt 0 ]; then
    SPINNER_VERBS=("${verbs[@]}")
  fi
}

estimate_seconds() {
  local offline_seconds="$1"
  local live_seconds="$2"
  if [ -n "$LIVE_FLAG" ]; then
    echo "$live_seconds"
  else
    echo "$offline_seconds"
  fi
}

run_with_liveness() {
  local label="$1"
  local estimate="$2"
  shift 2

  echo ""
  echo -e "${DIM}Start processing: $label ... estimated time for this run ${estimate} seconds${RESET}"

  "$@" &
  local pid=$!
  local tick=0
  local offset=$((RANDOM % ${#SPINNER_VERBS[@]}))
  local frames='|/-\'
  local started
  started=$(date +%s)
  trap 'printf "\r${CLEAR_LINE}"; echo -e "${DIM}Demo interrupted. Stopping child process...${RESET}" >&2; kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; trap interrupt_demo INT; exit 0' INT

  while kill -0 "$pid" 2>/dev/null; do
    local now elapsed frame verb
    now=$(date +%s)
    elapsed=$((now - started))
    frame="${frames:$((tick % 4)):1}"
    verb="${SPINNER_VERBS[$(((offset + tick) % ${#SPINNER_VERBS[@]}))]}"
    printf "\r${CLEAR_LINE}${DIM}%s %s... elapsed %ss${RESET}" "$frame" "$verb" "$elapsed"
    sleep 1
    tick=$((tick + 1))
  done

  local status=0
  wait "$pid" || status=$?
  trap interrupt_demo INT
  printf "\r${CLEAR_LINE}"

  if [ "$status" -eq 0 ]; then
    echo -e "${DIM}Finished processing: $label${RESET}"
  else
    echo -e "${RED}Failed processing: $label${RESET}"
  fi

  return "$status"
}

build_source_args() {
  SOURCE_ARGS=("--source-doc=$BUYER_BRIEF_FILE")
  for file in "${STATIC_FACTS_FILES[@]}"; do
    SOURCE_ARGS+=("--static-facts=$file")
  done
}

run_json() {
  local output_file="$1"
  local vendors_file="$2"
  local tmp_file="$output_file.tmp"
  local err_file="$output_file.err"
  shift 2
  rm -f "$tmp_file" "$err_file"
  build_source_args

  if ! $DEMO_BIN \
    --json \
    --vendors-json="$vendors_file" \
    "${SOURCE_ARGS[@]}" \
    --experience-path="$EXPERIENCE_PATH" \
    --model-compliance="$COMPLIANCE_MODEL" \
    --model-cost="$COST_MODEL" \
    --model-risk="$RISK_MODEL" \
    --model-synthesis="$SYNTHESIS_MODEL" \
    "$@" > "$tmp_file" 2> "$err_file"; then
    echo "Demo execution failed while writing $output_file" >&2
    if [ -s "$err_file" ]; then
      cat "$err_file" >&2
    fi
    rm -f "$tmp_file"
    return 1
  fi

  if ! jq empty "$tmp_file" >/dev/null 2>&1; then
    echo "Demo execution produced invalid JSON for $output_file" >&2
    echo "" >&2
    echo "Captured stdout preview:" >&2
    sed -n '1,40p' "$tmp_file" >&2
    if [ -s "$err_file" ]; then
      echo "" >&2
      echo "Captured stderr:" >&2
      cat "$err_file" >&2
    fi
    return 1
  fi

  mv "$tmp_file" "$output_file"
  rm -f "$err_file"
}

run_business_capture() {
  local output_file="$1"
  local vendors_file="$2"
  shift 2
  build_source_args
  $DEMO_BIN \
    --business \
    --vendors-json="$vendors_file" \
    "${SOURCE_ARGS[@]}" \
    --experience-path="$EXPERIENCE_PATH" \
    --model-compliance="$COMPLIANCE_MODEL" \
    --model-cost="$COST_MODEL" \
    --model-risk="$RISK_MODEL" \
    --model-synthesis="$SYNTHESIS_MODEL" \
    "$@" > "$output_file" 2>&1
}

require_json_file() {
  local file="$1"
  local label="$2"

  if [ ! -s "$file" ]; then
    echo -e "${RED}Missing result for $label${RESET}" >&2
    echo "Expected JSON file: $file" >&2
    if [ -s "$file.err" ]; then
      echo "" >&2
      cat "$file.err" >&2
    fi
    return 1
  fi

  if ! jq empty "$file" >/dev/null 2>&1; then
    echo -e "${RED}Invalid JSON result for $label${RESET}" >&2
    echo "JSON file: $file" >&2
    return 1
  fi
}

hitl_decision() {
  if [ "$HITL_MODE" != "hitl" ]; then
    HITL_CHOICE="promote"
    box_text "HITL default reply" "HITL mode is disabled for this run.
The demo uses the default reply: promote.
Run with --hitl if you want to choose promote, escalate, or reject interactively."
    return
  fi

  echo ""
  echo -e "${BOLD}${YELLOW}HITL decision required:${RESET}"
  echo "  [p]romote  - approve this commitment"
  echo "  [e]scalate - keep human review open"
  echo "  [r]eject   - stop this commitment"
  printf "Your decision [p/e/r]: "
  read -r answer
  case "${answer:-p}" in
    p|P|promote|Promote) HITL_CHOICE="promote" ;;
    e|E|escalate|Escalate) HITL_CHOICE="escalate" ;;
    r|R|reject|Reject) HITL_CHOICE="reject" ;;
    *) HITL_CHOICE="promote" ;;
  esac
}

ask_cedar_delegation() {
  local decision="$1"
  box_text "Cedar delegation candidate" "permit(principal, action == Action::\"commit\", resource)
when {
  principal.authority == \"supervisory\" &&
  context.commitment_type == \"contract\" &&
  context.amount <= 50000 &&
  context.required_gates_met == true &&
  context.prior_pattern == \"accepted-ai-provider-selection\"
};"

  if [ "$HITL_MODE" != "hitl" ]; then
    box_text "Delegation answer" "Auto demo mode: yes, record this as a delegation candidate for the next matching decision."
    return
  fi

  printf "Can Cedar use this delegation pattern next time? [y/N]: "
  read -r answer
  case "${answer:-n}" in
    y|Y|yes|Yes)
      box_text "Delegation answer" "Yes. In the real product this would become a reviewed Cedar delegation candidate, not a silent bypass."
      ;;
    *)
      box_text "Delegation answer" "No. Keep the next matching decision in explicit HITL review."
      ;;
  esac

  box_text "What was delegated?" "Your HITL decision was: $decision
Cedar may only reuse this when the authority, budget envelope, gates, and evidence coverage match.
The delegation does not allow non-compliant vendors, missing evidence, or advisory authority to commit."
}

run_step() {
  if [ -n "$STEP" ] && [ "$1" -ne "$STEP" ]; then
    return
  fi
  "step_$1"
}

formation_trace() {
  local result_file="$1"
  box_cmd "Formation selected suggestors" jq -r '
    "Coverage: " + (((.projection.details.formation.coverage_ratio // 0) * 100) | round | tostring) + "%",
    "",
    ((.projection.details.formation.assignments // [])[]
      | "• role=\(.role) -> suggestor=\(.suggestor)"),
    "",
    "Roles and outputs:",
    ((.projection.details.agents // [])[]
      | "• \(.id): \(.role) | pack=\(.pack) | output=\(.output)")
  ' "$result_file"
}

source_material_trace() {
  local result_file="$1"
  box_cmd "Source material used by this run" jq -r '
    (.projection.details.source_material // {}) as $s
    | "Document: \($s.source_document.path // "none")",
      "Document lines: \($s.source_document.line_count // 0)",
      "Document bytes: \($s.source_document.byte_count // 0)",
      "",
      "Static facts: \($s.static_facts.fact_count // 0)",
      ((($s.static_facts.paths // [])[])
        | "• " + .)
  ' "$result_file"
}

formation_success_discussion() {
  local result_file="$1"
  box_cmd "Why this formation can succeed" jq -r '
    ((.projection.details.context.strategies // [])[] | select(.id=="strategy:vendor-sel:tactic") | .content) as $t
    | "Selected tactic: \($t.name)",
      "Reason: \($t.why)",
      "",
      "Resources:",
      "• candidates=\(.projection.details.resources.candidate_count)",
      "• evidence channels=\((.projection.details.resources.evidence_channels // []) | join(", "))",
      "• compute budget cycles=\(.projection.details.resources.compute_budget.max_cycles)",
      "",
      "Invariants:",
      ((.projection.details.invariants // [])[] | "• \(.id): \(.statement)")
  ' "$result_file"
}

suggestor_trigger_trace() {
  local result_file="$1"
  box_cmd "Suggestor trigger seeds and context writes" jq -r '
    "• planning-seed",
    "  trigger: no strategy facts exist for this run",
    "  writes: " + (([(.projection.details.context.strategies // [])[].id] | join(", "))),
    "",
    "• compliance-screener",
    "  trigger: strategy:vendor-sel:compliance",
    "  writes: " + (([(.projection.details.context.seeds // [])[] | select(.id | startswith("compliance:screen:")) | .id] | join(", "))),
    "",
    "• cost-analysis",
    "  trigger: compliance screen facts + strategy:vendor-sel:cost",
    "  writes: " + (([(.projection.details.context.evaluations // [])[] | select(.id | startswith("cost:estimate:")) | .id] | join(", "))),
    "",
    "• vendor-risk",
    "  trigger: cost facts + compliance facts + strategy:vendor-sel:risk",
    "  writes: " + (([(.projection.details.context.evaluations // [])[] | select(.id | startswith("risk:score:")) | .id] | join(", "))),
    "",
    "• vendor-shortlist",
    "  trigger: risk score facts + strategy:vendor-sel:shortlist",
    "  writes: vendor:shortlist",
    "",
    "• decision-synthesis",
    "  trigger: vendor:shortlist + strategy:vendor-sel:decision",
    "  writes: decision:recommendation",
    "",
    "• policy-gate",
    "  trigger: decision:recommendation + selected amount + authority/HITL context",
    "  writes: policy:decision:vendor-selection"
  ' "$result_file"
}

final_result_trace() {
  local result_file="$1"
  box_cmd "Final result" jq -r '
    "Converged: \(.converged)",
    "Cycles: \(.cycles)",
    "Stop reason: \(.stop_reason)",
    "",
    "Recommendation: \(.projection.details.recommendation.recommendation)",
    "Policy outcome: \(.projection.details.policy.outcome)",
    "Selected vendor: \(.projection.details.policy.selected_vendor)",
    "Selected amount: $\(.projection.details.policy.selected_amount_major)/mo",
    "",
    "Top shortlist:",
    ((.projection.details.shortlist.shortlist // [])[]
      | "#\(.rank) \(.vendor_name): composite=\(.composite_score), capability=\(.score), risk=\(.risk_score), cost=$\(.cost_major)/mo")
  ' "$result_file"
}

experience_store_trace() {
  local result_file="$1"
  box_cmd "ExperienceStore write" jq -r '
    (.projection.details.learning // {"status":"first_run","prior_runs":0}) as $l
    | (.projection.details.source_material // {}) as $s
    | "Run summary stored for truth_key=vendor-selection",
      "• cycles=\(.cycles)",
      "• converged=\(.converged)",
      "• confidence=\(.projection.details.recommendation.confidence)",
      "• recommended_vendor=\(.projection.details.policy.selected_vendor)",
      "• source_document=\($s.source_document.path // "none")",
      "• static_facts=\($s.static_facts.fact_count // 0)",
      "• prior_runs_available_to_this_run=\($l.prior_runs)",
      "• learning_status=\($l.status // "prior_context_available")",
      (if $l.consistent_recommendation then "• consistent_recommendation=\($l.consistent_recommendation.vendor) (\($l.consistent_recommendation.count)/\($l.consistent_recommendation.total_prior_runs))" else empty end),
      (if $l.latest_prior_source then "• latest_prior_source=\($l.latest_prior_source.source_document_path // "none") static_facts=\($l.latest_prior_source.static_fact_count // 0)" else empty end)
  ' "$result_file"
}

creative_alternative_formations() {
  box_text "Alternative formations compared" "1. Single-winner panel
   Good when criteria are stable and one vendor can satisfy the whole job.
   Weakness here: no single provider dominates coding, synthesis, search, governance, cost, and EU posture.

2. Strict policy huddle
   Good when authority and HITL risk are the main uncertainty.
   Weakness here: it can approve or stop, but it does not search the provider-mix frontier.

3. Self-organizing Pareto/router formation
   Selected for the creative run.
   It compares non-dominated alternatives, lets Ferrox-style optimization expose the feasible frontier, and lets Converge keep policy and provenance gates intact."
}

# ============================================================================
# STEP 1: The Stack
# ============================================================================
step_1() {
  banner "Step 1: Helm Stack For AI Vendor Selection"
  narrate "Converge — correctness-first multi-agent runtime"
  narrate "Organism — intelligence layer (intent, planning, adversarial, simulation, learning)"
  narrate "Axiom   — truth contracts and normative specifications"
  narrate "Ferrox  — optimization substrate for Pareto and constraint decisions"
  narrate "Helm    — this application (the hackathon governance tool)"
  echo ""
  narrate "The business flow is a Formation."
  narrate "Organism assembles a team of agents. Converge runs them to a fixed point."
  narrate "Every fact has provenance. Every decision has evidence."
  pause
}

# ============================================================================
# STEP 2: The Buyer's Document Pack
# ============================================================================
step_2() {
  banner "Step 2: AI Vendor Criteria Pack"
  narrate "A buyer submits a document pack describing what they need."
  narrate "Today this is normalized into structured vendor JSON."
  narrate "Tomorrow: raw documents, gap detection, contradiction flagging."
  echo ""
  echo -e "${GREEN}Vendor candidates:${RESET}"
  jq -r '.[] | "  \(.name)  score=\(.score)  risk=\(.risk_score)  compliance=\(.compliance_status)  cost=$\(.monthly_cost_minor / 100)/mo"' "$VENDORS_FILE" | pace_stream
  echo ""
  narrate "Notice: Qwen (Alibaba Cloud) has pending compliance and high risk."
  narrate "The system should detect this and handle it — not paper over it."

  data_pack_summary
  box_file_excerpt "Buyer brief from source pack" "$BUYER_BRIEF_FILE" 34
  box_file_excerpt "Evaluation model from source pack" "$EVALUATION_MODEL_FILE" 44
  static_facts_summary

  box_text "Demo boundary: what we cover" "We are focusing on the RFP evaluation and recommendation slice:
1. A structured buyer document pack exists.
2. Candidate vendors and declared evidence are available.
3. Evaluation criteria and hard gates exist.
4. The system screens compliance, price, risk, ranks candidates, synthesizes, and gates the commitment."

  upstream_assumptions

  box_text "Where this can expand" "Near-term: ingest less-structured RFI/RFP packs and detect missing, underspecified, or contradictory inputs.
Next: support vendor clarification loops, PoC evidence, negotiation constraints, and management decision packs.
Future breakout: challenge the sandbox when the better answer is not one submitted vendor, but a better architecture."
  pause
}

# ============================================================================
# STEP 3: Governed Selection (the full flow)
# ============================================================================
step_3() {
  banner "Step 3: Governed Vendor Selection"
  narrate "This is the full governed process. We will show the formation, each agent outcome, the synthesis, and then stop at the HITL gate."
  echo ""

  run_with_liveness \
    "governed selection without HITL approval" \
    "$(estimate_seconds 4 90)" \
    run_json "$STEP3_ANALYSIS_JSON" "$VENDORS_FILE" \
    --mode=governed \
    --min-score=75 \
    --max-risk=30 \
    --no-human-approval \
    $LIVE_FLAG
  require_json_file "$STEP3_ANALYSIS_JSON" "governed selection without HITL approval"

  source_material_trace "$STEP3_ANALYSIS_JSON"
  formation_trace "$STEP3_ANALYSIS_JSON"
  formation_success_discussion "$STEP3_ANALYSIS_JSON"
  suggestor_trigger_trace "$STEP3_ANALYSIS_JSON"

  box_text "Where we are in the process" "1. Intake: normalize the buyer document pack
2. Formation: decide which agent roles are needed
3. Compliance: screen mandatory compliance evidence
4. Price: compare cost and value
5. Risk: score operational, lock-in, and compliance risk
6. Optimization: rank feasible candidates with a transparent objective
7. Synthesis: explain the recommended choice
8. Gate: ask whether the decision can become a commitment"

  box_cmd "Formation: agents put to work" jq -r '
    .projection.details.agents[]
    | "• \(.id) — \(.role) → \(.output)"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Planning seed outcome" jq -r '
    (.projection.details.context.strategies[] | select(.id=="strategy:vendor-sel:tactic") | .content) as $t
    | "Tactic: \($t.name)\nWhy: \($t.why)"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Compliance screener outcome" jq -r '
    .projection.details.context.seeds[]
    | select(.id | startswith("compliance:screen:"))
    | .content
    | "• \(.vendor_name // .vendor): \(.compliance_status // .status) | certs=\((.certifications // []) | join("/"))"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Price analysis outcome" jq -r '
    .projection.details.optimization.rows[]
    | "• \(.vendor): cost=$\(.cost_major)/mo | cost_score=\(.cost_score) | objective=\(.objective_score)"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Risk analysis outcome" jq -r '
    .projection.details.context.evaluations[]
    | select(.id | startswith("risk:score:"))
    | .content
    | "• \(.vendor_name // .vendor): \(.risk_level // .overall_risk) | risk_score=\(.risk_score // "declared score used by optimizer")"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Shortlist optimizer outcome" jq -r '
    "Objective: " + .projection.details.optimization.objective,
    "",
    (.projection.details.shortlist.shortlist[]
      | "#\(.rank) \(.vendor_name): composite=\(.composite_score), capability=\(.score), risk=\(.risk_score), cost=$\(.cost_major)/mo"),
    "",
    "Rejected:",
    (.projection.details.shortlist.rejected[]
      | "• \(.vendor_name): \(.reasons | join("; "))")
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Decision synthesis outcome" jq -r '
    .projection.details.recommendation
    | "Recommendation: \(.recommendation)\nConfidence: \((.confidence * 100) | round)%\nHuman review requested by synthesis: \(.needs_human_review)"
  ' "$STEP3_ANALYSIS_JSON"

  box_cmd "Gate before HITL" jq -r '
    .projection.details.policy
    | "Outcome: \(.outcome)\nSelected vendor: \(.selected_vendor)\nAmount: $\(.selected_amount_major) / threshold $\(.hitl_threshold_major)\nHuman approval present: \(.human_approval_present)\nMeaning: the system has a candidate, but it cannot commit without the human gate."
  ' "$STEP3_ANALYSIS_JSON"

  hitl_decision
  decision="$HITL_CHOICE"
  case "$decision" in
    promote)
      run_with_liveness \
        "governed selection after HITL promote" \
        "$(estimate_seconds 4 90)" \
        run_json "$STEP3_APPROVED_JSON" "$VENDORS_FILE" \
        --mode=governed \
        --min-score=75 \
        --max-risk=30 \
        --human-approval \
        $LIVE_FLAG
      require_json_file "$STEP3_APPROVED_JSON" "governed selection after HITL promote"
      box_cmd "Gate after your HITL decision" jq -r '
        .projection.details.policy
        | "Your decision: promote\nCedar outcome: \(.outcome)\nCommitment: \(.selected_vendor) at $\(.selected_amount_major)/mo"
      ' "$STEP3_APPROVED_JSON"
      final_result_trace "$STEP3_APPROVED_JSON"
      experience_store_trace "$STEP3_APPROVED_JSON"
      ask_cedar_delegation "$decision"
      ;;
    escalate)
      box_text "Gate after your HITL decision" "Your decision: escalate
Cedar outcome: Escalate
Commitment: not promoted yet; the decision remains in human review."
      final_result_trace "$STEP3_ANALYSIS_JSON"
      experience_store_trace "$STEP3_ANALYSIS_JSON"
      box_text "Demo stopped at HITL" "This is honest stopping.
The evidence and recommendation are preserved, but the process does not promote a commitment.
In a real RFI/RFP process, the next action would be clarification, stakeholder review, or a revised decision package."
      STOP_AFTER_HITL="true"
      ;;
    reject)
      box_text "Gate after your HITL decision" "Your decision: reject
Cedar outcome: Reject
Commitment: stopped by the human gate; no vendor commitment is promoted."
      final_result_trace "$STEP3_ANALYSIS_JSON"
      experience_store_trace "$STEP3_ANALYSIS_JSON"
      box_text "Demo stopped at HITL" "This is honest stopping.
The system keeps the audit trail, but the selected commitment is not allowed to proceed.
In a real RFI/RFP process, the next action would be to revise the requirement, change the vendor set, or close the process."
      STOP_AFTER_HITL="true"
      ;;
  esac

  pause
}

# ============================================================================
# STEP 4: The HITL Gate
# ============================================================================
step_4() {
  banner "Step 4: Gate Mechanics"
  box_text "What the gate is doing" "The agents do not commit anything directly.
They produce a candidate decision: Mistral is the best governed choice.
The gate then asks a different question: is this actor allowed to turn that decision into a commitment?"

  box_text "Possible gate outcomes" "Promote  - the decision may become a commitment.
Escalate - the evidence is usable, but a required approval/delegation is missing.
Reject   - the actor or facts violate a hard policy boundary."

  box_text "Why this matters" "This is the separation we want the team to see:
Agent synthesis can recommend.
Optimization can rank.
Humans can approve.
Cedar can authorize.
Converge records which step allowed or stopped the decision."
  pause
}

# ============================================================================
# STEP 5: What the system detected (gap/contradiction handling)
# ============================================================================
step_5() {
  banner "Step 5: Negative Control — Honest Stopping"
  box_text "What this step is" "Yes, we just selected Mistral as the best governed candidate.
This step is educational and important: it proves the recommendation is not enough by itself.
We rerun the same evidence with weaker authority to show that policy can still stop the commitment."

  echo -e "${RED}Run: Advisory authority tries to commit${RESET}"
  local advisory_output="$WORK_DIR/step5-advisory.out"
  run_with_liveness \
    "advisory-authority negative control" \
    "$(estimate_seconds 3 80)" \
    run_business_capture "$advisory_output" "$VENDORS_FILE" \
    --mode=governed \
    --authority=advisory \
    $LIVE_FLAG
  grep -E "^(Decision|Flow status|Rejected or blocked inputs|  - Qwen|Policy gate|  Outcome|  Selected vendor|  Amount|  Human approval present|  Reason)" "$advisory_output" | pace_stream
  echo ""

  box_text "What happened" "The candidate decision still points at Mistral.
Qwen is still rejected for compliance/risk.
But advisory authority cannot commit, so Cedar rejects at the policy gate.
That is honest stopping: the system keeps the evidence but refuses the action."
  pause
}

# ============================================================================
# STEP 6: The Learning Loop
# ============================================================================
step_6() {
  banner "Step 6: The Learning Loop"
  box_text "Why restart the process?" "We restart to show what the next run can learn from the previous one.
The hard constraints do not change: compliance, risk threshold, objective, and Cedar gates still apply.
The seed that changes is prior_context: previous cycles, confidence, elapsed time, and selected vendor."

  box_text "What could realistically change?" "The synthesis can become more calibrated about trade-offs.
The system can detect repeated low-risk patterns.
A Cedar delegation candidate can become more defensible after accepted prior runs.
The optimizer should not suddenly promote a non-compliant or over-risk vendor."

  narrate "Running 3 sequential evaluations to grow prior_context..."
  echo ""

  for i in 1 2 3; do
    echo -e "${GREEN}Run $i:${RESET}"
    local learning_output="$WORK_DIR/step6-run-$i.out"
    run_with_liveness \
      "learning-loop evaluation $i of 3" \
      "$(estimate_seconds 3 80)" \
      run_business_capture "$learning_output" "$VENDORS_FILE" \
      --mode=governed \
      $LIVE_FLAG
    grep -E "^(Flow status|Decision|Learning|  Prior runs|  Consistent recommendation)" "$learning_output" | pace_stream
    echo ""
  done

  box_text "Learning interpretation" "Watch prior_runs increase.
That does not mean policy gets weaker.
It means the next synthesis has more experience context, and a future Cedar delegation can be justified with evidence."
  pause
}

# ============================================================================
# STEP 7: The Pareto Breakout (preview of tomorrow)
# ============================================================================
step_7() {
  if [ "$TRACK" = "creative" ]; then
    banner "Creative Demo: Complex Criteria Pareto Breakout"
  else
    banner "Step 7: Pareto Breakout — Real Optimization Case"
  fi
  box_text "Richer AI vendor request" "The same user problem has grown more realistic.
The buyer needs to evaluate AI vendors across agentic coding, structured synthesis, broad search, deep evidence, EU posture, low-risk operations, gateway governance, and cost.
Those criteria are complex to weight and combine, so a simple winner-takes-all scorecard is too weak."
  creative_alternative_formations

  box_cmd "Competition matrix read from data directory" jq -r '
    "Source: \(.meta.source)",
    "Scoring: \(.meta.scoring)",
    "Thesis: \(.winning_basket.thesis)",
    "Cost scenario: \(.cost_comparison.scenario)",
    "Savings: \(.cost_comparison.estimates.savings_ratio)",
    "",
    "Winning basket evidence:",
    (.winning_basket.roles[] | "• \(.workload): \(.model) — \(.why)")
  ' "$COMPETITION_MATRIX_FILE"

  box_cmd "Richer candidate landscape" jq -r '
    .[]
    | "• \(.name): score=\(.score) risk=\(.risk_score) cost=$\(.monthly_cost_minor / 100)/mo compliance=\(.compliance_status) certs=\((.certifications // []) | join("/"))"
  ' "$COMPETITION_VENDORS_FILE"
  echo ""
  if [ -n "$LIVE_FLAG" ]; then
    narrate "Next: run the creative optimization with real Provider routing and web evidence."
  else
    narrate "Next: run the creative optimization with deterministic Provider mocks."
  fi
  pause

  breakout_json="$WORK_DIR/step7-breakout.json"
  run_with_liveness \
    "Pareto breakout optimization" \
    "$(estimate_seconds 4 140)" \
    run_json "$breakout_json" "$COMPETITION_VENDORS_FILE" \
    --mode=pareto-breakout \
    --min-score=75 \
    --max-risk=30 \
    $LIVE_FLAG
  require_json_file "$breakout_json" "Pareto breakout optimization"

  source_material_trace "$breakout_json"
  formation_trace "$breakout_json"
  formation_success_discussion "$breakout_json"
  suggestor_trigger_trace "$breakout_json"

  box_cmd "Governed Pareto frontier from the optimizer" jq -r '
    .projection.details.optimization.rows[]
    | select(.pareto_frontier == true and .feasible == true)
    | "• \(.vendor): objective=\(.objective_score), capability=\(.score), risk=\(.risk), cost=$\(.cost_major)/mo"
  ' "$breakout_json"

  box_cmd "Breakout decision" jq -r '
    (.projection.details.context.strategies[] | select(.id=="strategy:vendor-sel:router-hypothesis") | .content) as $r
    | "Router fit: \($r.router_fit)\nDecision: \($r.demo_line)\nWhy: \($r.why)"
  ' "$breakout_json"

  box_cmd "Provider mix proposed" jq -r '
    .projection.details.context.strategies[]
    | select(.id=="strategy:vendor-sel:router-hypothesis")
    | .content.provider_mix[]
    | "• \(.need) -> \(.route)"
  ' "$breakout_json"

  box_text "Why this is more trustworthy" "The breakout is now grounded in a richer input set and a visible Pareto frontier.
The system still produces the best single-candidate shortlist, but it also identifies that several non-dominated options serve different needs.
That is the moment where the formation can say: the better answer is not one vendor; it is a governed router strategy."
  final_result_trace "$breakout_json"
  experience_store_trace "$breakout_json"
  pause
}

# ============================================================================
# MAIN
# ============================================================================

load_spinner_verbs
require_demo_data

masthead
case "$TRACK" in
  creative) banner "Converge Governance — AI Vendor Selection Creative Demo" ;;
  today) banner "Converge Governance — AI Vendor Selection Today Demo" ;;
  *) banner "Converge Governance — AI Vendor Selection Demo" ;;
esac
narrate "Presenter: Kenneth Pernyer / Reflective Labs"
narrate "Date: $(date +%Y-%m-%d)"
if [ -n "$LIVE_FLAG" ]; then
  narrate "Mode: LIVE (real Providers)"
else
  narrate "Mode: MOCK (deterministic Provider mocks, no API keys needed)"
fi
if [ "$VERBOSE" = "true" ]; then
  narrate "Verbose: enabled"
fi
narrate "Track: $TRACK"
narrate "Data: $DEMO_DATA_DIR"
echo ""
provider_plan
demo_options_summary
demo_problem_statement
data_moat_principle
verbose_diagnostics

# Clean experience store for a fresh demo
mkdir -p "$WORK_DIR"
rm -f "$EXPERIENCE_PATH"
rm -f "$STEP3_ANALYSIS_JSON" "$STEP3_APPROVED_JSON" "$WORK_DIR/step7-breakout.json"
rm -f "$WORK_DIR"/step5-*.out "$WORK_DIR"/step6-*.out

case "$TRACK" in
  today)
    run_step 1
    run_step 2
    run_step 3
    if [ "$STOP_AFTER_HITL" != "true" ]; then
      run_step 4
      run_step 5
      run_step 6
    fi
    ;;
  creative)
    run_step 1
    run_step 2
    run_step 7
    ;;
  full)
    run_step 1
    run_step 2
    run_step 3
    if [ "$STOP_AFTER_HITL" != "true" ]; then
      run_step 4
      run_step 5
      run_step 6
      run_step 7
    fi
    ;;
  *)
    echo -e "${RED}Unknown demo track: $TRACK${RESET}" >&2
    exit 1
    ;;
esac

if [ "$STOP_AFTER_HITL" != "true" ]; then
  case "$TRACK" in
    today)
      if [ -z "$STEP" ]; then
        downstream_actions
      fi
      ;;
    creative)
      downstream_actions
      ;;
    full)
      if [ -z "$STEP" ] || [ "$STEP" -eq 7 ]; then
        downstream_actions
      fi
      ;;
  esac
fi

banner "End of Demo"
case "$TRACK" in
  today)
    narrate "Today:    Governed selection — find the best from a provided list."
    if [ "$STOP_AFTER_HITL" = "true" ]; then
      narrate "Outcome:  The HITL decision stopped the commitment before promotion."
    else
      narrate "Outcome:  A governed decision package is ready for downstream procurement."
    fi
    ;;
  creative)
    narrate "Creative: Pareto breakout — reframe the request when one vendor is the wrong abstraction."
    narrate "Outcome:  A governed provider mix behind OpenRouter/Kong is the stronger candidate."
    ;;
  full)
    narrate "Today:    Governed selection — find the best from a provided list."
    if [ "$STOP_AFTER_HITL" = "true" ]; then
      narrate "Outcome:  The HITL decision stopped the commitment before promotion."
    else
      narrate "Tomorrow: Hypothetics — break the sandbox, reframe the problem,"
      narrate "          come up with alternatives that better fulfill the real needs."
    fi
    ;;
esac
narrate ""
narrate "The AI assistance sticks to needs, not solutions."
narrate "Converge ensures: every fact has provenance, every decision has evidence,"
narrate "and the system converges OR honestly stops."
echo ""
