use std::collections::HashMap;
use std::path::PathBuf;

use governance_kernel::InMemoryStore;
use governance_server::experience::ExperienceRegistry;
use governance_server::llm_helpers::load_env;
use governance_server::truth_runtime::TruthExecutionResult;
use governance_server::truth_runtime::vendor_selection::{self, ModelOverrides};
use serde_json::Value;

const DEFAULT_VENDORS_JSON: &str =
    include_str!("../../../../examples/vendor-selection/seed-vendors.json");
const DEFAULT_EXPERIENCE_PATH: &str = "data/headless_vendor_selection_experience_store.json";

#[derive(Debug)]
struct CliOptions {
    live: bool,
    persist: bool,
    json: bool,
    business: bool,
    min_score: String,
    max_risk: String,
    max_vendors: String,
    principal_authority: String,
    demo_mode: String,
    human_approval_present: Option<String>,
    vendors_json_path: Option<PathBuf>,
    source_doc_path: Option<PathBuf>,
    static_facts_paths: Vec<PathBuf>,
    experience_path: PathBuf,
    model_overrides: ModelOverrides,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            live: false,
            persist: false,
            json: false,
            business: false,
            min_score: "75".to_string(),
            max_risk: "30".to_string(),
            max_vendors: "3".to_string(),
            principal_authority: "supervisory".to_string(),
            demo_mode: "governed".to_string(),
            human_approval_present: None,
            vendors_json_path: None,
            source_doc_path: None,
            static_facts_paths: Vec::new(),
            experience_path: PathBuf::from(DEFAULT_EXPERIENCE_PATH),
            model_overrides: ModelOverrides::default(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = parse_args()?;

    if options.live {
        load_env();
    }

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("GOVERNANCE_DEMO_LOG").unwrap_or_else(|_| "off".into()))
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();

    let vendors_json = load_vendors_json(&options)?;
    let mut inputs = HashMap::from([
        ("vendors_json".to_string(), vendors_json),
        ("min_score".to_string(), options.min_score.clone()),
        ("max_risk".to_string(), options.max_risk.clone()),
        ("max_vendors".to_string(), options.max_vendors.clone()),
        (
            "principal_authority".to_string(),
            options.principal_authority.clone(),
        ),
        ("demo_mode".to_string(), options.demo_mode.clone()),
    ]);
    load_source_material(&mut inputs, &options)?;
    if options.live {
        inputs.insert("live_mode".to_string(), "true".to_string());
    }
    if let Some(value) = &options.human_approval_present {
        inputs.insert("human_approval_present".to_string(), value.clone());
    }

    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::with_path(&options.experience_path);
    let result = if options.model_overrides.is_empty() {
        vendor_selection::execute_with_experience(
            &store,
            &inputs,
            options.persist,
            Some(&experience),
        )
        .await
    } else {
        vendor_selection::execute_with_model_overrides(
            &store,
            &inputs,
            options.persist,
            Some(&experience),
            &options.model_overrides,
        )
        .await
    }
    .map_err(anyhow::Error::msg)?;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if options.business {
        print_business_demo(&result, &options);
    } else {
        print_demo(&result, &options);
    }

    Ok(())
}

