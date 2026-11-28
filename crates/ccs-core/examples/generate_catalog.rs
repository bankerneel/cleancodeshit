//! Regenerate the item reference tables and the landing page's static item list.
//!
//! ```text
//! cargo run --locked -p ccs-core --example generate_catalog
//! cargo run --locked -p ccs-core --example generate_catalog -- --check
//! ```
//!
//! The Bash executable is still the release source. Both modes therefore also
//! compare every legacy item's id, group, risk and label with its `add` line in
//! `cleancodeshit`, including the indented, conditional `stale` registration.
//!
//! Outputs are deliberately narrow:
//!   * `docs/items.md` - only the content between the paired
//!     `<!-- ccs-catalog:start:KEY -->` / `<!-- ccs-catalog:end:KEY -->` markers.
//!   * `site/assets/app.js` - only the `  var CLI_ITEMS = [ ... ];` block.
//!
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CLI_ITEMS_START: &str = "  var CLI_ITEMS = [";
const CLI_ITEMS_END: &str = "\n  ];";
const MARKER_START: &str = "<!-- ccs-catalog:start:";
const MARKER_END: &str = "<!-- ccs-catalog:end:";
const WINDOWS_SECTION: &str = "Windows planning";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Legacy {
    group: String,
    risk: String,
    label: String,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let check = match args.as_slice() {
        [] => false,
        [only] if only == "--check" => true,
        _ => {
            eprintln!("usage: generate_catalog [--check]");
            return ExitCode::from(2);
        }
    };

    match generate(check) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("generate_catalog: {message}");
            ExitCode::FAILURE
        }
    }
}

fn generate(check: bool) -> Result<(), String> {
    let root = workspace_root();
    let docs_path = root.join("docs").join("items.md");
    let site_path = root.join("site").join("assets").join("app.js");
    let bash_path = root.join("cleancodeshit");

    let catalog = ccs_core::Catalog::builtin().map_err(|error| error.to_string())?;
    let docs_before = read_file(&docs_path)?;
    let site_before = read_file(&site_path)?;
    let bash_source = read_file(&bash_path)?;

    let legacy = parse_add_lines(&bash_source)?;
    let parity_problems = check_legacy_parity(&catalog, &legacy);
    let docs_after = render_docs(&docs_before, &catalog)?;
    let site_after = render_site(&site_before, &catalog)?;

    let docs_drift = docs_after != docs_before;
    let site_drift = site_after != site_before;

    if check {
        let mut problems = parity_problems;
        if docs_drift {
            problems.push(format!("{} is out of date", docs_path.display()));
            problems.push(short_diff(&docs_before, &docs_after));
        }
        if site_drift {
            problems.push(format!("{} is out of date", site_path.display()));
            problems.push(short_diff(&site_before, &site_after));
        }
        if problems.is_empty() {
            println!("catalog projections and Bash registrations are in sync");
            Ok(())
        } else {
            for problem in &problems {
                eprintln!("{problem}");
            }
            Err(format!("--check found {} problem(s)", problems.len()))
        }
    } else {
        // Never write a projection that contradicts the release source.
        if !parity_problems.is_empty() {
            for problem in &parity_problems {
                eprintln!("{problem}");
            }
            return Err(
                "Bash registrations do not match the catalog; no files written".to_string(),
            );
        }
        if docs_drift {
            fs::write(&docs_path, &docs_after)
                .map_err(|error| format!("writing {}: {error}", docs_path.display()))?;
            println!("updated {}", docs_path.display());
        }
        if site_drift {
            fs::write(&site_path, &site_after)
                .map_err(|error| format!("writing {}: {error}", site_path.display()))?;
            println!("updated {}", site_path.display());
        }
        if !docs_drift && !site_drift {
            println!("catalog projections are already current");
        }
        Ok(())
    }
}

/// `CARGO_MANIFEST_DIR` is `<workspace>/crates/ccs-core`; use native paths so
/// the same code resolves the repository root on Windows too.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("ccs-core must be two levels below the workspace root")
        .to_path_buf()
}

fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))
}

fn newline(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Minimal shell word splitter for `add` lines: double quotes group spaces, and
/// variables such as `$UV_PLAN` stay a single literal token.
fn shell_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut started = false;

    for character in line.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                started = true;
            }
            whitespace if whitespace.is_whitespace() && !in_quotes => {
                if started {
                    tokens.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            other => {
                current.push(other);
                started = true;
            }
        }
    }
    if started {
        tokens.push(current);
    }
    tokens
}

