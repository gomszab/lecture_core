//! Pure Rust library — no tauri proc-macros here.
//! Thin #[tauri::command] wrappers live in src-tauri/src/main.rs.

use md_core::{
    Diagnostic, LectureMeta, LectureYaml, StepMeta,
    parse_lecture_yaml_str, collect_lecture_diagnostics,
    collect_step_diagnostics, scan_step,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fs, vec};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LectureEntry {
    pub slug: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lecture{
    pub lecture: LectureYaml,
    pub assets: Vec<LectureAssets>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LectureAssets {
    path: String,
    name: String
}

// ── Path helpers ─────────────────────────────────────────────────────────────-

fn normalize_path_str(p: &str) -> String {
    // Windows tolerant: convert backslashes to forward slashes so
    // "C:\foo\bar.md" and "C:/foo/bar.md" compare equal.[web:57]
    p.replace('\\', "/")
}

fn normalize_pathbuf(path: &Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn load_lecture_yaml(path: &Path) -> Result<LectureYaml, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    parse_lecture_yaml_str(&content).map_err(|e| {
        format!("YAML parse error in {}: {e}", path.display())
    })
}

pub fn save_lecture_yaml(path: &Path, lecture: &LectureYaml) -> Result<(), String> {
    let content = serde_yaml::to_string(lecture)
        .map_err(|e| format!("Serialize: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Write: {e}"))
}

// ── Workspace ─────────────────────────────────────────────────────────────────

pub fn scan_workspace(content_path: String) -> Result<Vec<LectureEntry>, String> {
    let root = Path::new(&content_path);
    if !root.is_dir() {
        return Err(format!("Not a directory: {content_path}"));
    }

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for entry in WalkDir::new(root).max_depth(3).min_depth(1) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.file_name().map(|n| n == "lecture.yaml").unwrap_or(false) {
            if let (Some(lang_dir), Some(slug_dir)) =
                (path.parent(), path.parent().and_then(|p| p.parent()))
            {
                let lang = lang_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let slug = slug_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                map.entry(slug).or_default().push(lang);
            }
        }
    }

    Ok(map
        .into_iter()
        .map(|(slug, mut languages)| {
            languages.sort();
            LectureEntry { slug, languages }
        })
        .collect())
}

fn load_assets_folders(path: &Path) -> Result<Vec<LectureAssets>, String> {
    let parent_folder = path.parent().expect("Invalid path");
    let assets = read_asset_dir(&parent_folder.join("assets"))?;
    Ok(assets)
}

fn read_asset_dir(dir: &Path) -> Result<Vec<LectureAssets>, String>{
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut assets = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file()){
        let path = entry.path().to_str().unwrap().to_string();
        let name = entry.file_name().to_str().unwrap().to_string();

        assets.push(LectureAssets { path, name });
    }

    Ok(assets)
}

pub fn load_lecture(lecture_yaml_path: String) -> Result<Lecture, String> {
    let lecture = load_lecture_yaml(Path::new(&lecture_yaml_path))?;
    let assets = load_assets_folders(Path::new(&lecture_yaml_path))?;
    Ok(Lecture{
        lecture,
        assets
    })
}

pub fn save_lecture(
    lecture_yaml_path: String,
    lecture: LectureYaml,
) -> Result<(), String> {
    save_lecture_yaml(Path::new(&lecture_yaml_path), &lecture)
}

pub fn regenerate_manifest(
    content_path: String,
    manifest_path: String,
) -> Result<(), String> {
    let entries = scan_workspace(content_path)?;
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("JSON: {e}"))?;
    fs::write(&manifest_path, json).map_err(|e| format!("Write: {e}"))
}

pub fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Read {path}: {e}"))
}

pub fn write_file(path: String, content: String) -> Result<(), String> {
    if let Some(p) = Path::new(&path).parent() {
        fs::create_dir_all(p).map_err(|e| format!("mkdir: {e}"))?;
    }
    fs::write(&path, content).map_err(|e| format!("Write {path}: {e}"))
}

