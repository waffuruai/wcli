# wcli

[![CI](https://github.com/waffuruai/wcli/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/waffuruai/wcli/actions/workflows/ci.yml)

Command-line parsing for Waffuru's Rust tools, inspired by [commander.js](https://github.com/tj/commander.js). You declare each command once, with options and arguments written the way its help shows them. From that, wcli generates the parser, the usage line, the help, and the errors. It has no dependencies and no macros. [`wpantry`](https://github.com/waffuruai/pantry) and others are built on it.

```rust
use wcli::{Arg, Command, Opt};

let program = Command::new("tool")
    .version("1.0.0")
    .description("Does things.")
    .command(
        Command::new("report")
            .description("Show the latest report.")
            .arg(Arg::new("[part]", "what it covers").choices(&["findings", "summary"]).default("findings"))
            .option("--json", "as JSON")
            .option("-o, --out <file>", "write it there")
            .opt(Opt::new("--since <days>", "how far back").default("14")),
    )
    .command(
        Command::new("schedule")
            .subcommand_required()
            .command(Command::new("install").opt(Opt::new("--every <period>", "how often").choices(&["day", "week"]))),
    );

let argv: Vec<String> = std::env::args().skip(1).collect();
let m = program.parse(&argv).unwrap_or_else(|exit| exit.exit()); // help, version or a usage error
match m.path().as_slice() {
    ["report"] => {
        let since: u32 = m.get("since").unwrap_or_else(|e| e.exit()).unwrap_or(14);
        println!("{:?} {} {since}", m.arg("part"), m.flag("json"));
    }
    ["schedule", "install"] => println!("{:?}", m.value("every")),
    _ => unreachable!(),
}
```

## Specs

| Spec | Means |
|---|---|
| `--json` | A flag: `m.flag("json")` |
| `-o, --out <file>` | Takes a value: `-o x`, `-ox`, `--out x`, `--out=x`. The next word is always the value, even `-` |
| `--licenses [file]` | The value may be left out: `m.flag("licenses")` says it was given, and `m.value("licenses")` gives the value if there is one |
| `--tag <name...>` | Takes the words after it until the next option. Every option also collects a value each time it is repeated: `m.values("tag")` |
| `<query>`, `[part]`, `<sample...>` | Arguments: required, optional, and the rest of the words. Read them with `m.arg("query")` and `m.args("sample")` |

`Opt` adds `.choices(&[..])`, `.default(..)`, `.conflicts(&[..])` (options that can't be given together), `.required()` (leaving it out when its command runs is a usage error; the help says `(required)`; under a subcommand a parent's options are all optional) and `.hide()`. `Arg` adds `.choices` and `.default`. `Command` adds:

- `.alias(..)`
- `.summary(..)`, the command's line in its parent's list; without it, the first sentence of the description
- `.after_help(..)`
- `.subcommand_required()`
- `.one_of(&[..])`, options of this command of which exactly one must be given: none is a usage error naming them, two is a conflict; the help marks each with the set
- `.hide()`
- `.opts(..)`, a set of options several commands share
- `.completions()`, on the program: a `completions <shell>` command that prints a completion script for bash, zsh or fish, generated from the command tree (subcommands, aliases, options, choices); `completion_script(Shell)` gives the same text to the program

## What you get

- `-h` and `--help` on every command.
- `help [command...]` on every command with subcommands.
- `-V` and `--version` on the root when it has a version.
- With no words at all, the root's help.
- Errors in one shape: the usage line, then `error: …`, then `(Did you mean --json?)` when a name is close. They come back as `Exit::Error`, exit code 2. Help and the version come back as `Exit::Help` and `Exit::Version`, exit code 0. `exit.exit()` prints each where it belongs and ends the process.
- `m.get::<T>(name)` parses a value, and reports an option's bad value as a usage error.
- `m.count(name)` is how many times a flag was given: `-vvv` is 3.
- `m.error(msg)` makes a usage error for checks only the program can do.
- No abbreviations. An option must be spelled out in full; a typo gets a suggestion instead.
- Help that wraps to 100 columns, so it reads the same everywhere it is copied.

A complete program is [`examples/tool.rs`](examples/tool.rs): `cargo run --example tool -- --help`, and `cargo run --example tool -- completions bash | bash -n` checks a script.

## Install

From [crates.io](https://crates.io/crates/wcli):

```toml
[dependencies]
wcli = "0.1.0"
```

Waffuru's own tools pin it exactly (`wcli = "=0.1.0"`) and move on together. The crate needs Rust 1.97 or newer and nothing else.

## Development

The toolchain is the one [`rust-toolchain.toml`](rust-toolchain.toml) pins; [`mise.toml`](mise.toml) holds the tasks, and `mise tasks ls` lists them:

| Task | What it runs |
|---|---|
| `build` | `cargo build --locked` |
| `test` | `cargo test --locked`: the unit tests in `src/*_tests.rs` and the doc examples |
| `lint` | rustfmt, clippy with warnings denied, and the docs with broken links denied (what CI runs) |
| `coverage` | Line coverage of the tests with cargo-llvm-cov; fails under 90% |
| `package` | What crates.io would receive, built from the packaged sources alone |

Branches: pull requests go into `dev` (squash); `main` takes `dev` by a merge or rebase PR. Dependencies: none, and that is a feature; a change that needs one is a design discussion first.

## Releasing

1. Bump `version` in [`Cargo.toml`](Cargo.toml) and add the entry to [`CHANGELOG.md`](CHANGELOG.md), by PR into `dev`.
2. Merge `dev` into `main` by PR.
3. Tag `main` and push the tag:

   ```bash
   git tag vX.Y.Z && git push origin vX.Y.Z
   ```

[`release.yml`](.github/workflows/release.yml) checks the tag is on `main` and matches `Cargo.toml`, runs the tests, packages the crate and publishes a GitHub Release with generated notes and the `.crate` file. Publishing to crates.io is by hand for now, from a checkout of the tag: `cargo publish --locked`, with a token that can publish updates. Then bump the pin in each dependent.

## License

[MIT](LICENSE).
