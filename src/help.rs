//! The usage line and the help, generated from the declarations: what a
//! command does, its arguments, options and subcommands, in commander's
//! layout, wrapped to [`WIDTH`] columns.

use crate::{Command, Takes};

/// The help's width in columns: fixed, so help reads the same everywhere
/// (and in the docs it is copied into).
pub const WIDTH: usize = 100;
/// The widest the term column grows; a longer term puts its text below it.
const TERM: usize = 30;

/// `Usage: tool report [options] [part]` for the command at the end of
/// `chain`.
pub fn usage(chain: &[&Command]) -> String {
    let cmd = chain[chain.len() - 1];
    let mut line = String::from("Usage:");
    for c in chain {
        line.push(' ');
        line.push_str(&c.name);
    }
    line.push_str(" [options]");
    if !cmd.commands.is_empty() {
        line.push_str(if cmd.needs_command { " <command>" } else { " [command]" });
    }
    for a in &cmd.args {
        line.push(' ');
        line.push_str(&a.spec());
    }
    line
}

/// `text` wrapped to `width` columns, every line after the first indented
/// by `indent`. A line of `text` that starts with spaces keeps them, and
/// what wraps from it starts under its first word.
fn wrap(text: &str, indent: usize, width: usize) -> String {
    let mut out = String::new();
    for (n, line) in text.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
        }
        let para = line.trim_start_matches(' ');
        let lead = line.len() - para.len();
        out.push_str(&" ".repeat(lead));
        let indent = indent + lead;
        let room = width.saturating_sub(indent).max(20);
        let mut col = 0;
        for word in para.split(' ') {
            let len = word.chars().count();
            if col > 0 && col + 1 + len > room {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                col = 0;
            } else if col > 0 {
                out.push(' ');
                col += 1;
            }
            out.push_str(word);
            col += len;
        }
    }
    out
}

/// A section's rows: the term, then its text in a column.
fn rows(title: &str, rows: &[(String, String)]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let col = rows.iter().map(|(t, _)| t.chars().count()).filter(|n| *n <= TERM).max().unwrap_or(TERM);
    let mut out = format!("\n{title}:\n");
    for (term, text) in rows {
        let len = term.chars().count();
        out.push_str("  ");
        out.push_str(term);
        if text.is_empty() {
            out.push('\n');
            continue;
        }
        if len > col {
            out.push('\n');
            out.push_str(&" ".repeat(col + 4));
        } else {
            out.push_str(&" ".repeat(col - len + 2));
        }
        out.push_str(&wrap(text, col + 4, WIDTH));
        out.push('\n');
    }
    out
}

/// What the help adds after an option's or argument's text.
fn extras(choices: &[String], default: Option<&str>, required: bool, one_of: Option<&[String]>) -> String {
    let mut bits = Vec::new();
    if required {
        bits.push("required".to_string());
    }
    if let Some(set) = one_of {
        let names: Vec<String> = set.iter().map(|n| format!("--{n}")).collect();
        bits.push(format!("one of {}", names.join(", ")));
    }
    if !choices.is_empty() {
        bits.push(format!("choices: {}", choices.join(", ")));
    }
    if let Some(d) = default {
        bits.push(format!("default: {d}"));
    }
    if bits.is_empty() { String::new() } else { format!(" ({})", bits.join("; ")) }
}

/// A command's line in its parent's list: its summary, else the first
/// sentence of its description.
pub(crate) fn summary(c: &Command) -> String {
    if let Some(s) = &c.summary {
        return s.clone();
    }
    let first = c.description.split('\n').next().unwrap_or_default();
    match first.find(". ") {
        Some(i) => first[..i].to_string(),
        None => first.trim_end_matches('.').to_string(),
    }
}

/// The help of the command at the end of `chain`.
pub fn help(chain: &[&Command]) -> String {
    let cmd = chain[chain.len() - 1];
    let mut out = usage(chain);
    out.push('\n');
    if !cmd.description.is_empty() {
        out.push('\n');
        out.push_str(&wrap(&cmd.description, 0, WIDTH));
        out.push('\n');
    }
    if !cmd.aliases.is_empty() {
        out.push_str(&format!("\nAlso: {}\n", cmd.aliases.join(", ")));
    }
    out.push_str(&rows("Examples", &cmd.examples));
    let args: Vec<(String, String)> = cmd
        .args
        .iter()
        .map(|a| (a.name.clone(), format!("{}{}", a.help, extras(&a.choices, a.default.as_deref(), false, None))))
        .collect();
    out.push_str(&rows("Arguments", &args));
    let row = |o: &crate::Opt| {
        let default = match o.takes {
            Takes::Value { .. } => o.default.as_deref(),
            Takes::Nothing => None,
        };
        let set = cmd.one_of.iter().find(|s| s.contains(&o.name())).map(Vec::as_slice);
        (o.flags(), format!("{}{}", o.help, extras(&o.choices, default, o.required, set)))
    };
    // Each group's options under its own title, in the order the groups
    // first appear; the rest, with --help, under `Options:` after them.
    let shown: Vec<&crate::Opt> = cmd.opts.iter().filter(|o| !o.hidden).collect();
    let mut groups: Vec<&str> = Vec::new();
    for g in shown.iter().filter_map(|o| o.group.as_deref()) {
        if !groups.contains(&g) {
            groups.push(g);
        }
    }
    for g in groups {
        let of: Vec<(String, String)> =
            shown.iter().filter(|o| o.group.as_deref() == Some(g)).map(|o| row(o)).collect();
        out.push_str(&rows(&format!("{g} options"), &of));
    }
    let mut opts: Vec<(String, String)> = shown.iter().filter(|o| o.group.is_none()).map(|o| row(o)).collect();
    if chain.len() == 1 && cmd.version.is_some() {
        opts.push(("-V, --version".into(), "print the version".into()));
    }
    opts.push(("-h, --help".into(), "print this help".into()));
    out.push_str(&rows("Options", &opts));
    let mut cmds: Vec<(String, String)> = cmd
        .commands
        .iter()
        .filter(|c| !c.hidden)
        .map(|c| {
            let mut term = c.name.clone();
            for a in &c.aliases {
                term.push('|');
                term.push_str(a);
            }
            if !c.opts.is_empty() {
                term.push_str(" [options]");
            }
            if !c.commands.is_empty() {
                term.push_str(if c.needs_command { " <command>" } else { " [command]" });
            }
            for a in &c.args {
                term.push(' ');
                term.push_str(&a.spec());
            }
            (term, summary(c))
        })
        .collect();
    if !cmds.is_empty() {
        cmds.push(("help [command]".into(), "Print a command's help".into()));
    }
    out.push_str(&rows("Commands", &cmds));
    if !cmd.after.is_empty() {
        out.push('\n');
        out.push_str(&wrap(&cmd.after, 0, WIDTH));
        out.push('\n');
    }
    out
}

#[cfg(test)]
#[path = "help_tests.rs"]
mod tests;