pub fn list_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(&path).map_err(|e| format!("readdir: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path().to_string_lossy().into_owned(),
            is_dir: meta.is_dir(),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub fn create_lecture(
    content_path: String,
    slug: String,
    lang: String,
    title: String,
) -> Result<(), String> {
    let dir = PathBuf::from(&content_path).join(&slug).join(&lang);
    for sub in ["steps", "tables", "snippets"] {
        fs::create_dir_all(dir.join(sub)).map_err(|e| format!("mkdir: {e}"))?;
    }

    let fname = "010-welcome.md";
    let yaml = LectureYaml {
        lecture: LectureMeta {
            title: title.clone(),
            slug,
            lang,
            description: None,
        },
        steps: vec![StepMeta {
            filename: fname.into(),
            title: "Welcome".into(),
            slug: "welcome".into(),
        }],
    };

    save_lecture_yaml(&dir.join("lecture.yaml"), &yaml)?;
    fs::write(
        dir.join("steps").join(fname),
        format!("# Welcome to {title}\n\nStart writing here.\n"),
    )
    .map_err(|e| format!("Write step: {e}"))
}

pub fn create_language(lecture_path: String, lang: String) -> Result<(), String> {
    let lang_dir = PathBuf::from(&lecture_path).join(&lang);
    for sub in ["steps", "assets"] {
        fs::create_dir_all(lang_dir.join(sub)).map_err(|e| format!("mkdir: {e}"))?;
    }

    let slug = Path::new(&lecture_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let yaml = LectureYaml {
        lecture: LectureMeta {
            title: slug.replace('-', " "),
            slug,
            lang,
            description: None,
        },
        steps: vec![StepMeta {
            filename: "010-welcome.md".into(),
            title: "Welcome".into(),
            slug: "welcome".into(),
        }],
    };

    save_lecture_yaml(&lang_dir.join("lecture.yaml"), &yaml)?;
    fs::write(
        lang_dir.join("steps").join("010-welcome.md"),
        "# Welcome\n\nWrite here.\n",
    )
    .map_err(|e| format!("Write step: {e}"))
}

pub fn create_step(
    lecture_path: String,
    lang: String,
    slug: String,
    title: String,
) -> Result<(), String> {
    let lang_dir = PathBuf::from(&lecture_path).join(&lang);
    let yaml_path = lang_dir.join("lecture.yaml");
    let mut lecture = load_lecture_yaml(&yaml_path)?;

    let next_n = (lecture.steps.len() as u32 + 1) * 10;
    let filename = format!("{:03}-{slug}.md", next_n);

    lecture.steps.push(StepMeta {
        filename: filename.clone(),
        title,
        slug,
    });
    save_lecture_yaml(&yaml_path, &lecture)?;

    let step_path = lang_dir.join("steps").join(&filename);
    if !step_path.exists() {
        fs::write(&step_path, "# New Step\n\nWrite here.\n")
            .map_err(|e| format!("Write: {e}"))?;
    }
    Ok(())
}

pub fn delete_step(
    lecture_path: String,
    lang: String,
    filename: String,
) -> Result<(), String> {
    let lang_dir = PathBuf::from(&lecture_path).join(&lang);
    let yaml_path = lang_dir.join("lecture.yaml");
    let mut lecture = load_lecture_yaml(&yaml_path)?;

    lecture.steps.retain(|s| s.filename != filename);
    save_lecture_yaml(&yaml_path, &lecture)?;

    let sp = lang_dir.join("steps").join(&filename);
    if sp.exists() {
        fs::remove_file(&sp).map_err(|e| format!("Remove: {e}"))?;
    }
    Ok(())
}

pub fn rename_step(
    lecture_path: String,
    lang: String,
    filename: String,
    new_title: String,
) -> Result<(), String> {
    let lang_dir = PathBuf::from(&lecture_path).join(&lang);
    let yaml_path = lang_dir.join("lecture.yaml");
    let mut lecture = load_lecture_yaml(&yaml_path)?;

    lecture
        .steps
        .iter_mut()
        .find(|s| s.filename == filename)
        .ok_or_else(|| format!("Step not found: {filename}"))
        .map(|s| s.title = new_title)?;

    save_lecture_yaml(&yaml_path, &lecture)
}

pub fn reorder_steps(
    lecture_yaml_path: String,
    ordered_filenames: Vec<String>,
) -> Result<(), String> {
    let path = Path::new(&lecture_yaml_path);
    let mut lecture = load_lecture_yaml(path)?;

    let mut reordered: Vec<StepMeta> = ordered_filenames
        .iter()
        .filter_map(|f| lecture.steps.iter().find(|s| &s.filename == f).cloned())
        .collect();

    for step in &lecture.steps {
        if !ordered_filenames.contains(&step.filename) {
            reordered.push(step.clone());
        }
    }

    lecture.steps = reordered;
    save_lecture_yaml(path, &lecture)
}

// ── Diagnostics ───────────────────────────────────────────────────────────────

pub fn run_diagnostics(lecture_yaml_path: String) -> Result<Vec<Diagnostic>, String> {
    let yaml_path = Path::new(&lecture_yaml_path);
    let lecture = load_lecture_yaml(yaml_path)?;
    let base_dir = yaml_path
        .parent()
        .ok_or("Cannot determine lecture directory")?;

    // This is the directory that contains lecture.yaml (e.g. .../content/rust-intro/en).
    // Normalize to forward slashes so scan_step/resolve_path and our available-assets list
    // are comparable on Windows and POSIX.[web:57]
    let base_str = normalize_pathbuf(base_dir);

    let steps_dir = base_dir.join("steps");

    // For lecture-level diagnostics: we only need filenames, not full paths.
    let disk_files: Vec<String> = fs::read_dir(&steps_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();

    let known_slugs: Vec<String> = lecture.steps.iter().map(|s| s.slug.clone()).collect();
    let mut diags = collect_lecture_diagnostics(&lecture, &disk_files);

    // Build a normalized list of available asset paths (tables + snippets).
    // These are absolute OS paths normalized to forward slashes, e.g.
    // "C:/.../content/rust-intro/en/tables/foo.md".
    let mut available: Vec<String> = Vec::new();
    
    let d = base_dir.join("assets");
    if let Ok(rd) = fs::read_dir(&d) {
        for entry in rd.flatten() {
            available.push(normalize_pathbuf(&entry.path()));
        }
    }


    // Per-step diagnostics: scan with the same normalized base path string,
    // so IncludeDirective.resolved_path lines up with the normalized assets above.
    for step in &lecture.steps {
        let step_path = steps_dir.join(&step.filename);
        let content = match fs::read_to_string(&step_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let scan = scan_step(&content, &base_str);
        diags.extend(collect_step_diagnostics(
            &step.filename,
            &scan,
            &available,
            &known_slugs,
        ));
    }

    Ok(diags)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_scan() {
        let dir = tempdir().unwrap();
        let cp = dir.path().to_string_lossy().into_owned();
        create_lecture(cp.clone(), "lec".into(), "en".into(), "T".into()).unwrap();
        let entries = scan_workspace(cp).unwrap();
        assert_eq!(entries[0].slug, "lec");
    }

    #[test]
    fn test_step_lifecycle() {
        let dir = tempdir().unwrap();
        let cp = dir.path().to_string_lossy().into_owned();
        create_lecture(cp.clone(), "lec".into(), "en".into(), "T".into()).unwrap();
        let lp = format!("{cp}/lec");
        create_step(lp.clone(), "en".into(), "s2".into(), "S2".into()).unwrap();
        let yp = format!("{lp}/en/lecture.yaml");
        assert_eq!(load_lecture(yp.clone()).unwrap().lecture.steps.len(), 2);
        let fname = load_lecture(yp.clone()).unwrap().lecture.steps[1].filename.clone();
        rename_step(lp.clone(), "en".into(), fname.clone(), "Renamed".into()).unwrap();
        assert_eq!(load_lecture(yp.clone()).unwrap().lecture.steps[1].title, "Renamed");
        delete_step(lp, "en".into(), fname).unwrap();
        assert_eq!(load_lecture(yp).unwrap().lecture.steps.len(), 1);
    }

    #[test]
    fn test_manifest() {
        let dir = tempdir().unwrap();
        let cp = dir.path().to_string_lossy().into_owned();
        create_lecture(cp.clone(), "a".into(), "en".into(), "A".into()).unwrap();
        create_lecture(cp.clone(), "b".into(), "hu".into(), "B".into()).unwrap();
        let mp = format!("{cp}/manifest.json");
        regenerate_manifest(cp, mp.clone()).unwrap();
        let entries: Vec<LectureEntry> =
            serde_json::from_str(&fs::read_to_string(&mp).unwrap()).unwrap();
        assert_eq!(entries.len(), 2);
    }
}