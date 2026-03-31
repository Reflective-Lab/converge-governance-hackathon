use std::collections::HashMap;

use governance_kernel::InMemoryStore;
use serde::{Deserialize, Serialize};

use super::TruthExecutionResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TruthSourceFormat {
    Gherkin,
    TruthsJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthSourceFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSelectionSourcePreview {
    pub source_name: String,
    pub title: String,
    pub truth_key: String,
    pub format: TruthSourceFormat,
    pub vendors: Vec<String>,
    pub inputs: HashMap<String, String>,
}

pub fn preview_vendor_selection_source(
    source: TruthSourceFile,
) -> Result<VendorSelectionSourcePreview, String> {
    let format = detect_format(&source)?;
    match format {
        TruthSourceFormat::Gherkin => preview_gherkin_vendor_selection(source),
        TruthSourceFormat::TruthsJson => preview_truths_json_vendor_selection(source),
    }
}

pub fn execute_vendor_selection_source(
    store: &InMemoryStore,
    source: TruthSourceFile,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let preview = preview_vendor_selection_source(source)?;
    super::execute_truth(store, &preview.truth_key, preview.inputs, persist)
}

fn detect_format(source: &TruthSourceFile) -> Result<TruthSourceFormat, String> {
    let name = source.name.to_ascii_lowercase();
    let trimmed = source.content.trim_start();

    if name.ends_with(".feature") || trimmed.contains("\nFeature:") || trimmed.starts_with("Feature:")
    {
        return Ok(TruthSourceFormat::Gherkin);
    }

    if name.ends_with(".truths.json") || name.ends_with(".json") || trimmed.starts_with('{') {
        return Ok(TruthSourceFormat::TruthsJson);
    }

    Err(format!(
        "unsupported source format for {}: expected .feature or .truths.json",
        source.name
    ))
}

fn preview_gherkin_vendor_selection(
    source: TruthSourceFile,
) -> Result<VendorSelectionSourcePreview, String> {
    let mut title = None;
    let mut truth_key = None;
    let mut vendors = vec![];
    let mut collecting_vendor_table = false;

    for raw_line in source.content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            if collecting_vendor_table && line.is_empty() {
                collecting_vendor_table = false;
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("Feature:") {
            title = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = extract_step_value(line, &["Given truth", "And truth", "When truth"])
        {
            truth_key = Some(value);
            collecting_vendor_table = false;
            continue;
        }

        if let Some(value) =
            extract_step_value(line, &["Given vendors", "And vendors", "When vendors"])
        {
            vendors.extend(split_csv(&value));
            collecting_vendor_table = false;
            continue;
        }

        if mentions_vendor_table(line) {
            collecting_vendor_table = true;
            continue;
        }

        if collecting_vendor_table && line.starts_with('|') {
            if let Some(vendor) = parse_vendor_table_row(line) {
                vendors.push(vendor);
            }
            continue;
        }

        if collecting_vendor_table && is_step_line(line) {
            collecting_vendor_table = false;
        }
    }

    build_preview(
        source.name,
        title,
        truth_key.unwrap_or_else(|| "evaluate-vendor".into()),
        TruthSourceFormat::Gherkin,
        vendors,
    )
}

fn preview_truths_json_vendor_selection(
    source: TruthSourceFile,
) -> Result<VendorSelectionSourcePreview, String> {
    #[derive(Debug, Deserialize)]
    struct TruthSpecFile {
        title: Option<String>,
        #[serde(default, alias = "truth", alias = "truthKey")]
        truth_key: Option<String>,
        #[serde(default, alias = "vendor_names", alias = "vendorNames")]
        vendors: Vec<String>,
        #[serde(default)]
        inputs: HashMap<String, String>,
    }

    let spec: TruthSpecFile = serde_json::from_str(&source.content)
        .map_err(|e| format!("invalid truth spec JSON in {}: {e}", source.name))?;

    let mut vendors = spec.vendors;
    if vendors.is_empty()
        && let Some(raw_vendors) = spec.inputs.get("vendors") {
            vendors.extend(split_csv(raw_vendors));
        }

    let mut preview = build_preview(
        source.name,
        spec.title,
        spec.truth_key.unwrap_or_else(|| "evaluate-vendor".into()),
        TruthSourceFormat::TruthsJson,
        vendors,
    )?;

    for (key, value) in spec.inputs {
        if !value.trim().is_empty() {
            preview.inputs.insert(key, value.trim().to_string());
        }
    }
    preview.inputs.insert("vendors".into(), preview.vendors.join(", "));

    Ok(preview)
}

fn build_preview(
    source_name: String,
    title: Option<String>,
    truth_key: String,
    format: TruthSourceFormat,
    vendors: Vec<String>,
) -> Result<VendorSelectionSourcePreview, String> {
    if truth_key != "evaluate-vendor" {
        return Err(format!(
            "this prep repo is currently centered on vendor selection; expected truth_key \
             \"evaluate-vendor\", got \"{truth_key}\""
        ));
    }

    let vendors = normalize_vendors(vendors);
    if vendors.is_empty() {
        return Err("no vendors found in source; add a vendor list so the vendor-selection flow can run".to_string());
    }

    let title = title
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| fallback_title_from_name(&source_name));

    let inputs = HashMap::from([("vendors".into(), vendors.join(", "))]);

    Ok(VendorSelectionSourcePreview {
        source_name,
        title,
        truth_key,
        format,
        vendors,
        inputs,
    })
}

fn extract_step_value(line: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let trimmed = rest.trim();
            if let Some(value) = parse_quoted_value(trimmed) {
                return Some(value);
            }
            if !trimmed.is_empty() && !trimmed.ends_with(':') {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn parse_quoted_value(value: &str) -> Option<String> {
    let mut chars = value.chars();
    if chars.next()? != '"' {
        return None;
    }
    let remainder: String = chars.collect();
    let end = remainder.find('"')?;
    Some(remainder[..end].trim().to_string())
}

fn mentions_vendor_table(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("vendors:") || lower.contains("candidate vendors:")
}

fn is_step_line(line: &str) -> bool {
    ["Given ", "When ", "Then ", "And ", "But "]
        .iter()
        .any(|prefix| line.starts_with(prefix))
}

fn parse_vendor_table_row(line: &str) -> Option<String> {
    let cells: Vec<String> = line
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();

    let first = cells.first()?.trim();
    if first.eq_ignore_ascii_case("name") || first.eq_ignore_ascii_case("vendor") {
        return None;
    }
    Some(first.to_string())
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn normalize_vendors(vendors: Vec<String>) -> Vec<String> {
    let mut normalized = vec![];
    for vendor in vendors {
        let vendor = vendor.trim();
        if vendor.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(vendor))
        {
            continue;
        }
        normalized.push(vendor.to_string());
    }
    normalized
}

fn fallback_title_from_name(name: &str) -> String {
    let stem = name
        .rsplit_once('/')
        .map(|(_, tail)| tail)
        .unwrap_or(name)
        .split('.')
        .next()
        .unwrap_or(name)
        .replace(['-', '_'], " ");

    if stem.trim().is_empty() {
        "Vendor Selection".into()
    } else {
        stem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_gherkin_with_inline_vendor_step() {
        let preview = preview_vendor_selection_source(TruthSourceFile {
            name: "vendor-selection.feature".into(),
            content: r#"
Feature: Evaluate AI vendors

  Scenario: shortlist vendors
    Given truth "evaluate-vendor"
    And vendors "Acme AI, Beta ML, Gamma LLM"
"#
            .into(),
        })
        .unwrap();

        assert_eq!(preview.title, "Evaluate AI vendors");
        assert_eq!(preview.truth_key, "evaluate-vendor");
        assert_eq!(preview.format, TruthSourceFormat::Gherkin);
        assert_eq!(preview.vendors, vec!["Acme AI", "Beta ML", "Gamma LLM"]);
    }

    #[test]
    fn previews_gherkin_with_vendor_table() {
        let preview = preview_vendor_selection_source(TruthSourceFile {
            name: "vendor-selection.feature".into(),
            content: r#"
Feature: Vendor review

  Scenario: decide
    Given vendors:
      | name      |
      | Acme AI   |
      | Beta ML   |
"#
            .into(),
        })
        .unwrap();

        assert_eq!(preview.vendors, vec!["Acme AI", "Beta ML"]);
    }

    #[test]
    fn previews_truths_json_file() {
        let preview = preview_vendor_selection_source(TruthSourceFile {
            name: "vendor-selection.truths.json".into(),
            content: r#"{
  "title": "Desktop vendor selection",
  "truth_key": "evaluate-vendor",
  "vendors": ["Acme AI", "Beta ML"]
}"#
            .into(),
        })
        .unwrap();

        assert_eq!(preview.title, "Desktop vendor selection");
        assert_eq!(preview.format, TruthSourceFormat::TruthsJson);
        assert_eq!(preview.inputs.get("vendors").unwrap(), "Acme AI, Beta ML");
    }

    #[test]
    fn rejects_non_vendor_selection_truths() {
        let error = preview_vendor_selection_source(TruthSourceFile {
            name: "other.truths.json".into(),
            content: r#"{
  "truth_key": "other-truth",
  "vendors": ["Acme AI"]
}"#
            .into(),
        })
        .unwrap_err();

        assert!(error.contains("evaluate-vendor"));
    }
}
