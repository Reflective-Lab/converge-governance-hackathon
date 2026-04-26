#!/usr/bin/env bash
set -euo pipefail

DEMO_DATA_DIR="${DEMO_DATA_DIR:-examples/vendor-selection}"
VENDORS_FILE="$DEMO_DATA_DIR/demo-ai-vendors.json"
ROUTER_FILE="$DEMO_DATA_DIR/demo-ai-provider-mix.json"
COMPETITION_VENDORS_FILE="$DEMO_DATA_DIR/demo-competition-vendors.json"
COMPETITION_MATRIX_FILE="$DEMO_DATA_DIR/competition-matrix.json"
STRATEGY_CANDIDATES_FILE="$DEMO_DATA_DIR/demo-ai-strategy-candidates.json"
TMP_DIR="${TMPDIR:-/tmp}/governance-demo-verify-$$"
EXPERIENCE_PATH="$TMP_DIR/experience.json"
DEMO_BIN=(cargo run -q -p governance-server --bin vendor-selection-demo --)

mkdir -p "$TMP_DIR"

pass() {
  printf 'ok - %s\n' "$1"
}

fail() {
  printf 'not ok - %s\n' "$1" >&2
  exit 1
}

run_json() {
  local output_file="$1"
  shift
  "${DEMO_BIN[@]}" \
    --vendors-json="$VENDORS_FILE" \
    --experience-path="$EXPERIENCE_PATH" \
    --json \
    "$@" > "$output_file"
}

run_json_with_file() {
  local output_file="$1"
  local vendors_file="$2"
  shift 2
  "${DEMO_BIN[@]}" \
    --vendors-json="$vendors_file" \
    --experience-path="$EXPERIENCE_PATH" \
    --json \
    "$@" > "$output_file"
}

assert_jq() {
  local file="$1"
  local expression="$2"
  local message="$3"
  if jq -e "$expression" "$file" > /dev/null; then
    pass "$message"
  else
    fail "$message"
  fi
}

governed="$TMP_DIR/governed.json"
no_approval="$TMP_DIR/no-approval.json"
advisory="$TMP_DIR/advisory.json"
breakout="$TMP_DIR/breakout.json"
competition_breakout="$TMP_DIR/competition-breakout.json"
strategy_candidates="$TMP_DIR/strategy-candidates.json"
business="$TMP_DIR/business.txt"

jq empty "$COMPETITION_MATRIX_FILE" \
  && pass "competition matrix JSON is valid" \
  || fail "competition matrix JSON is valid"

jq empty "$COMPETITION_VENDORS_FILE" \
  && pass "competition vendor JSON is valid" \
  || fail "competition vendor JSON is valid"

jq empty "$STRATEGY_CANDIDATES_FILE" \
  && pass "strategy candidate JSON is valid" \
  || fail "strategy candidate JSON is valid"

run_json "$governed" --mode=governed --min-score=75 --max-risk=30
assert_jq "$governed" '.converged == true' "governed run converges"
assert_jq "$governed" '.cycles == 8' "governed run reaches the expected fixed point cycle"
assert_jq "$governed" '.projection.details.policy.selected_vendor == "Mistral"' "Mistral is the selected vendor"
assert_jq "$governed" '.projection.details.shortlist.shortlist[0].vendor_name == "Mistral"' "Mistral ranks first"
assert_jq "$governed" '.projection.details.shortlist.shortlist[0].composite_score == 79.2' "Mistral has the expected composite score"
assert_jq "$governed" 'any(.projection.details.shortlist.rejected[]; .vendor_name == "Qwen (Alibaba Cloud)" and (.reasons | index("non-compliant (pending)") != null) and (.reasons | index("risk 35 above maximum 30") != null))' "Qwen is rejected for compliance and risk"

run_json "$no_approval" --mode=governed --no-human-approval
assert_jq "$no_approval" '.projection.details.policy.outcome == "Escalate"' "missing human approval escalates"

run_json "$advisory" --mode=governed --authority=advisory
assert_jq "$advisory" '.projection.details.policy.outcome == "Reject"' "advisory authority cannot commit"

run_json_with_file "$breakout" "$ROUTER_FILE" --mode=pareto-breakout --min-score=75 --max-risk=30
assert_jq "$breakout" '(.projection.details.context.strategies[] | select(.id == "strategy:vendor-sel:router-hypothesis") | .content.router_fit) == true' "Pareto breakout router hypothesis fires"
assert_jq "$breakout" '(.projection.details.context.strategies[] | select(.id == "strategy:vendor-sel:router-hypothesis") | .content.name) == "router-first-provider-strategy"' "breakout proposes router-first strategy"
assert_jq "$breakout" 'any(.projection.details.optimization.rows[]; .pareto_frontier == true and .feasible == true and .vendor == "Kong AI Gateway")' "breakout exposes a governed Pareto frontier"

run_json_with_file "$competition_breakout" "$COMPETITION_VENDORS_FILE" --mode=pareto-breakout --min-score=75 --max-risk=30
assert_jq "$competition_breakout" '.projection.details.shortlist.shortlist[0].vendor_name == "Gemma 4 31B (Google)"' "competition data ranks Gemma first"
assert_jq "$competition_breakout" '(.projection.details.context.strategies[] | select(.id == "strategy:vendor-sel:router-hypothesis") | .content.router_fit) == true' "competition data triggers router hypothesis"

run_json_with_file "$strategy_candidates" "$STRATEGY_CANDIDATES_FILE" --mode=governed --min-score=75 --max-risk=30
assert_jq "$strategy_candidates" '.projection.details.shortlist.shortlist[0].vendor_name == "Governed Multi-Model Router Strategy (Gemma + Mistral + Arcee)"' "strategy candidates rank governed router mix first"
assert_jq "$strategy_candidates" '.projection.details.policy.selected_vendor == "Governed Multi-Model Router Strategy (Gemma + Mistral + Arcee)"' "strategy candidates promote governed router mix"

"${DEMO_BIN[@]}" \
  --business \
  --vendors-json="$VENDORS_FILE" \
  --experience-path="$EXPERIENCE_PATH" \
  --mode=governed > "$business"

grep -q "Decision: Mistral selected with composite 79.2." "$business" \
  && pass "business output exposes the Mistral decision" \
  || fail "business output exposes the Mistral decision"

grep -q "Qwen (Alibaba Cloud): non-compliant (pending)/risk 35 above maximum 30" "$business" \
  && pass "business output exposes Qwen rejection reasons" \
  || fail "business output exposes Qwen rejection reasons"

printf '\nDemo business story verified.\n'
