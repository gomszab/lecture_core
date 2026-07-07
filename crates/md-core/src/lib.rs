//! md-core: Pure Rust shared logic. No browser bindings.
//! Provides: markdown rendering, include directive parsing,
//! path resolution, internal link detection, diagnostics, shared types.

use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Shared Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LectureYaml {
    pub lecture: LectureMeta,
    pub steps: Vec<StepMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LectureMeta {
    pub title: String,
    pub slug: String,
    pub lang: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepMeta {
    pub filename: String,
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeDirective {
    pub src_raw: String,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalLink {
    pub href_raw: String,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepScanResult {
    pub includes: Vec<IncludeDirective>,
    pub internal_links: Vec<InternalLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticCode {
    BrokenInclude,
    BrokenInternalLink,
    MissingStepMetadata,
    InvalidIncludePath,
    NestedIncludeDisallowed,
    MissingRequiredField,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: DiagnosticCode,
    pub message: String,
    pub step_filename: Option<String>,
}

// ─── Markdown Rendering ───────────────────────────────────────────────────────

/// Render a Markdown string to HTML using pulldown-cmark.
pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(markdown, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

// ─── Include Directive Parsing ────────────────────────────────────────────────

/// Extract all `::include{src="..."}` directives from markdown source.
pub fn parse_includes(markdown: &str) -> Vec<String> {
    let re = Regex::new(r#"::include\{src="([^"]+)"\}"#).unwrap();
    re.captures_iter(markdown)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Validate include path: must be relative, no absolute paths, no nested includes
/// (nested includes are detected by context, not here — this checks path validity).
pub fn validate_include_path(src: &str) -> Result<(), DiagnosticCode> {
    if src.starts_with('/') || src.contains("://") {
        return Err(DiagnosticCode::InvalidIncludePath);
    }
    Ok(())
}

// ─── Path Resolution ──────────────────────────────────────────────────────────

/// Resolve a relative path like `./tables/foo.md` against a base like
/// `content/rust-intro/en` → `content/rust-intro/en/tables/foo.md`
pub fn resolve_path(base_path: &str, relative: &str) -> String {
    let base = base_path.trim_end_matches('/');
    let rel = relative.trim_start_matches("./");
    format!("{}/{}", base, rel)
}

// ─── Internal Link Detection ──────────────────────────────────────────────────

/// Detect Markdown links that point to relative `.md` step paths.
pub fn parse_internal_links(markdown: &str) -> Vec<String> {
    let re = Regex::new(r#"\[([^\]]+)\]\((\.\/steps\/[^)]+\.md)\)"#).unwrap();
    re.captures_iter(markdown)
        .map(|cap| cap[2].to_string())
        .collect()
}

// ─── Step Scanning ────────────────────────────────────────────────────────────

/// Pass 1: scan a raw step markdown file and return all includes + internal links.
pub fn scan_step(markdown: &str, base_path: &str) -> StepScanResult {
    let raw_includes = parse_includes(markdown);
    let raw_links = parse_internal_links(markdown);

    let includes = raw_includes
        .into_iter()
        .map(|src_raw| {
            let resolved_path = resolve_path(base_path, &src_raw);
            IncludeDirective { src_raw, resolved_path }
        })
        .collect();

    let internal_links = raw_links
        .into_iter()
        .map(|href_raw| {
            let resolved_path = resolve_path(base_path, &href_raw);
            InternalLink { href_raw, resolved_path }
        })
        .collect();

    StepScanResult { includes, internal_links }
}

// ─── Step Rendering (Pass 2) ──────────────────────────────────────────────────

/// Pass 2: render a step markdown file, substituting includes from an asset map,
/// and preserving internal links for later SPA anchor rewriting.
pub fn render_step(
    markdown: &str,
    base_path: &str,
    assets: &HashMap<String, String>,
) -> String {
    let include_re = Regex::new(r#"::include\{src="([^"]+)"\}"#).unwrap();

    // Replace ::include directives with asset content or error blocks
    let expanded = include_re.replace_all(markdown, |caps: &regex::Captures| {
        let src_raw = &caps[1];
        let resolved = resolve_path(base_path, src_raw);
        match assets.get(&resolved) {
            Some(content) => {
                // Wrap in a fenced code block if it's a code snippet
                if src_raw.ends_with(".rs") || src_raw.ends_with(".wit") || src_raw.ends_with(".toml") {
                    let ext = src_raw.rsplit('.').next().unwrap_or("text");
                    format!("```{}\n{}\n```", ext, content.trim())
                } else {
                    content.clone()
                }
            }
            None => format!(
                "<div class=\"diagnostic-inline error\">⚠ Missing include: <code>{}</code></div>",
                src_raw
            ),
        }
    });

    render_markdown(&expanded)
}

// ─── YAML Parsing ─────────────────────────────────────────────────────────────

pub fn parse_lecture_yaml_str(yaml: &str) -> Result<LectureYaml, String> {
    serde_yaml::from_str::<LectureYaml>(yaml).map_err(|e| e.to_string())
}

// ─── Internal Link Rewriting ──────────────────────────────────────────────────

/// Given a slug map (relative path → SPA anchor id), rewrite all internal links
/// in a rendered HTML fragment.
pub fn rewrite_internal_links(html: &str, slug_map: &HashMap<String, String>) -> String {
    let mut result = html.to_string();
    for (href_raw, anchor) in slug_map {
        // Match both URL-encoded and raw href variants
        let pattern = format!("href=\"{}\"", href_raw);
        let replacement = format!("href=\"#{}\"", anchor);
        result = result.replace(&pattern, &replacement);
    }
    result
}

// ─── Diagnostics ─────────────────────────────────────────────────────────────

pub fn collect_step_diagnostics(
    step_filename: &str,
    scan: &StepScanResult,
    available_assets: &[String],
    known_step_slugs: &[String],
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for inc in &scan.includes {
        if let Err(code) = validate_include_path(&inc.src_raw) {
            diags.push(Diagnostic {
                level: DiagnosticLevel::Error,
                code,
                message: format!("Invalid include path: {}", inc.src_raw),
                step_filename: Some(step_filename.to_string()),
            });
        } else if !available_assets.contains(&inc.resolved_path) {
            diags.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                code: DiagnosticCode::BrokenInclude,
                message: format!("Missing include: {}", inc.src_raw),
                step_filename: Some(step_filename.to_string()),
            });
        }
    }

    for link in &scan.internal_links {
        // Extract slug from path like ./steps/020-basics.md → basics
        let slug = link
            .href_raw
            .rsplit('/')
            .next()
            .unwrap_or("")
            .trim_end_matches(".md")
            .splitn(2, '-')
            .nth(1)
            .unwrap_or("")
            .to_string();
        if !known_step_slugs.contains(&slug) {
            diags.push(Diagnostic {
                level: DiagnosticLevel::Error,
                code: DiagnosticCode::BrokenInternalLink,
                message: format!("Target step not found: {}", link.href_raw),
                step_filename: Some(step_filename.to_string()),
            });
        }
    }

    diags
}

pub fn collect_lecture_diagnostics(lecture: &LectureYaml, step_filenames: &[String]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for step_meta in &lecture.steps {
        if !step_filenames.contains(&step_meta.filename) {
            diags.push(Diagnostic {
                level: DiagnosticLevel::Error,
                code: DiagnosticCode::MissingStepMetadata,
                message: format!("Step file not found on disk: {}", step_meta.filename),
                step_filename: Some(step_meta.filename.clone()),
            });
        }
    }

    diags
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_includes() {
        let md = r#"Hello\n::include{src="./tables/ownership.md"}\nWorld"#;
        let includes = parse_includes(md);
        assert_eq!(includes, vec!["./tables/ownership.md"]);
    }

    #[test]
    fn test_parse_internal_links() {
        let md = r#"See [Basics](./steps/020-basics.md) for more."#;
        let links = parse_internal_links(md);
        assert_eq!(links, vec!["./steps/020-basics.md"]);
    }

    #[test]
    fn test_resolve_path() {
        assert_eq!(
            resolve_path("content/rust-intro/en", "./tables/ownership.md"),
            "content/rust-intro/en/tables/ownership.md"
        );
    }

    #[test]
    fn test_validate_include_path_invalid() {
        assert!(validate_include_path("/etc/passwd").is_err());
        assert!(validate_include_path("https://evil.com/x").is_err());
    }

    #[test]
    fn test_render_markdown_basic() {
        let html = render_markdown("# Hello\n\nWorld");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
    }
}