fn parse_args() -> anyhow::Result<CliOptions> {
    let mut options = CliOptions::default();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--live" => options.live = true,
            "--persist" => options.persist = true,
            "--json" => options.json = true,
            "--business" => options.business = true,
            "--human-approval" => options.human_approval_present = Some("true".to_string()),
            "--no-human-approval" => options.human_approval_present = Some("false".to_string()),
            "--governed" => options.demo_mode = "governed".to_string(),
            "--pareto-breakout" | "--creative" => {
                options.demo_mode = "pareto-breakout".to_string();
            }
            _ if arg.starts_with("--min-score=") => {
                options.min_score = value_after_equals(&arg).to_string();
            }
            _ if arg.starts_with("--max-risk=") => {
                options.max_risk = value_after_equals(&arg).to_string();
            }
            _ if arg.starts_with("--max-vendors=") => {
                options.max_vendors = value_after_equals(&arg).to_string();
            }
            _ if arg.starts_with("--authority=") => {
                options.principal_authority = value_after_equals(&arg).to_string();
            }
            _ if arg.starts_with("--mode=") => {
                options.demo_mode = normalize_demo_mode(value_after_equals(&arg))?;
            }
            _ if arg.starts_with("--vendors-json=") => {
                options.vendors_json_path = Some(PathBuf::from(value_after_equals(&arg)));
            }
            _ if arg.starts_with("--source-doc=")
                || arg.starts_with("--doc=")
                || arg.starts_with("--document=") =>
            {
                options.source_doc_path = Some(PathBuf::from(value_after_equals(&arg)));
            }
            _ if arg.starts_with("--static-facts=") => {
                options
                    .static_facts_paths
                    .push(PathBuf::from(value_after_equals(&arg)));
            }
            _ if arg.starts_with("--experience-path=") => {
                options.experience_path = PathBuf::from(value_after_equals(&arg));
            }
            _ if arg.starts_with("--model-compliance=") => {
                options.model_overrides.compliance = Some(value_after_equals(&arg).to_string());
            }
            _ if arg.starts_with("--model-cost=") => {
                options.model_overrides.cost = Some(value_after_equals(&arg).to_string());
            }
            _ if arg.starts_with("--model-risk=") => {
                options.model_overrides.risk = Some(value_after_equals(&arg).to_string());
            }
            _ if arg.starts_with("--model-synthesis=") => {
                options.model_overrides.synthesis = Some(value_after_equals(&arg).to_string());
            }
            _ => {
                anyhow::bail!("unknown argument: {arg}. Run with --help for usage.");
            }
        }
    }

    Ok(options)
}

fn normalize_demo_mode(value: &str) -> anyhow::Result<String> {
    match value {
        "governed" | "governed-selection" | "selection" => Ok("governed".to_string()),
        "pareto-breakout" | "creative" | "open" => Ok("pareto-breakout".to_string()),
        _ => anyhow::bail!("invalid --mode={value}. Expected governed or pareto-breakout."),
    }
}

fn value_after_equals(arg: &str) -> &str {
    arg.split_once('=')
        .map(|(_, value)| value)
        .unwrap_or_default()
}

fn load_vendors_json(options: &CliOptions) -> anyhow::Result<String> {
    match &options.vendors_json_path {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display())),
        None => Ok(DEFAULT_VENDORS_JSON.to_string()),
    }
}

fn load_source_material(
    inputs: &mut HashMap<String, String>,
    options: &CliOptions,
) -> anyhow::Result<()> {
    if let Some(path) = &options.source_doc_path {
        let source_document = std::fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
        inputs.insert(
            "source_document_path".to_string(),
            path.display().to_string(),
        );
        inputs.insert("source_document".to_string(), source_document);
    }

    if options.static_facts_paths.is_empty() {
        return Ok(());
    }

    let mut facts = Vec::new();
    let mut paths = Vec::new();
    for path in &options.static_facts_paths {
        let raw = std::fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
        let content = serde_json::from_str::<Value>(&raw).unwrap_or(Value::String(raw));
        paths.push(path.display().to_string());
        facts.push(serde_json::json!({
            "path": path.display().to_string(),
            "content": content,
        }));
    }

    inputs.insert(
        "static_facts_paths_json".to_string(),
        serde_json::to_string(&paths)?,
    );
    inputs.insert(
        "static_facts_json".to_string(),
        serde_json::to_string(&facts)?,
    );
    Ok(())
}

