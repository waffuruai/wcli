# Changelog

Notable changes, newest first. Versions follow [semver](https://semver.org); every release is a `vX.Y.Z` tag on `main`.

## 0.1.5 (2026-11-25)

- The release job runs in the `release` environment that the crates.io trusted publisher requires, so this is the first version published to crates.io by the workflow; no API or behaviour change. 0.1.3 and 0.1.4 were released on GitHub only.

## 0.1.4 (2026-11-25)

- Releases are published to crates.io by the release workflow, through trusted publishing; no API or behaviour change. 0.1.3 was released on GitHub only and is not on crates.io.

## 0.1.3 (2026-11-25)

- The crate is formatted with the org-wide `rustfmt.toml` (stable rustfmt only); no API or behaviour change.
- Builds and CI use Rust 1.100.0 (`rust-toolchain.toml`); `rust-version` stays 1.97.

## 0.1.2 (2026-11-15)

- `Opt::required()` is checked on the command that runs, for its own options: under a subcommand a parent's options are all optional, so a parent whose own run needs `--input` doesn't demand it of `parent verify`.
- `Command::one_of(&[..])`: exactly one of a set of this command's options must be given. None is a usage error naming them all, two is a conflict, and the help marks each with `(one of --a, --b)`.

## 0.1.1 (2026-11-15)

- `Opt::required()`: an option that must be given; leaving it out is a usage error naming it, and the help marks it `(required)`. A flag can't be required, and a required option can't have a default.
- `Matches::count(name)`: how many times a flag was given, so `-vvv` is 3.
- `Command::completions()`: a built-in `completions <shell>` command that prints a completion script for bash, zsh or fish, generated from the command tree; `Command::completion_script(Shell)` for the program's own use. Hidden commands and options are left out; aliases and the built-in `help`, `-h`, `-V` are in.
- `examples/tool.rs`, a small program to try the help and the scripts with.

## 0.1.0 (2026-11-14)

The first release, moved out of [pantry](https://github.com/waffuruai/pantry), where it was `wpantry`'s parser from 2026-11-05:

- A command tree declared from specs written the way help shows them: `--json`, `-o, --out <file>`, `--licenses [file]`, `--tag <name...>`, `<query>`, `[part]`, `<sample...>`.
- Generated usage, help (`-h`, `--help`, `help [command...]`), version (`-V`, `--version`) and errors with suggestions, exit code 2 for a usage error.
- `Opt` choices, defaults, conflicts and hidden options; `Arg` choices and defaults; command aliases, summaries, after-help, required subcommands and shared option sets.
- Typed values through `Matches::get::<T>`, and `Matches::error` for the program's own checks.
- No dependencies, no macros, `unsafe` forbidden.
