//! step-renderer: WASM crate for single-step operations.
//! Exposes render_step_scan and render_step_render to JavaScript.

use md_core::{render_step, scan_step};
use serde_json;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Pass 1 — scan a raw step markdown file.
/// Returns JSON: { includes: [...], internal_links: [...] }
#[wasm_bindgen]
pub fn render_step_scan(markdown: &str, base_path: &str) -> Result<JsValue, JsValue> {
    let result = scan_step(markdown, base_path);
    serde_json::to_string(&result)
        .map(|s| JsValue::from_str(&s))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Pass 2 — render a step to HTML, resolving includes from a preloaded asset map.
/// `assets_json` is a JSON object mapping resolved paths to file contents.
#[wasm_bindgen]
pub fn render_step_render(
    markdown: &str,
    base_path: &str,
    assets_json: &str,
) -> Result<String, JsValue> {
    let assets: HashMap<String, String> =
        serde_json::from_str(assets_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(render_step(markdown, base_path, &assets))
}

// ─── Native unit tests (test md-core logic, not WASM bindings) ───────────────
#[cfg(test)]
mod tests {
    use md_core::{render_step, scan_step};
    use std::collections::HashMap;

    #[test]
    fn test_render_step_no_includes() {
        let md = "# Hello\n\nThis is step one.";
        let result = render_step(md, "content/rust-intro/en", &HashMap::new());
        assert!(result.contains("<h1>"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_scan_step_finds_include() {
        let md = r#"::include{src="./tables/ownership.md"}"#;
        let scan = scan_step(md, "content/rust-intro/en");
        assert_eq!(scan.includes.len(), 1);
        assert_eq!(scan.includes[0].src_raw, "./tables/ownership.md");
        assert_eq!(
            scan.includes[0].resolved_path,
            "content/rust-intro/en/tables/ownership.md"
        );
    }

    #[test]
    fn test_scan_step_finds_internal_link() {
        let md = "See [Basics](./steps/020-basics.md) for more.";
        let scan = scan_step(md, "content/rust-intro/en");
        assert_eq!(scan.internal_links.len(), 1);
        assert_eq!(scan.internal_links[0].href_raw, "./steps/020-basics.md");
    }

    #[test]
    fn test_render_step_with_asset() {
        let md = r#"::include{src="./snippets/hello.rs"}"#;
        let mut assets = HashMap::new();
        assets.insert(
            "content/rust-intro/en/snippets/hello.rs".to_string(),
            "fn main() { println!(\"hello\"); }".to_string(),
        );
        let html = render_step(md, "content/rust-intro/en", &assets);
        assert!(html.contains("fn main"));
    }

    #[test]
    fn test_render_step_missing_asset_shows_error() {
        let md = r#"::include{src="./tables/missing.md"}"#;
        let html = render_step(md, "content/rust-intro/en", &HashMap::new());
        assert!(html.contains("Missing include"));
    }
}