fn print_help() {
    println!(
        r#"Headless vendor-selection demo

Usage:
  cargo run -p governance-server --bin vendor-selection-demo -- [options]

Options:
  --live                         Use live Brave/Tavily/LLM-backed suggestors
  --persist                      Persist the projected decision into the in-memory store
  --json                         Print raw TruthExecutionResult JSON
  --business                     Print concise business-facing demo output
  --vendors-json=PATH            Vendor input JSON file
  --source-doc=PATH              Buyer/source document used for the run
  --doc=PATH                     Alias for --source-doc
  --static-facts=PATH            Static facts JSON/text file; may be repeated
  --min-score=N                  Minimum vendor score for shortlist (default: 75)
  --max-risk=N                   Maximum risk score for shortlist (default: 30)
  --max-vendors=N                Maximum shortlist size (default: 3)
  --authority=LEVEL              advisory | supervisory | participatory | sovereign
  --mode=MODE                    governed | pareto-breakout (default: governed)
  --governed                     Shortcut for --mode=governed
  --pareto-breakout              Shortcut for --mode=pareto-breakout
  --human-approval               Force HITL approval present
  --no-human-approval            Force HITL approval absent
  --model-compliance=MODEL       Live LLM model for compliance screening
  --model-cost=MODEL             Live LLM model for cost analysis
  --model-risk=MODEL             Live LLM model for risk analysis
  --model-synthesis=MODEL        Live LLM model for decision synthesis
  --experience-path=PATH         Persistent prior-run store
"#
    );
}

fn print_business_demo(result: &TruthExecutionResult, options: &CliOptions) {
    let details = result
        .projection
        .as_ref()
        .and_then(|projection| projection.details.as_ref());

    println!("=== Vendor Selection Business Output ===");
    println!(
        "Execution: {} | Track: {}",
        if options.live {
            "live providers"
        } else {
            "offline deterministic"
        },
        options.demo_mode
    );
    println!(
        "Flow status: {} in {} cycles; fixed point {}.",
        if result.converged {
            "converged"
        } else {
            "stopped"
        },
        result.cycles,
        if result.converged {
            "reached"
        } else {
            "not reached"
        }
    );

    let Some(details) = details else {
        println!("Decision: no business projection was produced.");
        print_business_provider_telemetry(result);
        return;
    };

    print_business_intake(details);
    print_business_decision(details);
    print_business_shortlist(details);
    print_business_policy(details);
    print_business_learning(details);
    print_business_router(details);
    print_business_stack_pressure(details);
    print_business_provider_telemetry(result);
}

