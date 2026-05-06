---
tags: [development]
---
# Getting Started

## Prerequisites

**Minimum:** Git, 8GB RAM, ~10 minutes of time.

**Everything else installs automatically** with `just setup`.

No need to pre-install Rust, Cargo, Bun, or tauri-cli — the setup recipe will verify and install what you need.

## First-Time Setup (8 minutes)

⏱️ **Time estimate: ~8 minutes from `git clone` to first governance decision**

### Step 1: Clone and configure (1 minute)

```bash
git clone <repo>
cd hackathon
cp .env.example .env
```

At minimum, `.env` needs one of:
- **Kong gateway** (at hackathon): `KONG_API_KEY=<your-team-key>`
- **Direct provider**: `ANTHROPIC_API_KEY=sk-...` or `OPENAI_API_KEY=sk-...`

### Step 2: Toolchain and dependencies (4 minutes)

```bash
just setup
```

**Expected output:**
```
==> Checking toolchain
rustc 1.94.0 (stable)
cargo 1.94.0
==> Building workspace
   Compiling governance-kernel v0.1.0
   Compiling governance-truths v0.1.0
   Compiling governance-server v0.1.0
   ... (Rust compilation, ~3-4 min first time)
==> Running tests
test governance_kernel::tests::test_vendor_evaluation ... ok
test governance_truths::tests::test_cedar_policy ... ok
==> Lint check
Finished release profile with 0 warnings
==> Ready. Run 'just dev' to start the local harness.
```

**Breakdown:** Rust compilation (3–4 min) is the longest part. Subsequent builds are much faster (caching).

### Step 3: Populate demo data (1 minute)

Open a **new terminal window** and start the server:

```bash
just server
```

**Expected output:**
```
listening on 127.0.0.1:8080
```

Then, in your **original terminal**, seed the vendor database:

```bash
just seed
```

**Expected output:**
```
Seeding vendor-selection truth with 5 vendors...
{
  "recommendations": [
    {
      "vendor": "Acme AI",
      "score": 85.0,
      "risk_score": 15.0,
      "decision": "shortlist"
    },
    {
      "vendor": "Epsilon AI",
      "score": 88.0,
      "risk_score": 20.0,
      "decision": "shortlist"
    },
    ...
  ],
  "explanation": "..."
}
```

### Step 4: Start the desktop app (1 minute)

From a **third terminal**, start the desktop app:

```bash
just dev
```

**Expected output:**
```
Server started (PID: 12345)
Desktop app launching...
```

A Tauri window opens with the governance dashboard. You'll see:
- A vendor selection scenario
- The governance decision flow (proposal → policy check → convergence)
- Recommendation explanation

**You've got it.** You're running a full governed AI system.

## First Governance Decision

After `just seed` runs, you'll see a recommendation like:

```
Vendor Shortlist (Governed)

Recommended: Acme AI, Epsilon AI
Explanation:
  - Score: 85–88 (all above 75 threshold)
  - Risk: 15–20 (all below 30 threshold)
  - Policy: Cedar approval granted (SOC2 + GDPR certified)
  
Excluded: Gamma LLM (risk too high: 35 > 30)

Time: 2s | Policy cycles: 2 | Agent proposals: 5
```

This is the output of:
1. **Vendor screening agent** proposes all vendors
2. **Cedar policy gate** filters by compliance
3. **Criteria evaluator** ranks by score/risk
4. **Convergence** settles on shortlist

This is the governance story. Participants modify the Cedar policy, adjust scoring weights, or add new agents — and see how it changes recommendations.

## Expected Output for Key Commands

### `just setup`

Green checkmarks, no errors. Final line: `==> Ready.`

If you see warnings (from Clippy), that's fine. Errors mean something needs fixing — scroll to Troubleshooting.

### `just server`

```
2026-04-28T10:23:45Z [info] listening on 127.0.0.1:8080
2026-04-28T10:23:45Z [info] governance runtime initialized
```

Server is ready when you see `listening on 127.0.0.1:8080`.

