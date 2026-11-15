//! Completion scripts for bash, zsh and fish, generated from the command
//! tree: every command's subcommands, aliases, options (its own and its
//! parents'), the choices of its arguments, and the choices of an option's
//! value after that option. Hidden commands and options are left out, the
//! built-in `help`, `-h`, `--help`, `-V`, `--version` and `completions` are
//! in.

use crate::{Command, Opt, Takes};
use std::str::FromStr;

/// What the `completions` command's help says.
pub(crate) const DESCRIPTION: &str = "Print a completion script for bash, zsh or fish. Source it, or save it where the shell loads completions from: bash in ~/.local/share/bash-completion/completions/<name>, zsh as _<name> in a directory on fpath, fish in ~/.config/fish/completions/<name>.fish.";

/// A shell a completion script can be generated for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// The shells by name, as `completions <shell>` accepts them.
    pub const NAMES: [&str; 3] = ["bash", "zsh", "fish"];

    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

impl FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Shell, String> {
        match s {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            other => Err(format!("unknown shell '{other}'; one of {}", Shell::NAMES.join(", "))),
        }
    }
}

/// The script for `shell`, for the program `root`.
pub(crate) fn script(root: &Command, shell: Shell) -> String {
    let mut nodes = Vec::new();
    walk(&[root], &mut nodes);
    match shell {
        Shell::Bash => bash(root, &nodes),
        Shell::Zsh => zsh(root, &nodes),
        Shell::Fish => fish(root, &nodes),
    }
}

/// One command of the tree, with the chain of commands above it.
struct Node<'a> {
    chain: Vec<&'a Command>,
}

impl Node<'_> {
    fn cmd(&self) -> &Command {
        self.chain[self.chain.len() - 1]
    }

    /// `tool report`: the canonical names from the program down.
    fn path(&self) -> String {
        self.chain.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" ")
    }

    fn is_root(&self) -> bool {
        self.chain.len() == 1
    }

    /// The options that work here: the command's own, then its parents'.
    fn options(&self) -> Vec<&Opt> {
        self.chain.iter().rev().flat_map(|c| c.opts.iter()).filter(|o| !o.hidden).collect()
    }

    /// The words that complete here: subcommands and their aliases, the
    /// arguments' choices, every option's spelling, and the built-ins.
    fn words(&self) -> Vec<String> {
        let cmd = self.cmd();
        let mut words = Vec::new();
        for c in cmd.commands.iter().filter(|c| !c.hidden) {
            words.push(c.name.clone());
            words.extend(c.aliases.iter().cloned());
        }
        if !cmd.commands.is_empty() {
            words.push("help".to_string());
        }
        for a in &cmd.args {
            words.extend(a.choices.iter().cloned());
        }
        for o in self.options() {
            words.extend(spellings(o));
        }
        words.push("-h".to_string());
        words.push("--help".to_string());
        if self.is_root() && cmd.version.is_some() {
            words.push("-V".to_string());
            words.push("--version".to_string());
        }
        words
    }
}

/// `-o` and `--out` for an option.
fn spellings(o: &Opt) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(s) = o.short {
        out.push(format!("-{s}"));
    }
    if let Some(l) = &o.long {
        out.push(format!("--{l}"));
    }
    out
}

fn walk<'a>(chain: &[&'a Command], out: &mut Vec<Node<'a>>) {
    out.push(Node { chain: chain.to_vec() });
    for c in chain[chain.len() - 1].commands.iter().filter(|c| !c.hidden) {
        let mut next = chain.to_vec();
        next.push(c);
        walk(&next, out);
    }
}

/// A word in single quotes, for bash, zsh and fish.
fn sq(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The case arms that descend into subcommands: `'tool report'|'tool rep')
/// path="$path report" ;;` for every command below the root, aliases too.
fn descend_arms(nodes: &[Node]) -> String {
    let mut out = String::new();
    for n in nodes.iter().filter(|n| !n.is_root()) {
        let parent = n.chain[..n.chain.len() - 1].iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" ");
        let cmd = n.cmd();
        let names = std::iter::once(&cmd.name).chain(&cmd.aliases);
        let patterns: Vec<String> = names.map(|name| sq(&format!("{parent} {name}"))).collect();
        out.push_str(&format!("            {}) path=\"$path {}\" ;;\n", patterns.join("|"), cmd.name));
    }
    out
}

/// What completes after an option that takes a value: its choices, or
/// files. One arm per spelling of the option, under each command's path.
fn value_arms(nodes: &[Node], choices: impl Fn(&[String]) -> String, files: &str) -> String {
    let mut out = String::new();
    for n in nodes {
        let path = n.path();
        for o in n.options() {
            if o.takes == Takes::Nothing {
                continue;
            }
            let patterns: Vec<String> = spellings(o).iter().map(|s| sq(&format!("{path} {s}"))).collect();
            let action = if o.choices.is_empty() { files.to_string() } else { choices(&o.choices) };
            out.push_str(&format!("        {}) {action}; return ;;\n", patterns.join("|")));
        }
    }
    out
}