fn print_business_intake(details: &Value) {
    let vendors = details
        .pointer("/root_intent/candidate_vendors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "none".to_string());
    let min_score = number_at(details, "/root_intent/objective/min_score").unwrap_or_default();
    let max_risk = number_at(details, "/root_intent/objective/max_risk").unwrap_or_default();

    println!("Buyer document pack: {vendors}");
    println!(
        "Buyer constraints: score >= {}, risk <= {}, compliance must be clear.",
        format_number(min_score),
        format_number(max_risk)
    );
}

fn print_business_decision(details: &Value) {
    let selected = str_at(details, "/policy/selected_vendor")
        .or_else(|| str_at(details, "/shortlist/shortlist/0/vendor_name"))
        .unwrap_or("no vendor");
    let top = details.pointer("/shortlist/shortlist/0");
    let composite = top
        .and_then(|value| number_at(value, "/composite_score"))
        .unwrap_or_default();
    let capability = top
        .and_then(|value| number_at(value, "/score"))
        .unwrap_or_default();
    let risk = top
        .and_then(|value| number_at(value, "/risk_score"))
        .unwrap_or_default();
    let cost = top
        .and_then(|value| number_at(value, "/cost_major"))
        .unwrap_or_default();

    println!(
        "Decision: {selected} selected with composite {}.",
        format_one_decimal(composite)
    );
    println!(
        "Why this wins: capability {}, risk {}, cost ${}/month, balanced against certifications.",
        format_one_decimal(capability),
        format_one_decimal(risk),
        format_number(cost)
    );

    if let Some((vendor, score)) = highest_capability_vendor(details)
        && vendor != selected
    {
        println!(
            "Business insight: highest raw capability was {vendor} ({}), but the objective favors the best governed balance.",
            format_one_decimal(score)
        );
    }
}

fn highest_capability_vendor(details: &Value) -> Option<(&str, f64)> {
    details
        .pointer("/optimization/rows")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(|row| Some((str_at(row, "/vendor")?, number_at(row, "/score")?)))
        .max_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn print_business_shortlist(details: &Value) {
    println!("Candidate ranking:");
    if let Some(shortlist) = details
        .pointer("/shortlist/shortlist")
        .and_then(Value::as_array)
    {
        for row in shortlist {
            println!(
                "  #{} {}: composite {}, capability {}, risk {}, cost ${}/month",
                format_number(number_at(row, "/rank").unwrap_or_default()),
                str_at(row, "/vendor_name").unwrap_or("vendor"),
                format_one_decimal(number_at(row, "/composite_score").unwrap_or_default()),
                format_one_decimal(number_at(row, "/score").unwrap_or_default()),
                format_one_decimal(number_at(row, "/risk_score").unwrap_or_default()),
                format_number(number_at(row, "/cost_major").unwrap_or_default()),
            );
        }
    }

    println!("Rejected or blocked inputs:");
    if let Some(rejected) = details
        .pointer("/shortlist/rejected")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
    {
        for row in rejected {
            println!(
                "  - {}: {}",
                str_at(row, "/vendor_name").unwrap_or("vendor"),
                Some(array_join(row.pointer("/reasons")))
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "no qualifying reason recorded".to_string())
            );
        }
    } else {
        println!("  - none");
    }
}

fn print_business_policy(details: &Value) {
    println!("Policy gate:");
    let Some(policy) = details.get("policy") else {
        println!("  Outcome: missing");
        return;
    };

    println!(
        "  Outcome: {}",
        str_at(policy, "/outcome").unwrap_or("unknown")
    );
    println!(
        "  Selected vendor: {}",
        str_at(policy, "/selected_vendor").unwrap_or("none")
    );
    println!(
        "  Amount: ${} / threshold ${}",
        format_number(number_at(policy, "/selected_amount_major").unwrap_or_default()),
        format_number(number_at(policy, "/hitl_threshold_major").unwrap_or_default())
    );
    println!(
        "  Human approval present: {}",
        bool_at(policy, "/human_approval_present").unwrap_or(false)
    );
    println!("  Reason: {}", business_policy_reason(policy));
}

fn business_policy_reason(policy: &Value) -> String {
    if let Some(reason) = str_at(policy, "/reason")
        && !reason.trim().is_empty()
    {
        return reason.to_string();
    }

    match str_at(policy, "/outcome").unwrap_or("unknown") {
        "Promote" => "policy permit matched",
        "Escalate" if !bool_at(policy, "/human_approval_present").unwrap_or(false) => {
            "human approval required before commitment"
        }
        "Escalate" => "policy requires escalation before commitment",
        "Reject" if str_at(policy, "/principal_authority") == Some("advisory") => {
            "advisory authority cannot commit"
        }
        "Reject" => "policy rejected the commitment",
        _ => "none recorded",
    }
    .to_string()
}

fn print_business_learning(details: &Value) {
    println!("Learning:");
    let Some(learning) = details.get("learning") else {
        println!("  Prior runs available to this run: 0");
        return;
    };

    println!(
        "  Prior runs available to this run: {}",
        format_number(number_at(learning, "/prior_runs").unwrap_or_default())
    );
    if let Some(consistent) = learning.get("consistent_recommendation") {
        println!(
            "  Consistent recommendation: {} ({}/{})",
            str_at(consistent, "/vendor").unwrap_or("none"),
            format_number(number_at(consistent, "/count").unwrap_or_default()),
            format_number(number_at(consistent, "/total_prior_runs").unwrap_or_default())
        );
    }
}

fn print_business_router(details: &Value) {
    let Some(router) = router_payload(details) else {
        return;
    };

    println!("Router hypothesis:");
    println!("  Fit: {}", bool_at(router, "/router_fit").unwrap_or(false));
    println!(
        "  Demo line: {}",
        str_at(router, "/demo_line").unwrap_or("No router hypothesis produced.")
    );
    println!("  Why: {}", str_at(router, "/why").unwrap_or("not stated"));
    if let Some(mix) = router.pointer("/provider_mix").and_then(Value::as_array) {
        println!("  Provider mix:");
        for route in mix {
            println!(
                "    - {} -> {}",
                str_at(route, "/need").unwrap_or("need"),
                str_at(route, "/route").unwrap_or("route")
            );
        }
    }
}

fn print_business_stack_pressure(details: &Value) {
    let Some(rows) = details
        .get("stack_pressure")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };

    println!("Foundation pressure:");
    for row in rows {
        println!(
            "  - {} {}: {}",
            str_at(row, "/layer").unwrap_or("layer"),
            str_at(row, "/version").unwrap_or(""),
            str_at(row, "/demo_signal").unwrap_or("no signal")
        );
        println!(
            "    Next: {}",
            str_at(row, "/pressure").unwrap_or("no pressure recorded")
        );
    }
}