### `just seed`

Shows vendor names and a governance recommendation. If you see an error like `connection refused`, the server is not running — go back and check Step 3.

### `just dev`

Desktop app window opens. If you see a blank screen or "connecting...", wait 3 seconds. If it stays blank, check [[Troubleshooting]] → "Failed to connect to server on localhost:8080".

## Troubleshooting

### Port 8080 already in use

```bash
# Kill the process using port 8080
lsof -i :8080
kill -9 <PID>

# Then try again
just server
```

### Desktop app shows "No available providers"

Your `.env` is not configured. Fix it:

```bash
source .env
echo $KONG_API_KEY    # Should NOT be blank

# If blank, add your Kong API key or provider key to .env, then:
just server           # Restart server with new .env
```

### "Connection refused" in desktop app

The server is not running. Open a **new terminal** and run:

```bash
just server
```

Wait for `listening on 127.0.0.1:8080`, then refresh the desktop app (Cmd+R or F5).

### "Error: ...Criterion unmet: all-vendors-screened"

A governance agent failed silently. This is rare. Check:

```bash
# Read the audit trail
curl http://localhost:8080/v1/audit | jq '.'

# Look for "AgentEffect" entries with empty facts or errors
```

If you're stuck, see [[Troubleshooting]] for the full FAQ.

### Rust compilation fails

Run `rustup update`, then `just clean && just setup`:

```bash
rustup update
just clean
just setup
```

For other issues, see [[Troubleshooting]] → "Getting Started".

## Quick Reference

| Command | What it does |
|---------|-------------|
| `just setup` | Verify toolchain, build workspace, run tests, lint |
| `just server` | Start HTTP API on port 8080 |
| `just seed` | Populate demo vendors and run vendor-selection governance |
| `just dev` | Start server + desktop app together |
| `just dev-desktop` | Start desktop app only (server must be running) |
| `just test` | Run all tests |
| `just lint` | Run clippy |
| `just clean` | Delete all build artifacts |

## API Reference

### Endpoints

| Endpoint | Purpose |
|---|---|
| `GET /health` | Health check |
| `GET /v1/truths` | List available truths |
| `POST /v1/truths/{key}/execute` | Execute a truth with given inputs |
| `GET /v1/audit` | Query governance audit trail |

### Example: Manual vendor evaluation

```bash
# Make sure server is running (just server)

curl -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": {
      "vendors_json": "[{\"name\":\"MyVendor\",\"score\":85,\"risk_score\":20}]",
      "min_score": "75",
      "max_risk": "30"
    }
  }' | jq .
```

## Project Structure

```
kb/                    Obsidian knowledgebase — THE documentation
.claude/skills/        Claude Code slash commands
scripts/workflow/      Shared workflow helpers
apps/desktop/          Svelte + Tauri desktop app
crates/
  governance-kernel/   Domain model + in-memory store
  governance-truths/   Truth catalog + Converge bindings
  governance-server/   HTTP API + truth runtime
  governance-app/      Shared app layer
examples/              Example input files (seed vendors, policies, etc.)
```

## Next Steps

1. **Understand the vendor selection challenge:** Read [[Domain/Vendor Selection]]

2. **Explore the reference implementation:** Study `crates/governance-server/src/truth_runtime/vendor_selection.rs` to see how a truth executes.

3. **Build your first truth:** Follow [[Development/Build Your First Truth]] tutorial. Participants write a new governance rule and see it in action.

4. **Learn about Cedar policies:** Read [[Development/Cedar Policies for Participants]]. Modify `examples/vendor-selection/policies/` and watch recommendations change.

5. **Study API surfaces:** Read [[Development/Programming API Surfaces]] before copying patterns from the desktop app or other examples.

6. **Daily workflows:** See [[Workflow/Daily Journey]] for the cheat sheet.

7. **Advanced:** [[Architecture/Layers]], [[Development/Writing Suggestors]], [[Architecture/Convergence Loop]]

See also: [[Troubleshooting]] for the FAQ and gotchas.
