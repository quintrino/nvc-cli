mod data;
mod lookup;

use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use data::{Catalog, Entry};
use lookup::{Role, Row};
use serde_json::json;

const USAGE: &str = "\
nvc-cli: look up Nonviolent Communication feelings and needs

USAGE:
    nvc-cli [OPTIONS] <WORD>...       Feelings and needs behind a word (faux feelings included)
    nvc-cli [OPTIONS] feelings [FILTER]
    nvc-cli [OPTIONS] needs [FILTER]

OPTIONS:
        --json        Print JSON instead of text
        --no-color    Do not use bold text (also off when NO_COLOR is set or output is piped)
    -h, --help        Print this help
    -V, --version     Print the version

EXAMPLES:
    nvc-cli betrayed
    nvc-cli I feel misunderstood
    nvc-cli feelings angry
    nvc-cli needs connection
";

struct Style {
    bold: bool,
}

impl Style {
    fn bold(&self, text: &str) -> String {
        if self.bold {
            format!("\x1b[1m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

fn entry_json(entry: &Entry, role: Option<Role>) -> serde_json::Value {
    let mut value = json!({
        "kind": entry.kind.as_str(),
        "name": entry.name,
        "emoji": entry.emoji,
        "description": entry.description,
        "group": entry.group,
    });
    if let Some(role) = role {
        value["role"] = json!(role.key());
    }
    value
}

fn print_rows(out: &mut impl Write, rows: &[Row], style: &Style) -> std::io::Result<()> {
    let mut printed: Vec<Role> = Vec::new();
    for row in rows {
        if printed.contains(&row.role) {
            continue;
        }
        printed.push(row.role);
        if row.role == Role::Primary {
            let e = row.entry;
            writeln!(out, "{} {}  ({})", e.emoji, style.bold(&e.name), e.group)?;
            writeln!(out, "   {}", e.description)?;
            continue;
        }
        writeln!(out, "\n{}", style.bold(row.role.heading()))?;
        for item in rows.iter().filter(|r| r.role == row.role) {
            let e = item.entry;
            writeln!(out, "  {} {} — {}", e.emoji, e.name, e.description)?;
        }
    }
    Ok(())
}

fn print_browse(out: &mut impl Write, entries: &[&Entry], style: &Style) -> std::io::Result<()> {
    let mut last_group: Option<&str> = None;
    for e in entries {
        if last_group != Some(e.group.as_str()) {
            if last_group.is_some() {
                writeln!(out)?;
            }
            writeln!(out, "{}", style.bold(&e.group))?;
            last_group = Some(e.group.as_str());
        }
        writeln!(out, "  {} {} — {}", e.emoji, e.name, e.description)?;
    }
    Ok(())
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let mut as_json = false;
    let mut color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let mut words: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("nvc-cli {}", env!("CARGO_PKG_VERSION"));
                return Ok(ExitCode::SUCCESS);
            }
            "--json" => as_json = true,
            "--no-color" => color = false,
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("unknown option: {other} (see --help)"));
            }
            _ => words.push(arg),
        }
    }
    if words.is_empty() {
        print!("{USAGE}");
        return Ok(ExitCode::from(2));
    }

    let catalog = Catalog::load();
    let style = Style { bold: color };
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let io = |e: std::io::Error| e.to_string();

    let subcommand = words[0].to_lowercase();
    let browse_source = match subcommand.as_str() {
        "feelings" => Some(&catalog.feelings),
        "needs" => Some(&catalog.needs),
        _ => None,
    };
    if let Some(source) = browse_source {
        let filter = words[1..].join(" ");
        let entries = lookup::browse(source, &filter);
        if as_json {
            let list: Vec<_> = entries.iter().map(|e| entry_json(e, None)).collect();
            writeln!(out, "{}", json!({ "filter": filter, "results": list })).map_err(io)?;
        } else if entries.is_empty() {
            eprintln!("No {subcommand} matching \"{filter}\"");
            return Ok(ExitCode::from(1));
        } else {
            print_browse(&mut out, &entries, &style).map_err(io)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    let query = words.join(" ");
    let rows = lookup::lookup(&catalog, &query);
    if as_json {
        let list: Vec<_> = rows.iter().map(|r| entry_json(r.entry, Some(r.role))).collect();
        writeln!(out, "{}", json!({ "query": query, "results": list })).map_err(io)?;
    } else if rows.is_empty() {
        eprintln!("No NVC match for \"{query}\". Try a simpler word, e.g. hurt, angry or tired.");
        return Ok(ExitCode::from(1));
    } else {
        print_rows(&mut out, &rows, &style).map_err(io)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("nvc-cli: {message}");
            ExitCode::from(2)
        }
    }
}
