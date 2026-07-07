# clone setup and development 
-clone: 
```bash 
git clone --recurse-submodules https://github.com/gomszab/lecture_core.git
```
-build wasm modules:
```
sh ./build.sh
```
outputs:
|target|path| where to use|
|------|----|------|
|bundler|./pkg/*_bundler postfix| use the wasm as a bundle eg.: in vite|
|web|  ./pkg/* | use wasm as import js eg.: in simple js. |

- clone a lecture repo eg.: [webfejl](https://github.com/gomszab/webfejlesztes2.git) recommendation: clone both repository to the same folder
- test production lecture (recomendation: create to the same folder as the repositories):
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Lecture Workspace Prototype</title>
  <style>
    /* ── Reset & Base ───────────────────────────────────── */
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    html { font-size: 16px; }
    body {
      font-family: "Segoe UI", system-ui, sans-serif;
      background: #0f1117;
      color: #e2e8f0;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }

    /* ── Toolbar ────────────────────────────────────────── */
    #toolbar {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.75rem 1.25rem;
      background: #1a1d27;
      border-bottom: 1px solid #2d3148;
      flex-wrap: wrap;
    }
    #toolbar h1 {
      font-size: 1rem;
      font-weight: 600;
      color: #a78bfa;
      margin-right: 0.5rem;
    }
    select, button {
      padding: 0.35rem 0.75rem;
      border-radius: 6px;
      border: 1px solid #3d4270;
      background: #252840;
      color: #e2e8f0;
      font-size: 0.9rem;
      cursor: pointer;
    }
    button#reload-btn {
      background: #4f46e5;
      border-color: #6366f1;
      font-weight: 600;
    }
    button#reload-btn:hover { background: #4338ca; }
    label { font-size: 0.85rem; color: #94a3b8; }
    #status-bar {
      margin-left: auto;
      font-size: 0.8rem;
      color: #64748b;
      font-style: italic;
    }

    /* ── Main Layout ────────────────────────────────────── */
    #main-layout {
      display: flex;
      flex: 1;
      overflow: hidden;
    }

    /* ── Lecture SPA area ───────────────────────────────── */
    #lecture-area {
      flex: 1;
      overflow-y: auto;
      padding: 0;
    }

    /* Assembled SPA inner structure */
    .lecture-spa {
      display: flex;
      height: 100%;
    }
    .lecture-sidebar {
      width: 220px;
      min-width: 180px;
      background: #141622;
      border-right: 1px solid #2d3148;
      padding: 1.25rem 1rem;
      overflow-y: auto;
      flex-shrink: 0;
    }
    .lecture-sidebar h3 {
      font-size: 0.85rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: #6366f1;
      margin-bottom: 0.75rem;
    }
    .step-nav { list-style: none; }
    .step-nav li { margin-bottom: 0.35rem; }
    .nav-link {
      display: block;
      padding: 0.4rem 0.6rem;
      border-radius: 5px;
      color: #94a3b8;
      text-decoration: none;
      font-size: 0.9rem;
      transition: background 0.15s, color 0.15s;
    }
    .nav-link:hover { background: #252840; color: #e2e8f0; }

    .lecture-content {
      flex: 1;
      padding: 2rem 2.5rem;
      overflow-y: auto;
      max-width: 860px;
    }
    .lecture-step { margin-bottom: 3rem; }
    .step-title {
      font-size: 1.5rem;
      font-weight: 700;
      color: #a78bfa;
      margin-bottom: 1rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid #2d3148;
    }
    .step-body h1, .step-body h2, .step-body h3 {
      color: #c4b5fd;
      margin: 1.25rem 0 0.5rem;
    }
    .step-body p { line-height: 1.7; color: #cbd5e1; margin-bottom: 0.75rem; }
    .step-body a { color: #818cf8; text-decoration: underline; }
    .step-body a:hover { color: #a5b4fc; }
    .step-body ul, .step-body ol { padding-left: 1.5rem; margin-bottom: 0.75rem; color: #cbd5e1; }
    .step-body li { margin-bottom: 0.3rem; }
    .step-body blockquote {
      border-left: 3px solid #4f46e5;
      padding: 0.5rem 1rem;
      background: #1e2235;
      color: #94a3b8;
      border-radius: 0 6px 6px 0;
      margin: 0.75rem 0;
    }
    .step-body code {
      background: #1e2235;
      color: #f472b6;
      padding: 0.15rem 0.4rem;
      border-radius: 4px;
      font-family: "Fira Code", "Cascadia Code", monospace;
      font-size: 0.88em;
    }
    .step-body pre {
      background: #1a1d2e;
      border: 1px solid #2d3148;
      border-radius: 8px;
      padding: 1rem 1.25rem;
      overflow-x: auto;
      margin-bottom: 1rem;
    }
    .step-body pre code {
      background: none;
      color: #e2e8f0;
      padding: 0;
      font-size: 0.9em;
    }
    .step-body table {
      border-collapse: collapse;
      width: 100%;
      margin-bottom: 1rem;
      font-size: 0.9rem;
    }
    .step-body th {
      background: #252840;
      color: #a78bfa;
      padding: 0.5rem 0.75rem;
      text-align: left;
      border: 1px solid #2d3148;
    }
    .step-body td {
      padding: 0.45rem 0.75rem;
      border: 1px solid #2d3148;
      color: #cbd5e1;
    }
    .step-body tr:nth-child(even) td { background: #181b2a; }

    /* Inline diagnostic error blocks from WASM */
    .diagnostic-inline.error {
      background: #2d1b1b;
      border-left: 3px solid #f87171;
      padding: 0.4rem 0.75rem;
      color: #fca5a5;
      border-radius: 0 5px 5px 0;
      font-size: 0.88em;
      margin: 0.5rem 0;
    }

    /* ── Diagnostics Panel ──────────────────────────────── */
    #diagnostics-section {
      padding: 1rem 1.5rem;
      background: #141622;
      border-top: 1px solid #2d3148;
    }
    #diagnostics-section h2 {
      font-size: 0.85rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: #64748b;
      margin-bottom: 0.6rem;
    }
    .diag-list { list-style: none; display: flex; flex-direction: column; gap: 0.4rem; }
    .diag-item {
      display: flex;
      align-items: baseline;
      gap: 0.5rem;
      font-size: 0.85rem;
      background: #1a1d27;
      padding: 0.4rem 0.75rem;
      border-radius: 5px;
      border-left: 3px solid #64748b;
    }
    .diag-item.diag-error { border-color: #f87171; }
    .diag-item.diag-warning { border-color: #fbbf24; }
    .diag-badge {
      font-size: 0.7rem;
      font-weight: 700;
      padding: 0.1rem 0.4rem;
      border-radius: 3px;
      background: #252840;
    }
    .diag-item.diag-error .diag-badge { color: #f87171; }
    .diag-item.diag-warning .diag-badge { color: #fbbf24; }
    .diag-code { color: #818cf8; font-family: monospace; font-size: 0.82rem; }
    .diag-file { color: #64748b; font-style: italic; }
    .diag-msg { color: #94a3b8; flex: 1; }
    .diag-ok { font-size: 0.85rem; color: #4ade80; }
  </style>
</head>
<body>

  <div id="toolbar">
    <h1>📖 Lecture Workspace</h1>
    <label for="lecture-select">Lecture</label>
    <select id="lecture-select"></select>
    <label for="lang-select">Language</label>
    <select id="lang-select"></select>
    <button id="reload-btn">↺ Reload</button>
    <span id="status-bar">Initialising…</span>
  </div>

  <div id="main-layout">
    <div id="lecture-area">
      <p style="padding:2rem;color:#64748b;">Select a lecture above to begin.</p>
    </div>
  </div>

  <div id="diagnostics-section">
    <h2>🩺 Diagnostics</h2>
    <div id="diagnostics-panel"></div>
  </div>

  <script type="module" src="./main.js"></script>
</body>
</html>
```
main.js (replace all [PATHTOLECTURES] to the lecture repository and [PATHTOWASM] to pkg folder of the build output)
```js 
import init_step, {
  render_step_scan,
  render_step_render,
} from "[PATHTOWASM]/step_renderer/step_renderer.js";

import init_assembler, {
  parse_lecture_yaml,
  assemble_lecture_spa,
  collect_diagnostics,
} from "[PATHTOWASM]/lecture_assembler/lecture_assembler.js";

// ─── Bootstrap ────────────────────────────────────────────────────────────────

async function bootstrap() {
  await init_step();
  await init_assembler();

  const manifest = await fetchJSON("./[PATHTOLECTURES]/manifest.json");
  populateLectureSelector(manifest.lectures);

  document
    .getElementById("lecture-select")
    .addEventListener("change", onSelectionChange);
  document
    .getElementById("lang-select")
    .addEventListener("change", loadSelectedLecture);
  document
    .getElementById("reload-btn")
    .addEventListener("click", loadSelectedLecture);

  if (manifest.lectures.length > 0) {
    onSelectionChange();
  }
}

// ─── Selectors ────────────────────────────────────────────────────────────────

function populateLectureSelector(lectures) {
  const sel = document.getElementById("lecture-select");
  sel.innerHTML = "";
  for (const lec of lectures) {
    const opt = document.createElement("option");
    opt.value = lec.slug;
    opt.textContent = lec.title;
    opt.dataset.languages = JSON.stringify(lec.languages);
    sel.appendChild(opt);
  }
}

function onSelectionChange() {
  const sel = document.getElementById("lecture-select");
  const selected = sel.options[sel.selectedIndex];
  const langs = JSON.parse(selected.dataset.languages || '["en"]');

  const langSel = document.getElementById("lang-select");
  langSel.innerHTML = "";
  for (const lang of langs) {
    const opt = document.createElement("option");
    opt.value = lang;
    opt.textContent = lang.toUpperCase();
    langSel.appendChild(opt);
  }

  loadSelectedLecture();
}

// ─── Main Pipeline ────────────────────────────────────────────────────────────

async function loadSelectedLecture() {
  const slug = document.getElementById("lecture-select").value;
  const lang = document.getElementById("lang-select").value;
  const basePath = `[PATHTOLECTURES]/${slug}/${lang}`;

  setStatus("Loading…");
  document.getElementById("lecture-area").innerHTML = "";
  document.getElementById("diagnostics-panel").innerHTML = "";

  try {
    // 1. Fetch lecture.yaml
    const lectureYaml = await fetchText(`${basePath}/lecture.yaml`);

    // 2. Parse lecture metadata
    const lectureMetaJson = parse_lecture_yaml(lectureYaml);
    const lectureMeta = JSON.parse(lectureMetaJson);

    // 3. Fetch each step markdown
    const stepContents = [];
    for (const step of lectureMeta.steps) {
      const md = await fetchText(`${basePath}/steps/${step.filename}`);
      stepContents.push({ ...step, markdown: md });
    }

    // 4. Pass 1 — scan each step for includes and internal links
    const scanResults = [];
    const allIncludePaths = new Set();

    for (const step of stepContents) {
      const scanJson = render_step_scan(step.markdown, basePath);
      const scan = JSON.parse(scanJson);
      scanResults.push({ filename: step.filename, scan });
      for (const inc of scan.includes) {
        allIncludePaths.add(inc.resolved_path);
      }
    }

    // 5. Fetch all include assets — track which ones actually loaded
    const assets = {};
    const loadedAssetPaths = []; // only successfully fetched paths

    for (const path of allIncludePaths) {
      try {
        const content = await fetchText(path);
        assets[path] = content;
        loadedAssetPaths.push(path); // mark as available
      } catch {
        // Missing asset — intentionally not added to loadedAssetPaths
        // so Rust will produce a BROKEN_INCLUDE diagnostic for it
      }
    }

    // 6. Pass 2 — render each step to HTML
    const renderedSteps = [];
    for (const step of stepContents) {
      const assetsJson = JSON.stringify(assets);
      const html = render_step_render(step.markdown, basePath, assetsJson);
      renderedSteps.push({
        slug: step.slug,
        title: step.title,
        filename: step.filename,
        html,
      });
    }

    // 7. Assemble the SPA
    const renderedStepsJson = JSON.stringify(renderedSteps);
    const spaHtml = assemble_lecture_spa(lectureYaml, renderedStepsJson);
    document.getElementById("lecture-area").innerHTML = spaHtml;

    // 8. Collect diagnostics — pass the loaded asset paths as the 4th argument
    //    so Rust can distinguish "include existed" from "include was missing"
    const stepsJson = JSON.stringify(
      stepContents.map((s) => ({ filename: s.filename }))
    );
    const scanResultsJson = JSON.stringify(scanResults);
    const loadedAssetsJson = JSON.stringify(loadedAssetPaths);

    const diagJson = collect_diagnostics(
      lectureYaml,
      stepsJson,
      scanResultsJson,
      loadedAssetsJson  // ← new 4th argument
    );
    const diagnostics = JSON.parse(diagJson);
    renderDiagnostics(diagnostics);

    setStatus(`Loaded: ${lectureMeta.lecture.title} [${lang}]`);

    // Wire up smooth-scroll SPA nav
    wireSpaNav();
  } catch (err) {
    setStatus(`Error: ${err.message}`);
    console.error(err);
  }
}

// ─── Diagnostics Rendering ────────────────────────────────────────────────────

function renderDiagnostics(diagnostics) {
  const panel = document.getElementById("diagnostics-panel");
  if (!diagnostics || diagnostics.length === 0) {
    panel.innerHTML = "<p class=\"diag-ok\">&#10003; No diagnostics &mdash; lecture looks clean.</p>";
    return;
  }

  const items = diagnostics
    .map((d) => {
      const step = d.step_filename
        ? `<span class="diag-file">${d.step_filename}</span>`
        : "";
      return `<li class="diag-item diag-${d.level}">
        <span class="diag-badge">${d.level.toUpperCase()}</span>
        <span class="diag-code">${d.code}</span>
        ${step}
        <span class="diag-msg">${d.message}</span>
      </li>`;
    })
    .join("");

  panel.innerHTML = `<ul class="diag-list">${items}</ul>`;
}

// ─── SPA Navigation ───────────────────────────────────────────────────────────

function wireSpaNav() {
  document.querySelectorAll(".nav-link").forEach((link) => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      const id = link.getAttribute("href").replace("#", "");
      const el = document.getElementById(id);
      if (el) el.scrollIntoView({ behavior: "smooth" });
    });
  });
}

// ─── Utilities ────────────────────────────────────────────────────────────────

async function fetchText(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
  return res.text();
}

async function fetchJSON(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${url}`);
  return res.json();
}

function setStatus(msg) {
  const el = document.getElementById("status-bar");
  if (el) el.textContent = msg;
}

// ─── Run ─────────────────────────────────────────────────────────────────────

bootstrap().catch((err) => {
  console.error("Bootstrap error:", err);
  document.getElementById("status-bar").textContent =
    "Failed to initialise WASM modules.";
});
```
start a live server for host the wasm files and index.html (reason of the recommendations)

- setup lecture editor:
  navigate to the crates/editor
  ```bash
  npm install
  ```
  ```bash
  npm run tauri dev
  ```

# use lecture editor
Open workspace: open the cloned lecture repository folder
(the navigation links are broken in the editor preview)