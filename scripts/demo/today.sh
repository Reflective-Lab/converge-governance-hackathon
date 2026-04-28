#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export DEMO_COMPLIANCE_MODEL="${DEMO_COMPLIANCE_MODEL:-mistralai/mistral-small-2603}"
export DEMO_COST_MODEL="${DEMO_COST_MODEL:-arcee-ai/trinity-large-preview}"
export DEMO_RISK_MODEL="${DEMO_RISK_MODEL:-mistralai/mistral-small-2603}"
export DEMO_SYNTHESIS_MODEL="${DEMO_SYNTHESIS_MODEL:-writer/palmyra-x5}"

exec "$SCRIPT_DIR/presentation.sh" --today "$@"