fn print_business_provider_telemetry(result: &TruthExecutionResult) {
    println!("Provider layer:");
    let Some(calls) = &result.llm_calls else {
        println!("  Live provider calls: none captured in this run.");
        return;
    };
    let fallback_count = calls
        .iter()
        .filter(|call| {
            call.metadata
                .get("fallback")
                .is_some_and(|value| value == "true")
        })
        .count();
    println!(
        "  Live provider calls recorded: {} (fallbacks: {}).",
        calls.len(),
        fallback_count
    );
    for call in calls {
        println!(
            "  - {} via {}/{} {}ms tokens={}",
            call.context,
            call.provider,
            call.model,
            call.elapsed_ms,
            call.usage
                .as_ref()
                .and_then(|usage| usage.total_tokens)
                .map_or_else(|| "-".to_string(), |tokens| tokens.to_string())
        );
    }
}

fn print_demo(result: &TruthExecutionResult, options: &CliOptions) {
    let details = result
        .projection
        .as_ref()
        .and_then(|projection| projection.details.as_ref());

    println!("=== Headless Vendor Selection Flow ===");
    println!(
        "Mode: {} | Persist projection: {} | Experience: {}",
        if options.live {
            "live providers"
        } else {
            "offline deterministic"
        },
        options.persist,
        options.experience_path.display()
    );
    println!(
        "Converged: {} | Cycles: {} | Stop: {}",
        result.converged, result.cycles, result.stop_reason
    );
    println!("Demo track: {}", options.demo_mode);

    print_intake(details);
    print_formation(details);
    print_huddle(details);
    print_steps(details);
    print_consensus(result, details);
    print_provider_telemetry(result);
}

