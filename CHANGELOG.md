# Changelog

Notable changes, newest first. Versions follow [semver](https://semver.org); every release is a `vX.Y.Z` tag on `main`.

## 0.1.0 (2026-11-14)

The first release, moved out of [pantry](https://github.com/waffuruai/pantry), where it was `wpantry`'s parser from 2026-11-05:

- A command tree declared from specs written the way help shows them: `--json`, `-o, --out <file>`, `--licenses [file]`, `--tag <name...>`, `<query>`, `[part]`, `<sample...>`.
- Generated usage, help (`-h`, `--help`, `help [command...]`), version (`-V`, `--version`) and errors with suggestions, exit code 2 for a usage error.
- `Opt` choices, defaults, conflicts and hidden options; `Arg` choices and defaults; command aliases, summaries, after-help, required subcommands and shared option sets.
- Typed values through `Matches::get::<T>`, and `Matches::error` for the program's own checks.
- No dependencies, no macros, `unsafe` forbidden.