fn bash(root: &Command, nodes: &[Node]) -> String {
    let name = &root.name;
    let func = format!("_{}", name.replace(|c: char| !c.is_ascii_alphanumeric(), "_"));
    let mut out = format!(
        "# bash completion for {name}, generated by wcli ({name} completions bash).\n# Source it, or save it as ~/.local/share/bash-completion/completions/{name}.\n{func}() {{\n    local cur prev path={} i word\n    COMPREPLY=()\n    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n    prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n    for ((i = 1; i < COMP_CWORD; i++)); do\n        word=\"${{COMP_WORDS[i]}}\"\n        case \"$path $word\" in\n",
        sq(name)
    );
    out.push_str(&descend_arms(nodes));
    out.push_str("        esac\n    done\n    case \"$path $prev\" in\n");
    out.push_str(&value_arms(
        nodes,
        |c| format!("COMPREPLY=($(compgen -W {} -- \"$cur\"))", sq(&c.join(" "))),
        "COMPREPLY=($(compgen -f -- \"$cur\"))",
    ));
    out.push_str("    esac\n    case \"$path\" in\n");
    for n in nodes {
        out.push_str(&format!(
            "        {}) COMPREPLY=($(compgen -W {} -- \"$cur\")) ;;\n",
            sq(&n.path()),
            sq(&n.words().join(" "))
        ));
    }
    out.push_str(&format!("    esac\n}}\ncomplete -F {func} {name}\n"));
    out
}

fn zsh(root: &Command, nodes: &[Node]) -> String {
    let name = &root.name;
    let func = format!("_{}", name.replace(|c: char| !c.is_ascii_alphanumeric(), "_"));
    let mut out = format!(
        "#compdef {name}\n# zsh completion for {name}, generated by wcli ({name} completions zsh).\n# Source it, or save it as _{name} in a directory on fpath.\n{func}() {{\n    local cur prev path={} i word\n    cur=\"${{words[CURRENT]}}\"\n    prev=\"${{words[CURRENT-1]}}\"\n    for ((i = 2; i < CURRENT; i++)); do\n        word=\"${{words[i]}}\"\n        case \"$path $word\" in\n",
        sq(name)
    );
    out.push_str(&descend_arms(nodes));
    out.push_str("        esac\n    done\n    case \"$path $prev\" in\n");
    out.push_str(&value_arms(
        nodes,
        |c| format!("compadd -- {}", c.iter().map(|w| sq(w)).collect::<Vec<_>>().join(" ")),
        "_files",
    ));
    out.push_str("    esac\n    case \"$path\" in\n");
    for n in nodes {
        let words: Vec<String> = n.words().iter().map(|w| sq(w)).collect();
        out.push_str(&format!("        {}) compadd -- {} ;;\n", sq(&n.path()), words.join(" ")));
    }
    out.push_str(&format!(
        "    esac\n}}\nif [ \"${{funcstack[1]}}\" = {func} ]; then\n    {func} \"$@\"\nelse\n    compdef {func} {name}\nfi\n"
    ));
    out
}

fn fish(root: &Command, nodes: &[Node]) -> String {
    let name = &root.name;
    let mut out = format!(
        "# fish completion for {name}, generated by wcli ({name} completions fish).\n# Save it as ~/.config/fish/completions/{name}.fish.\ncomplete -c {name} -f\n"
    );
    for n in nodes {
        let cmd = n.cmd();
        // Where we are: every command named on the way down, and none of
        // this command's own subcommands yet.
        let mut seen: Vec<String> =
            n.chain.iter().skip(1).map(|c| format!("__fish_seen_subcommand_from {}", c.name)).collect();
        if n.is_root() {
            seen.push("__fish_use_subcommand".to_string());
        }
        let here = seen.join("; and ");
        let subs: Vec<&str> = cmd.commands.iter().filter(|c| !c.hidden).map(|c| c.name.as_str()).collect();
        let before_subs = if subs.is_empty() || n.is_root() {
            here.clone()
        } else {
            format!("{here}; and not __fish_seen_subcommand_from {}", subs.join(" "))
        };
        for c in cmd.commands.iter().filter(|c| !c.hidden) {
            let names = std::iter::once(&c.name).chain(&c.aliases).cloned().collect::<Vec<_>>().join(" ");
            out.push_str(&format!(
                "complete -c {name} -n {} -a {} -d {}\n",
                sq(&before_subs),
                sq(&names),
                sq(&crate::help::summary(c))
            ));
        }
        if !cmd.commands.is_empty() {
            out.push_str(&format!(
                "complete -c {name} -n {} -a help -d {}\n",
                sq(&before_subs),
                sq("Print a command's help")
            ));
        }
        for a in &cmd.args {
            if !a.choices.is_empty() {
                out.push_str(&format!("complete -c {name} -n {} -a {}\n", sq(&here), sq(&a.choices.join(" "))));
            }
        }
        // This command's own options; the parents' were written with them.
        for o in cmd.opts.iter().filter(|o| !o.hidden) {
            let mut line = format!("complete -c {name}");
            if !n.is_root() {
                line.push_str(&format!(" -n {}", sq(&here)));
            }
            if let Some(s) = o.short {
                line.push_str(&format!(" -s {s}"));
            }
            if let Some(l) = &o.long {
                line.push_str(&format!(" -l {l}"));
            }
            match &o.takes {
                Takes::Nothing => {}
                Takes::Value { .. } if o.choices.is_empty() => line.push_str(" -r -F"),
                Takes::Value { .. } => line.push_str(&format!(" -x -a {}", sq(&o.choices.join(" ")))),
            }
            line.push_str(&format!(" -d {}\n", sq(&o.help)));
            out.push_str(&line);
        }
        let builtin = if n.is_root() { String::new() } else { format!(" -n {}", sq(&here)) };
        out.push_str(&format!("complete -c {name}{builtin} -s h -l help -d {}\n", sq("print this help")));
        if n.is_root() && cmd.version.is_some() {
            out.push_str(&format!("complete -c {name} -s V -l version -d {}\n", sq("print the version")));
        }
    }
    out
}

#[cfg(test)]
#[path = "completions_tests.rs"]
mod tests;