fn print_intake(details: Option<&Value>) {
    section("0. RFI / RFP Intake");

    let Some(details) = details else {
        println!("No normalized intake projection available.");
        return;
    };

    let vendors = details
        .pointer("/root_intent/candidate_vendors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "none".to_string());

    println!("Buyer upload: RFI/RFP document collection");
    println!("Current headless input: normalized vendor JSON from that collection");
    println!("Candidate vendors: {vendors}");

    if let Some(channels) = details
        .pointer("/resources/evidence_channels")
        .and_then(Value::as_array)
    {
        println!("Evidence channels:");
        for channel in channels {
            if let Some(channel) = channel.as_str() {
                println!("  - {channel}");
            }
        }
    }

    println!(
        "Boundary: intake extracts requirements, vendors, constraints, and source artifacts; formation decides which capabilities are needed next."
    );
}

fn print_formation(details: Option<&Value>) {
    section("1. Formation");

    let Some(details) = details else {
        println!("No projection details available.");
        return;
    };

    if let Some(root) = details.get("root_intent") {
        println!(
            "Intent: {}",
            str_at(root, "/statement").unwrap_or("select a governed vendor")
        );
        println!(
            "Outcome: {}",
            str_at(root, "/outcome").unwrap_or("ranked recommendation or honest stop")
        );
        println!(
            "Demo mode: {}",
            str_at(root, "/demo_mode/label").unwrap_or("Governed selection")
        );
        println!(
            "  Thesis: {}",
            str_at(root, "/demo_mode/thesis").unwrap_or("governed convergence")
        );
        println!(
            "  Boundary: {}",
            str_at(root, "/demo_mode/selection_boundary").unwrap_or("policy-gated")
        );
        println!(
            "Authority: {} in {}",
            str_at(root, "/authority/level").unwrap_or("unknown"),
            str_at(root, "/authority/domain").unwrap_or("unknown")
        );
        println!(
            "Constraints: min_score={} max_risk={} max_vendors={}",
            number_at(root, "/objective/min_score").unwrap_or_default(),
            number_at(root, "/objective/max_risk").unwrap_or_default(),
            number_at(root, "/objective/max_vendors").unwrap_or_default()
        );
    }

    if let Some(tactic) = tactic_payload(details) {
        println!(
            "Tactic selected: {}",
            str_at(tactic, "/name").unwrap_or("unknown")
        );
        println!("  Why: {}", str_at(tactic, "/why").unwrap_or("not stated"));
        println!(
            "  Demo line: {}",
            str_at(tactic, "/surprise_line").unwrap_or("Formation selected a tactic.")
        );
    }

    if let Some(router) = router_payload(details) {
        println!(
            "Router hypothesis: {}",
            str_at(router, "/name").unwrap_or("unknown")
        );
        println!(
            "  Router fit: {}",
            bool_at(router, "/router_fit").unwrap_or(false)
        );
        println!("  Why: {}", str_at(router, "/why").unwrap_or("not stated"));
        println!(
            "  Demo line: {}",
            str_at(router, "/demo_line").unwrap_or("No router change proposed.")
        );
        if let Some(mix) = router.pointer("/provider_mix").and_then(Value::as_array) {
            if bool_at(router, "/router_fit").unwrap_or(false) {
                println!("  Proposed provider mix:");
            } else {
                println!("  Lower-layer capability examples:");
            }
            for route in mix {
                println!(
                    "    - {} -> {}",
                    str_at(route, "/need").unwrap_or("need"),
                    str_at(route, "/route").unwrap_or("route")
                );
            }
        }
    }

    println!("Formation declares needs, not providers:");
    for need in [
        "compliance evidence",
        "price/cost evaluation",
        "risk scoring",
        "shortlist optimization",
        "decision synthesis",
        "policy authorization",
    ] {
        println!("  - need: {need}");
    }

    if let Some(assignments) = details
        .pointer("/formation/assignments")
        .and_then(Value::as_array)
    {
        println!("Assigned roles:");
        for assignment in assignments {
            println!(
                "  - {} -> {}",
                str_at(assignment, "/role").unwrap_or("role"),
                str_at(assignment, "/suggestor").unwrap_or("suggestor")
            );
        }
    }
}

fn tactic_payload(details: &Value) -> Option<&Value> {
    details
        .pointer("/context/strategies")
        .and_then(Value::as_array)?
        .iter()
        .find(|fact| fact.get("id").and_then(Value::as_str) == Some("strategy:vendor-sel:tactic"))
        .and_then(|fact| fact.get("content"))
}

fn router_payload(details: &Value) -> Option<&Value> {
    details
        .pointer("/context/strategies")
        .and_then(Value::as_array)?
        .iter()
        .find(|fact| {
            fact.get("id").and_then(Value::as_str) == Some("strategy:vendor-sel:router-hypothesis")
        })
        .and_then(|fact| fact.get("content"))
}

fn print_huddle(details: Option<&Value>) {
    section("2. Huddle");
    let Some(details) = details else {
        println!("No huddle context available.");
        return;
    };

    println!("Agents coordinate through promoted context, not direct chat.");
    for key in ["strategies", "seeds", "evaluations", "proposals"] {
        let count = details
            .pointer(&format!("/context/{key}"))
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        println!("  - {key}: {count} facts");
    }

    if let Some(agents) = details.get("agents").and_then(Value::as_array) {
        println!("Roster:");
        for agent in agents {
            println!(
                "  - {} [{}] -> {}",
                str_at(agent, "/id").unwrap_or("agent"),
                str_at(agent, "/class").unwrap_or("class"),
                str_at(agent, "/output").unwrap_or("output")
            );
        }
    }
}

fn print_steps(details: Option<&Value>) {
    section("3. Steps");
    let Some(details) = details else {
        println!("No step details available.");
        return;
    };

    print_compliance(details);
    print_cost(details);
    print_risk(details);
    print_optimization(details);
    print_recommendation(details);
}

fn print_compliance(details: &Value) {
    println!("Compliance:");
    for fact in facts_with_prefix(details, "seeds", "compliance:screen:") {
        let content = fact.get("content").unwrap_or(&Value::Null);
        println!(
            "  - {}: {} certs={}",
            str_at(content, "/vendor_name")
                .or_else(|| str_at(content, "/vendor"))
                .unwrap_or("vendor"),
            str_at(content, "/compliance_status")
                .or_else(|| str_at(content, "/status"))
                .unwrap_or("unknown"),
            array_join(content.get("certifications"))
        );
    }
}

fn print_cost(details: &Value) {
    println!("Price / Cost:");
    for fact in facts_with_prefix(details, "evaluations", "cost:estimate:") {
        let content = fact.get("content").unwrap_or(&Value::Null);
        let vendor = str_at(content, "/vendor_name")
            .or_else(|| str_at(content, "/vendor"))
            .unwrap_or("vendor");
        if let Some(cost_minor) = number_at(content, "/monthly_cost_minor") {
            println!("  - {vendor}: ${:.0}/month", cost_minor / 100.0);
        } else {
            println!(
                "  - {vendor}: ${:.0}/month value={}",
                number_at(content, "/monthly_cost_usd").unwrap_or_default(),
                number_at(content, "/value_score").unwrap_or_default()
            );
        }
    }
}

fn print_risk(details: &Value) {
    println!("Risk:");
    for fact in facts_with_prefix(details, "evaluations", "risk:score:") {
        let content = fact.get("content").unwrap_or(&Value::Null);
        println!(
            "  - {}: {} ({})",
            str_at(content, "/vendor_name")
                .or_else(|| str_at(content, "/vendor"))
                .unwrap_or("vendor"),
            str_at(content, "/risk_level")
                .or_else(|| str_at(content, "/overall_risk"))
                .unwrap_or("unknown"),
            number_at(content, "/risk_score").unwrap_or_default()
        );
    }
}

fn print_optimization(details: &Value) {
    println!("Optimization / Shortlist:");
    if let Some(objective) = str_at(details, "/optimization/objective") {
        println!("  Objective: {objective}");
    }
    if let Some(rows) = details
        .pointer("/optimization/rows")
        .and_then(Value::as_array)
    {
        for row in rows {
            println!(
                "  - {} feasible={} objective={} pareto={}",
                str_at(row, "/vendor").unwrap_or("vendor"),
                bool_at(row, "/feasible").unwrap_or(false),
                number_at(row, "/objective_score").unwrap_or_default(),
                bool_at(row, "/pareto_frontier").unwrap_or(false),
            );
        }
    }

    if let Some(shortlist) = details
        .pointer("/shortlist/shortlist")
        .and_then(Value::as_array)
    {
        println!("  Shortlist:");
        for row in shortlist {
            println!(
                "    #{} {} composite={}",
                number_at(row, "/rank").unwrap_or_default(),
                str_at(row, "/vendor_name").unwrap_or("vendor"),
                number_at(row, "/composite_score").unwrap_or_default(),
            );
        }
    }

    if let Some(rejected) = details
        .pointer("/shortlist/rejected")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
    {
        println!("  Rejected:");
        for row in rejected {
            println!(
                "    - {}: {}",
                str_at(row, "/vendor_name").unwrap_or("vendor"),
                Some(array_join(row.pointer("/reasons")))
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "no qualifying reason recorded".to_string())
            );
        }
    }
}