fn parse_add_lines(source: &str) -> Result<BTreeMap<String, Legacy>, String> {
    let mut entries = BTreeMap::new();
    for (index, raw) in source.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let tokens = shell_tokens(trimmed);
        if tokens.first().map(String::as_str) != Some("add") {
            continue;
        }
        if tokens.len() < 5 {
            return Err(format!(
                "cleancodeshit:{}: add line has fewer than five fields",
                index + 1
            ));
        }
        let id = tokens[1].clone();
        let entry = Legacy {
            group: tokens[2].clone(),
            risk: tokens[3].clone(),
            label: tokens[4].clone(),
        };
        if entries.insert(id.clone(), entry).is_some() {
            return Err(format!(
                "cleancodeshit:{}: duplicate add line for `{id}`",
                index + 1
            ));
        }
    }
    Ok(entries)
}

fn check_legacy_parity(
    catalog: &ccs_core::Catalog,
    legacy: &BTreeMap<String, Legacy>,
) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();

    for item in &catalog.items {
        seen.insert(item.id.as_str());
        let Some(expected) = legacy.get(&item.id) else {
            problems.push(format!(
                "Bash has no add line for catalog item `{}`",
                item.id
            ));
            continue;
        };
        if expected.group != item.group {
            problems.push(format!(
                "`{}` group: Bash `{}`, catalog `{}`",
                item.id, expected.group, item.group
            ));
        }
        if expected.risk != item.risk.as_str() {
            problems.push(format!(
                "`{}` risk: Bash `{}`, catalog `{}`",
                item.id,
                expected.risk,
                item.risk.as_str()
            ));
        }
        if expected.label != item.label {
            problems.push(format!(
                "`{}` label: Bash `{}`, catalog `{}`",
                item.id, expected.label, item.label
            ));
        }
    }

    for id in legacy.keys() {
        if !seen.contains(id.as_str()) {
            problems.push(format!("catalog has no item for Bash add `{id}`"));
        }
    }
    problems
}

fn sources_text<'a>(
    sources: impl Iterator<Item = (&'a str, &'a str)>,
    note: Option<&str>,
) -> String {
    let mut text = sources
        .map(|(label, url)| format!("[{label}]({url})"))
        .collect::<Vec<_>>()
        .join(", ");
    if let Some(note) = note.filter(|note| !note.is_empty()) {
        if !text.is_empty() {
            text.push_str(", ");
        }
        text.push_str(note);
    }
    text
}

fn render_docs(source: &str, catalog: &ccs_core::Catalog) -> Result<String, String> {
    let nl = newline(source);
    let mut sections: Vec<String> = Vec::new();
    let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, item) in catalog.items.iter().enumerate() {
        let section = item.docs.section.clone();
        if !grouped.contains_key(&section) {
            sections.push(section.clone());
        }
        grouped.entry(section).or_default().push(index);
    }

    let mut regions: Vec<(String, String)> = Vec::new();
    for section in sections {
        let mut block = String::new();
        block.push_str(match section.as_str() {
            "Docker" => "| ID | what it targets | Action | Risk | Official source |",
            "Opt-in stale projects" => "| ID | Path | Action | Risk | Source |",
            _ => "| ID | Path | Action | Risk | Official source |",
        });
        block.push_str(nl);
        block.push_str("| --- | --- | --- | --- | --- |");
        block.push_str(nl);
        for index in &grouped[&section] {
            let item = &catalog.items[*index];
            let sources = sources_text(
                item.docs
                    .sources
                    .iter()
                    .map(|source| (source.label.as_str(), source.url.as_str())),
                item.docs.source_note.as_deref(),
            );
            block.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |{}",
                item.id,
                item.docs.path,
                item.docs.action,
                item.risk.as_str(),
                sources,
                nl
            ));
        }
        regions.push((section, block));
    }

    // Planning only: Windows entries carry a catalog status, never an action
    // claim. The surrounding prose in items.md states this explicitly.
    let mut windows = String::new();
    windows.push_str(
        "| ID | Group | Risk | Status | Path | Planned action or reason | Official source |",
    );
    windows.push_str(nl);
    windows.push_str("| --- | --- | --- | --- | --- | --- | --- |");
    windows.push_str(nl);
    for item in &catalog.windows {
        let sources = sources_text(
            item.docs
                .sources
                .iter()
                .map(|source| (source.label.as_str(), source.url.as_str())),
            item.docs.source_note.as_deref(),
        );
        windows.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |{}",
            item.id,
            item.group,
            item.risk.as_str(),
            item.status.as_str(),
            markdown_text(&item.docs.path),
            markdown_text(&item.docs.action),
            sources,
            nl
        ));
    }
    regions.push((WINDOWS_SECTION.to_string(), windows));

    replace_regions(source, &regions)
}

