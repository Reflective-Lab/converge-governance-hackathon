---
tags: [development, troubleshooting, faq]
provenance: llm
---
# Troubleshooting & FAQ

This guide covers common issues you may encounter during setup, development, and demo runs of the governance hackathon. Most problems fall into one of a few categories — find your error below and follow the fix.

## How to Use This Guide

1. **Read the error message** — focus on the first line and any mention of a file or key term
2. **Search this page** — Ctrl+F or Cmd+F for a keyword (e.g., "Clippy", "Cedar", "provider")
3. **Jump to the category** — if you know it's a setup issue, scroll to "Getting Started"
4. **Follow the fix exactly** — copy commands as shown, including paths
5. **Try the prevention tip** — it will save you next time

If you're stuck after trying a fix, check **[Where to Find Help](#where-to-find-help)** at the end.

## How to Read Error Messages

Rust errors are verbose. Here's what to focus on:

- **First line:** The most specific error (e.g., `error[E0514]: found crate X compiled with incompatible version`)
- **File and line:** Tells you where the problem is (`src/lib.rs:42:15`)
- **The "error message" part after the colon:** The actual problem (ignore surrounding context)
- **Caused by:** If present, scroll down to the "caused by" section — that's often the root cause

Ignore:
- `warning:` messages (they won't stop the build)
- Stack traces 50+ lines long (just read the top)
- Compiler suggestions you don't understand (ask in Discord)

---

## Category: Getting Started

### Q: "error[E0514]: found crate X compiled with incompatible version of crate Y"

**Cause:** Your Rust version is older than 1.94, or your dependencies were built with a different compiler. Rust 1.94+ is required (see Cargo.toml).

**Fix:**
```bash
rustup update                  # Update Rust to latest stable
rustup show                    # Verify version is 1.94+
cargo clean                    # Remove cached builds
just hit-the-ground-running    # Rebuild everything
```

**Prevention:** Run `rustup update` every Monday, or after any long break. Check `rustup show` before starting.

**Related:** [[Getting Started]] — Prerequisites section

---

### Q: "error: failed to find tool 'cargo-clippy' — is 'clippy' installed?"

**Cause:** The clippy linter is not installed. This usually means rustup is out of date or the component wasn't installed.

**Fix:**
```bash
rustup component add clippy    # Install clippy for current toolchain
just lint                      # Verify it works
```

If that doesn't work:
```bash
rustup toolchain uninstall stable
rustup toolchain install stable
rustup component add clippy
```

**Prevention:** After `rustup update`, always run `rustup component add clippy` — it's not installed by default.

**Related:** [[Getting Started]] — Prerequisites

---

### Q: "thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: VarError(NotPresent)' — LLM_API_KEY not set" or ".env file not found"

**Cause:** The `.env` file is missing or an environment variable required by Converge is not set. The hackathon requires Kong gateway credentials or direct LLM API keys.

**Fix:**

1. Copy the template:
   ```bash
   cp .env.example .env
   ```

2. Open `.env` and fill in at least one of:
   - **Kong gateway (recommended):** Fill in `KONG_AI_GATEWAY_URL`, `KONG_API_KEY`, `KONG_LLM_ROUTE`
   - **Direct provider:** Uncomment `CONVERGE_LLM_FORCE_PROVIDER` and add your API key (e.g., `ANTHROPIC_API_KEY`)

3. Verify:
   ```bash
   source .env
   echo $KONG_API_KEY    # Should print your key, not blank
   just server           # Should start without panic
   ```

**Prevention:** Never commit `.env` — it's in `.gitignore`. Always `cp .env.example .env` after cloning.

**Related:** `.env.example` in repo root

---

### Q: Linux only — "linker error: linker `cc` not found" or "webkit2gtk-4.1-dev not found"

**Cause:** Tauri desktop app requires GTK development libraries on Linux. Missing development headers prevent linking.

**Fix (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

**Fix (Fedora/RHEL):**
```bash
sudo dnf install webkit2gtk3-devel \
  gcc \
  openssl-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

Then:
```bash
just install-desktop    # Install desktop frontend
just dev-desktop        # Run desktop app
```

**Prevention:** If you're on Linux, install dev headers before `just hit-the-ground-running`. Check your distro's Tauri documentation.

**Related:** [Tauri docs — Linux setup](https://tauri.app/develop/guides/getting-started/setup/linux/)

---

### Q: "Failed to connect to server on localhost:8080" in desktop app, or "connection refused"

**Cause:** The governance server is not running. The desktop app expects the HTTP API on `127.0.0.1:8080`.

**Fix:**

1. Open a **new terminal window** (keep the app running in the first):
   ```bash
   just server
   ```

2. Wait for output like:
   ```
   listening on 127.0.0.1:8080
   ```

3. In the desktop app, refresh (Cmd+R or F5). If you see the provider setup screen, the connection works.

**Prevention:** Always start the server *before* or *alongside* the desktop app. Use two terminal windows: one for `just server`, one for `just dev-desktop`.

**Related:** [[Getting Started]] — Local Harness API

---

## Category: Runtime Errors

### Q: "criterion unmet: all-vendors-screened" or "criterion unmet: X" after running a truth

**Cause:** A required fact was not generated. Truths define criteria that must be satisfied before returning. If a suggestor failed silently, or its name doesn't match the dispatcher, the fact never gets created.

**Fix:**

1. Check the audit trail:
   ```bash
   curl http://localhost:8080/v1/audit | jq '.' | tail -50
   ```
   Look for error messages or missing `Fact` entries.

2. Verify the truth definition matches the dispatcher:
   ```bash
   grep -r "all-vendors-screened" crates/
   ```
   Check that the criterion name matches what the suggestor emits.

3. Check suggestor logs:
   ```bash
   RUST_LOG=debug just server
   ```
   Run the truth again and look for `Suggestor` or `Effect` messages.

4. If a suggestor is not running, check its `accepts()` method — if it returns `false`, the dispatcher skips it.

**Prevention:** Write the criterion name and what fact creates it in a comment above the criterion definition. Example:
```rust
("all-vendors-screened", "Every vendor has a screening_result Fact")
```

**Related:** [[Domain/Truths]], [[Development/Writing Truths]]

---

### Q: "Cedar policy parse error: line X, column Y: unexpected token" in policy file

**Cause:** Cedar policy syntax is strict. Common mistakes: missing semicolon, wrong operator (e.g., `==` instead of `=`), undefined entity type, or quoted string in wrong place.

**Fix:**

1. Go to line X, column Y in the file (editor usually jumps there on error)

2. Check for common mistakes:
   - **Missing semicolon** at end of permit/forbid statement
   - **Wrong operator:** Cedar uses `==` for comparison, `=` in is-expressions
   - **Undefined entity:** Entity types (like `User`, `Vendor`) must be defined in schema
   - **String quoting:** Use `"exact-string"` for names, `'single'` for IDs

3. Validate the policy file:
   ```bash
   # No direct validator, but you can test by running a truth that uses it
   RUST_LOG=debug just server
   # Then call the truth endpoint
   curl -X POST http://localhost:8080/v1/truths/authorize-vendor/execute \
     -H "Content-Type: application/json" \
     -d '{"vendor_id": "acme"}'
   ```

4. Check error logs for the exact parse failure location.

**Prevention:** Use the Cedar playground at [cedarpolicy.com](https://cedarpolicy.com/playground) to test policy syntax before committing. Start simple (one permit statement) and build up.

**Related:** Cedar policy docs at [cedarpolicy.com/docs](https://cedarpolicy.com/docs)

---

### Q: "Agent returned no proposals" or "No agents produced an AgentEffect" after executing a truth

**Cause:** A suggestor ran but produced an empty `AgentEffect::default()` instead of facts. This usually means the LLM call succeeded but returned a useless response, or the parsing logic discarded all results.

**Fix:**

1. Check the raw LLM response:
   ```bash
   RUST_LOG=converge_pack=debug just server
   ```
   Run the truth again and search logs for `LLM response:` or `parsed:`.

2. Inspect the suggestor's parsing logic:
   - Find the suggestor file (e.g., `governance-server/src/truth_runtime/evaluate_vendor.rs`)
   - Look for the line that parses LLM output into facts
   - If it's a regex or JSON parse, test it locally with a sample response

3. Verify the LLM provider is working:
   ```bash
   curl -X POST https://<kong-url>/v1/chat/completions \
     -H "Authorization: Bearer $KONG_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{
       "model": "gpt-4",
       "messages": [{"role": "user", "content": "Say hello"}]
     }'
   ```

4. If the LLM returns empty or malformed, check:
   - Provider keys in `.env` are correct
   - Kong gateway is accessible (if using Kong)
   - Rate limits or quotas are not exceeded

**Prevention:** Add debug assertions in parsing:
```rust
let parsed_facts = parse_response(&response);
assert!(!parsed_facts.is_empty(), "parsed zero facts from response: {}", response);
```

**Related:** [[Development/Writing Suggestors]]

---

### Q: "10 cycles used, criteria unmet" or "budget exhausted, criterion X not met"

**Cause:** The suggestor loop ran the maximum number of times but still didn't satisfy all criteria. This indicates either the criteria are too strict, the LLM is not converging on a solution, or the criterion evaluator has a bug.

**Fix:**

1. Increase the budget:
   ```rust
   // In truth_runtime/your_truth.rs, find the Budget creation:
   let budget = Budget::new(20, Duration::from_secs(120)); // Increase from 10 to 20
   ```

2. Relax the criterion if it's impossible:
   ```rust
   // If criterion is "all-vendors-risk-scored" but risk scoring is optional,
   // change to "risk-scores-for-available-data"
   ```

3. Check the criterion evaluator logic:
   ```bash
   # Find where the criterion is checked:
   grep -n "all-vendors-risk-scored" crates/governance-truths/src/criteria.rs
   
   # Verify its logic — it should return true when satisfied
   # Test it with sample data:
   cargo test --lib criteria --
   ```

4. Log what's missing:
   ```bash
   RUST_LOG=governance_truths=debug just server
   ```
   Run the truth and search logs for criterion check output.

**Prevention:** Start with a budget of 5 cycles and increase only if needed. Write criteria that are achievable (e.g., "at least one result" instead of "all results").

**Related:** [[Architecture/Convergence Loop]], [[Development/Writing Truths]]

---

### Q: "No available providers" on Provider Setup screen or "all providers exhausted"

**Cause:** Converge tried all configured LLM providers and they all failed. Usually means API keys are missing, incorrect, or the provider is offline.

**Fix:**

1. Verify `.env`:
   ```bash
   source .env
   echo "Kong: $KONG_API_KEY"
   echo "Anthropic: $ANTHROPIC_API_KEY"
   echo "OpenAI: $OPENAI_API_KEY"
   ```
   At least one should be non-empty.

2. Test Kong connectivity:
   ```bash
   curl -X GET "$KONG_AI_GATEWAY_URL/health" \
     -H "Authorization: Bearer $KONG_API_KEY"
   ```
   Should return HTTP 200.

3. Test direct provider (if using):
   ```bash
   curl -X POST https://api.openai.com/v1/chat/completions \
     -H "Authorization: Bearer $OPENAI_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "hi"}]}'
   ```
   Should return a response, not 401 or 429.

4. Check provider order in Converge config:
   - Kong is tried first, then direct providers in order: anthropic, openai, gemini, etc.
   - If Kong fails, the fallback providers are used
   - Ensure at least one is configured correctly

5. Review server logs:
   ```bash
   RUST_LOG=converge_provider=debug just server
   ```
   Look for "Provider X failed" messages.

**Prevention:** Test one provider end-to-end before adding others. Start with Kong since it's configured at the hackathon. If using direct providers, test curl commands first before running the app.

**Related:** `.env.example`, [[Integrations/Kong Gateway]]

---

## Category: Development / Modification

### Q: "ContextKey 'my-key' not found" or "dispatcher doesn't recognize truth key"

**Cause:** You registered a truth in the catalog but forgot to register its suggestor with the dispatcher, or you used a different name in each place.

**Fix:**

1. Find where you defined the truth:
   ```bash
   grep -n "my-truth" crates/governance-truths/src/lib.rs
   ```

2. Copy the exact key (case-sensitive):
   ```rust
   TruthDef {
       key: "my-truth",  // <- Use this exact string
       ...
   }
   ```

3. Register the executor in `governance-server/src/main.rs`:
   ```rust
   dispatcher.register_truth(
       "my-truth",  // <- Must match exactly
       Box::new(MyTruthExecutor),
   );
   ```

4. Verify both names match exactly (including hyphens and underscores):
   ```bash
   grep -r '"my-truth"' crates/
   ```
   All should match.

**Prevention:** Define the truth key as a constant:
```rust
const MY_TRUTH_KEY: &str = "my-truth";

// Then use it in both places:
TruthDef { key: MY_TRUTH_KEY, ... }
dispatcher.register_truth(MY_TRUTH_KEY, ...);
```

**Related:** [[Development/Writing Truths]]

---

### Q: "Suggestor never runs" or "suggestor's accept() always returns false"

**Cause:** The dispatcher checks `Suggestor::accepts()` before running. If it returns `false`, the suggestor is skipped, so its facts never appear.

**Fix:**

1. Find the suggestor implementation:
   ```bash
   grep -r "impl Suggestor for.*YourSuggestor" crates/
   ```

2. Check the `accepts()` method:
   ```rust
   impl Suggestor for YourSuggestor {
       fn accepts(&self, state: &State) -> bool {
           // This should return true when you WANT to run
           state.has_fact("some-prerequisite-fact")
       }
   }
   ```

3. If it's too restrictive, verify the prerequisite fact exists:
   ```bash
   curl http://localhost:8080/v1/audit | jq '.[] | select(.fact_type == "some-prerequisite-fact")'
   ```
   Should return at least one result.

4. If the prerequisite doesn't exist, either:
   - Add a suggestor that creates it first, or
   - Remove the prerequisite check from `accepts()`

5. Test the suggestor in isolation:
   ```bash
   cargo test --lib your_truth_tests -- --nocapture
   ```
   Look for your suggestor name in output.

**Prevention:** Start with `accepts()` returning `true` always, then add guards only when necessary. Comment why each guard exists.

**Related:** [[Development/Writing Suggestors]]

---

### Q: "test assertion mismatch: expected X, found Y" in unit test

**Cause:** Your test expects different output than what the code produces. Common causes: hardcoded expected value is wrong, the code changed but the test didn't, or the assertion compares the wrong thing.

**Fix:**

1. Find the failing test:
   ```bash
   just test 2>&1 | grep -A 10 "assertion mismatch"
   ```

2. Open the test file and look at the line number from the error:
   ```bash
   # Error mentions "src/truth_runtime/my_truth.rs:42"
   vim crates/governance-server/src/truth_runtime/my_truth.rs +42
   ```

3. Read the assertion:
   ```rust
   assert_eq!(actual, expected);  // What's actual? What's expected?
   ```

4. Decide: is the expected value wrong, or is the code wrong?
   - If the expected value is outdated, update it:
     ```rust
     assert_eq!(actual, "new-expected-value");
     ```
   - If the code is wrong, fix the code, not the test

5. Re-run the test:
   ```bash
   just test --lib my_truth
   ```

**Prevention:** Use descriptive test names and assertion messages:
```rust
#[test]
fn test_vendor_screening_marks_high_risk_vendors() {
    let result = screen_vendor(&vendor);
    assert!(
        result.risk_level >= RiskLevel::High,
        "vendor should be high-risk, got {:?}",
        result
    );
}
```

**Related:** [[Contributing.md]] — Testing section

---

## Category: Compilation Issues

### Q: "error[E0433]: cannot find type X in this scope"

**Cause:** A type is used but not imported. Common causes: forgot to add `use` statement, type is private, or typo in the type name.

**Fix:**

1. Find the missing type (error mentions it):
   ```
   error[E0433]: cannot find type `VendorScreening` in this scope
   ```

2. Check if it's imported at the top of the file:
   ```rust
   use governance_kernel::facts::VendorScreening;  // Is this present?
   ```

3. If not, add it. Find where the type is defined:
   ```bash
   grep -r "struct VendorScreening" crates/
   ```

4. Add the import:
   ```rust
   use governance_kernel::facts::VendorScreening;
   ```

5. If the type is in the same module, just use it directly (no import needed).

6. Check for typos:
   ```rust
   // Wrong:
   let screening: VendorScreeningg = ...;  // Extra 'g'
   
   // Right:
   let screening: VendorScreening = ...;
   ```

**Prevention:** Use IDE autocomplete (Rust Analyzer in VS Code, rust-analyzer in Vim/Neovim). Type `VendorScreen` and it will suggest the import.

**Related:** [[Development/Programming API Surfaces]]

---

### Q: "error: attribute macro converge_pack not found" or "macro `pack_suggestor!` not found"

**Cause:** A procedural macro from Converge is not available. Usually means the crate dependency is missing or the feature is not enabled.

**Fix:**

1. Check the workspace dependencies in `Cargo.toml`:
   ```bash
   grep "converge-pack" Cargo.toml
   ```
   Should show:
   ```toml
   [workspace.dependencies]
   converge-pack = "3.8.1"
   ```

2. In your crate's `Cargo.toml`, add the dependency:
   ```toml
   [dependencies]
   converge-pack = { workspace = true }
   ```

3. Rebuild:
   ```bash
   cargo clean
   just build
   ```

4. If it still fails, check the macro is exported:
   ```bash
   grep -r "pub use.*pack_suggestor" ~/.cargo/registry/src/
   ```
   Or use `cargo search converge-pack` to verify the version.

5. If using a custom macro, make sure it's defined in the right crate:
   ```bash
   grep -n "#\[proc_macro" crates/*/src/lib.rs
   ```

**Prevention:** Always use `workspace = true` for Converge deps in Cargo.toml. Check `Cargo.lock` after updating dependencies to ensure they're the right version.

**Related:** [[Converge/Crate Catalog]]

---

## If All Else Fails

### Nuclear Option: Clean Build

If you've tried everything above and the build is still stuck, do a complete reset:

```bash
# Remove all build artifacts and cached dependencies
cargo clean
rm -rf ~/.cargo/registry ~/.cargo/git
rm -rf target/

# Update Rust
rustup update
rustup component add clippy

# Fresh setup
cp .env.example .env
just hit-the-ground-running
```

This will take 5–10 minutes but will resolve 99% of "impossible" build issues.

---

## Where to Find Help

**Before asking for help:**
1. Run `just check && just test` and capture the full error output
2. Search this page (Ctrl+F) for your error message
3. Read the linked "Related" pages — they may have more context

**Then:**
- **Discord** — #hackathon-support channel, @maintainers
- **GitHub Issues** — If you think you found a bug, create an issue with:
  - OS and Rust version (`rustup show`)
  - Full error output (copy entire error block)
  - What you were trying to do (e.g., "ran `just server`")
- **Email** — kpernyer@gmail.com if you need immediate help (Slack preferred)

**When posting in Discord:**
- Use a thread (don't clutter #general)
- Paste the error (use code block with \`\`\`)
- Say what command triggered it
- Say what you've already tried

---

## Symptoms → Solutions Quick Index

| Symptom | Section | Try this first |
|---------|---------|---|
| Build fails with `E0514` | Rust version mismatch | `rustup update && cargo clean` |
| "Clippy not found" | Clippy not installed | `rustup component add clippy` |
| "LLM_API_KEY not set" | Missing .env | `cp .env.example .env` |
| Desktop app can't connect | Server not running | `just server` in another terminal |
| "Criterion unmet" | Suggestor failed silently | Check audit trail, verify names match |
| Cedar parse error | Policy syntax wrong | Use Cedar playground to validate |
| "No proposals returned" | LLM call failed | Check provider keys, test Kong connectivity |
| Budget exhausted | Criteria impossible | Increase budget, relax criteria |
| No available providers | API keys missing | Source .env, test provider endpoints |
| "Key not found" | Name mismatch | Grep for the key, verify exact spelling |
| Suggestor never runs | accepts() returns false | Check prerequisite facts exist |
| Test fails | Expected value wrong | Update test or fix code |
| "Type not in scope" | Import missing | Add `use` statement for the type |
| "Macro not found" | Dependency missing | Add to Cargo.toml with `workspace = true` |
