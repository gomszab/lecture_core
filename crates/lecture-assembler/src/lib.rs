//! lecture-assembler: WASM crate for lecture-level assembly.
//! Exposes parse_lecture_yaml, assemble_lecture_spa, and collect_diagnostics.

use md_core::{
    collect_lecture_diagnostics, collect_step_diagnostics, parse_lecture_yaml_str,
    rewrite_internal_links, Diagnostic, LectureYaml, StepScanResult,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// ─── Supporting types ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct RenderedStep {
    slug: String,
    title: String,
    filename: String,
    html: String,
}

#[derive(Serialize, Deserialize)]
struct StepScanEntry {
    filename: String,
    scan: StepScanResult,
}

// ─── Internal helpers (also used by tests) ────────────────────────────────────

fn build_spa_html(lecture: &LectureYaml, steps: &[RenderedStep]) -> String {
    let slug_map: HashMap<String, String> = steps
        .iter()
        .map(|s| {
            (
                format!("./steps/{}", s.filename),
                format!("step-{}", s.slug),
            )
        })
        .collect();

    let nav_items: String = steps
        .iter()
        .map(|s| {
            format!(
                "<li><a href=\"#step-{}\" class=\"nav-link\">{}</a></li>",
                s.slug, s.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let sections: String = steps
        .iter()
        .map(|s| {
            let rewritten = rewrite_internal_links(&s.html, &slug_map);
            format!(
                "<section id=\"step-{}\" class=\"lecture-step\">\n  <h2 class=\"step-title\">{}</h2>\n  <div class=\"step-body\">{}</div>\n</section>",
                s.slug, s.title, rewritten
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "<div class=\"lecture-spa\" data-slug=\"{}\" data-lang=\"{}\">\n  <aside class=\"lecture-sidebar\">\n    <h3>{}</h3>\n    <ul class=\"step-nav\">{}</ul>\n  </aside>\n  <main class=\"lecture-content\">{}</main>\n</div>",
        lecture.lecture.slug,
        lecture.lecture.lang,
        lecture.lecture.title,
        nav_items,
        sections
    )
}

/// `step_filenames`    — filenames found on disk (for MISSING_STEP_METADATA checks)
/// `loaded_asset_paths` — resolved include paths that were successfully fetched
///                        (the only ones that should NOT be flagged as BROKEN_INCLUDE)
fn build_diagnostics(
    lecture: &LectureYaml,
    step_filenames: &[String],
    scan_entries: &[StepScanEntry],
    loaded_asset_paths: &[String],
) -> Vec<Diagnostic> {
    let known_slugs: Vec<String> = lecture.steps.iter().map(|s| s.slug.clone()).collect();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Lecture-level: step files declared in YAML but missing from disk
    diagnostics.extend(collect_lecture_diagnostics(lecture, step_filenames));

    // Per-step: broken includes and broken internal links
    for entry in scan_entries {
        diagnostics.extend(collect_step_diagnostics(
            &entry.filename,
            &entry.scan,
            loaded_asset_paths, // only successfully fetched assets are "available"
            &known_slugs,
        ));
    }

    diagnostics
}

// ─── WASM exports ─────────────────────────────────────────────────────────────

/// Parse lecture.yaml and return lecture metadata + ordered step list as JSON.
#[wasm_bindgen]
pub fn parse_lecture_yaml(lecture_yaml: &str) -> Result<JsValue, JsValue> {
    let lecture = parse_lecture_yaml_str(lecture_yaml)
        .map_err(|e| JsValue::from_str(&format!("YAML parse error: {}", e)))?;
    serde_json::to_string(&lecture)
        .map(|s| JsValue::from_str(&s))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Assemble rendered step HTML fragments into a full SPA lecture shell.
/// `rendered_steps_json` is an array of { slug, title, filename, html }.
#[wasm_bindgen]
pub fn assemble_lecture_spa(
    lecture_yaml: &str,
    rendered_steps_json: &str,
) -> Result<String, JsValue> {
    let lecture = parse_lecture_yaml_str(lecture_yaml)
        .map_err(|e| JsValue::from_str(&format!("YAML parse error: {}", e)))?;
    let steps: Vec<RenderedStep> = serde_json::from_str(rendered_steps_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(build_spa_html(&lecture, &steps))
}

/// Collect structured diagnostics for a lecture.
///
/// Parameters:
/// - `lecture_yaml`        — raw YAML string
/// - `steps_json`          — array of `{ filename }` (step files found on disk)
/// - `scan_results_json`   — array of `{ filename, scan: StepScanResult }`
/// - `loaded_assets_json`  — array of resolved asset paths that were successfully fetched
///                           e.g. `["content/rust-intro/en/tables/ownership.md"]`
///                           Only missing entries produce BROKEN_INCLUDE diagnostics.
#[wasm_bindgen]
pub fn collect_diagnostics(
    lecture_yaml: &str,
    steps_json: &str,
    scan_results_json: &str,
    loaded_assets_json: &str,
) -> Result<JsValue, JsValue> {
    let lecture = parse_lecture_yaml_str(lecture_yaml)
        .map_err(|e| JsValue::from_str(&format!("YAML parse error: {}", e)))?;

    let step_filenames: Vec<String> = serde_json::from_str::<Vec<serde_json::Value>>(steps_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .into_iter()
        .filter_map(|v| v["filename"].as_str().map(|s| s.to_string()))
        .collect();

    let scan_entries: Vec<StepScanEntry> = serde_json::from_str(scan_results_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Loaded asset paths come from JS — these are the paths fetch() actually succeeded for
    let loaded_asset_paths: Vec<String> = serde_json::from_str(loaded_assets_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let diagnostics = build_diagnostics(
        &lecture,
        &step_filenames,
        &scan_entries,
        &loaded_asset_paths,
    );

    serde_json::to_string(&diagnostics)
        .map(|s| JsValue::from_str(&s))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ─── Native unit tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use md_core::parse_lecture_yaml_str;

    const YAML: &str = "
lecture:
  title: \"Rust Intro\"
  slug: \"rust-intro\"
  lang: \"en\"
steps:
  - filename: \"010-welcome.md\"
    title: \"Welcome\"
    slug: \"welcome\"
  - filename: \"020-basics.md\"
    title: \"Basics\"
    slug: \"basics\"
";

    #[test]
    fn test_parse_lecture_yaml_valid() {
        let result = parse_lecture_yaml_str(YAML);
        assert!(result.is_ok());
        let lec = result.unwrap();
        assert_eq!(lec.lecture.title, "Rust Intro");
        assert_eq!(lec.steps.len(), 2);
    }

    #[test]
    fn test_build_spa_html() {
        let lecture = parse_lecture_yaml_str(YAML).unwrap();
        let steps = vec![
            RenderedStep {
                slug: "welcome".to_string(),
                title: "Welcome".to_string(),
                filename: "010-welcome.md".to_string(),
                html: "<p>Hello</p>".to_string(),
            },
            RenderedStep {
                slug: "basics".to_string(),
                title: "Basics".to_string(),
                filename: "020-basics.md".to_string(),
                html: "<p>Basics</p>".to_string(),
            },
        ];
        let html = build_spa_html(&lecture, &steps);
        assert!(html.contains("lecture-spa"));
        assert!(html.contains("step-welcome"));
        assert!(html.contains("step-basics"));
    }

    #[test]
    fn test_internal_link_rewriting() {
        let lecture = parse_lecture_yaml_str(YAML).unwrap();
        let steps = vec![
            RenderedStep {
                slug: "welcome".to_string(),
                title: "Welcome".to_string(),
                filename: "010-welcome.md".to_string(),
                html: "<a href=\"./steps/020-basics.md\">Basics</a>".to_string(),
            },
            RenderedStep {
                slug: "basics".to_string(),
                title: "Basics".to_string(),
                filename: "020-basics.md".to_string(),
                html: "<p>Basics content</p>".to_string(),
            },
        ];
        let html = build_spa_html(&lecture, &steps);
        assert!(html.contains("href=\"#step-basics\""));
        assert!(!html.contains("href=\"./steps/020-basics.md\""));
    }

    #[test]
    fn test_diagnostics_only_flags_missing_assets() {
        use md_core::{IncludeDirective, StepScanResult};

        let lecture = parse_lecture_yaml_str(YAML).unwrap();
        let step_files = vec![
            "010-welcome.md".to_string(),
            "020-basics.md".to_string(),
        ];

        // Two includes: one that loaded successfully, one that did not
        let scan_entries = vec![StepScanEntry {
            filename: "020-basics.md".to_string(),
            scan: StepScanResult {
                includes: vec![
                    IncludeDirective {
                        src_raw: "./tables/ownership.md".to_string(),
                        resolved_path: "content/rust-intro/en/tables/ownership.md".to_string(),
                    },
                    IncludeDirective {
                        src_raw: "./tables/missing.md".to_string(),
                        resolved_path: "content/rust-intro/en/tables/missing.md".to_string(),
                    },
                ],
                internal_links: vec![],
            },
        }];

        // Only ownership.md was successfully fetched
        let loaded = vec!["content/rust-intro/en/tables/ownership.md".to_string()];

        let diags = build_diagnostics(&lecture, &step_files, &scan_entries, &loaded);

        // Should be exactly 1 diagnostic — for missing.md only
        let broken: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d.code, md_core::DiagnosticCode::BrokenInclude))
            .collect();
        assert_eq!(broken.len(), 1);
        assert!(broken[0].message.contains("missing.md"));
    }

    #[test]
    fn test_diagnostics_no_false_positives_when_all_loaded() {
        use md_core::{IncludeDirective, StepScanResult};

        let lecture = parse_lecture_yaml_str(YAML).unwrap();
        let step_files = vec![
            "010-welcome.md".to_string(),
            "020-basics.md".to_string(),
        ];
        let scan_entries = vec![StepScanEntry {
            filename: "010-welcome.md".to_string(),
            scan: StepScanResult {
                includes: vec![IncludeDirective {
                    src_raw: "./tables/ownership.md".to_string(),
                    resolved_path: "content/rust-intro/en/tables/ownership.md".to_string(),
                }],
                internal_links: vec![],
            },
        }];

        // All includes were successfully loaded
        let loaded = vec!["content/rust-intro/en/tables/ownership.md".to_string()];

        let diags = build_diagnostics(&lecture, &step_files, &scan_entries, &loaded);
        let broken: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d.code, md_core::DiagnosticCode::BrokenInclude))
            .collect();
        assert_eq!(broken.len(), 0, "No false-positive BROKEN_INCLUDE diagnostics");
    }
}