fn print_recommendation(details: &Value) {
    println!("Decision synthesis:");
    if let Some(recommendation) = details.get("recommendation") {
        println!(
            "  Recommendation: {}",
            str_at(recommendation, "/recommendation").unwrap_or("none")
        );
        println!(
            "  Confidence: {:.0}%",
            number_at(recommendation, "/confidence").unwrap_or_default() * 100.0
        );
        println!(
            "  Human review: {}",
            bool_at(recommendation, "/needs_human_review").unwrap_or(true)
        );
    }
}

fn print_consensus(result: &TruthExecutionResult, details: Option<&Value>) {
    section("4. Consensus");

    println!("Criteria:");
    for outcome in &result.criteria_outcomes {
        println!("  - {} => {}", outcome.criterion, outcome.result);
    }

    if let Some(policy) = details.and_then(|details| details.get("policy")) {
        println!("Policy gate:");
        println!(
            "  Outcome: {}",
            str_at(policy, "/outcome").unwrap_or("unknown")
        );
        println!(
            "  Selected vendor: {}",
            str_at(policy, "/selected_vendor").unwrap_or("none")
        );
        println!(
            "  Selected amount: ${}",
            number_at(policy, "/selected_amount_major").unwrap_or_default()
        );
        println!(
            "  Human approval present: {}",
            bool_at(policy, "/human_approval_present").unwrap_or(false)
        );
    }

    if let Some(fixed_point) = details.and_then(|details| details.get("fixed_point")) {
        println!("Fixed point:");
        println!(
            "  {}",
            str_at(fixed_point, "/definition").unwrap_or("no more promotable facts")
        );
    }

    if let Some(learning) = details.and_then(|details| details.get("learning")) {
        println!("Learning:");
        println!(
            "  prior_runs={} status={}",
            number_at(learning, "/prior_runs").unwrap_or_default(),
            str_at(learning, "/status").unwrap_or("updated")
        );
    }

    if let Some(rows) = details
        .and_then(|details| details.get("stack_pressure"))
        .and_then(Value::as_array)
    {
        println!("Foundation pressure:");
        for row in rows {
            println!(
                "  - {}: {}",
                str_at(row, "/layer").unwrap_or("layer"),
                str_at(row, "/demo_signal").unwrap_or("no signal")
            );
        }
    }
}