fn marker_key<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let body = line.strip_prefix(prefix)?;
    body.strip_suffix(" -->")
}

fn markdown_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn replace_regions(source: &str, regions: &[(String, String)]) -> Result<String, String> {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut output = String::with_capacity(source.len());
    let mut applied: HashMap<&str, usize> = HashMap::new();
    let mut index = 0;

    while index < lines.len() {
        let current = lines[index].trim_end_matches(['\r', '\n']);
        if let Some(key) = marker_key(current, MARKER_START) {
            let mut end = index + 1;
            while end < lines.len()
                && marker_key(lines[end].trim_end_matches(['\r', '\n']), MARKER_END) != Some(key)
            {
                end += 1;
            }
            if end == lines.len() {
                return Err(format!("docs/items.md: missing end marker for `{key}`"));
            }
            output.push_str(lines[index]);
            match regions.iter().find(|(section, _)| section == key) {
                Some((_, block)) => output.push_str(block),
                None => return Err(format!("docs/items.md: no catalog section for `{key}`")),
            }
            output.push_str(lines[end]);
            *applied.entry(key).or_default() += 1;
            index = end + 1;
            continue;
        }
        output.push_str(lines[index]);
        index += 1;
    }

    for (section, _) in regions {
        if applied.get(section.as_str()).copied().unwrap_or(0) != 1 {
            return Err(format!(
                "docs/items.md: expected exactly one marker pair for `{section}`"
            ));
        }
    }
    Ok(output)
}

fn render_site(source: &str, catalog: &ccs_core::Catalog) -> Result<String, String> {
    let start = source
        .find(CLI_ITEMS_START)
        .ok_or("site/assets/app.js: missing CLI_ITEMS declaration")?;
    let tail = &source[start..];
    let end_offset = tail
        .find(CLI_ITEMS_END)
        .ok_or("site/assets/app.js: missing CLI_ITEMS closing `];`")?;
    let end = start + end_offset + CLI_ITEMS_END.len();

    let nl = newline(source);
    let mut block = String::new();
    block.push_str(CLI_ITEMS_START);
    block.push_str(nl);
    let last = catalog.items.len().saturating_sub(1);
    for (index, item) in catalog.items.iter().enumerate() {
        let cells = [
            item.id.as_str(),
            item.group.as_str(),
            item.risk.as_str(),
            item.label.as_str(),
        ];
        let rendered = cells
            .iter()
            .map(|cell| serde_json::to_string(cell).expect("string escaping cannot fail"))
            .collect::<Vec<_>>()
            .join(", ");
        block.push_str("    [");
        block.push_str(&rendered);
        block.push(']');
        if index != last {
            block.push(',');
        }
        block.push_str(nl);
    }
    block.push_str("  ];");

    let mut output = String::with_capacity(source.len());
    output.push_str(&source[..start]);
    output.push_str(&block);
    output.push_str(&source[end..]);
    Ok(output)
}

fn short_diff(before: &str, after: &str) -> String {
    let old_lines: Vec<&str> = before.lines().collect();
    let new_lines: Vec<&str> = after.lines().collect();
    let mut first = 0;
    while first < old_lines.len() && first < new_lines.len() && old_lines[first] == new_lines[first]
    {
        first += 1;
    }
    let mut tail = 0;
    while tail < old_lines.len() - first
        && tail < new_lines.len() - first
        && old_lines[old_lines.len() - 1 - tail] == new_lines[new_lines.len() - 1 - tail]
    {
        tail += 1;
    }
    let start = first.saturating_sub(2);
    let old_end = (old_lines.len() - tail).min(first + 6);
    let new_end = (new_lines.len() - tail).min(first + 6);
    let mut output = format!(
        "  @@ -{},{} +{},{} @@",
        first + 1,
        old_end - start,
        first + 1,
        new_end - start
    );
    for line in &old_lines[start..old_end] {
        output.push_str(&format!("\n  -{line}"));
    }
    for line in &new_lines[start..new_end] {
        output.push_str(&format!("\n  +{line}"));
    }
    output
}