fn print_provider_telemetry(result: &TruthExecutionResult) {
    section("5. Provider Telemetry");
    let Some(calls) = &result.llm_calls else {
        println!("No live provider calls captured. Run with --live to exercise Brave/Tavily/LLMs.");
        return;
    };
    for call in calls {
        println!(
            "  - {} <- {}/{} {}ms tokens={}",
            call.context,
            call.provider,
            call.model,
            call.elapsed_ms,
            call.usage
                .as_ref()
                .and_then(|usage| usage.total_tokens)
                .map_or_else(|| "-".to_string(), |tokens| tokens.to_string())
        );
    }
}

fn section(title: &str) {
    println!();
    println!("--- {title} ---");
}

fn facts_with_prefix<'a>(details: &'a Value, bucket: &str, prefix: &str) -> Vec<&'a Value> {
    details
        .pointer(&format!("/context/{bucket}"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|fact| {
            fact.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.starts_with(prefix))
        })
        .collect()
}

fn str_at<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

fn number_at(value: &Value, pointer: &str) -> Option<f64> {
    value.pointer(pointer).and_then(Value::as_f64)
}

fn bool_at(value: &Value, pointer: &str) -> Option<bool> {
    value.pointer(pointer).and_then(Value::as_bool)
}

fn array_join(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("/")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn format_one_decimal(value: f64) -> String {
    format!("{value:.1}")
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format_one_decimal(value)
    }
}
