//! A small program on wcli, for trying the help and the completion scripts:
//! `cargo run --example tool -- completions bash | bash -n`.

use wcli::{Arg, Command, Opt};

fn main() {
    let program =
        Command::new("tool")
            .version("1.0.0")
            .description("Does things.")
            .option("-v, --verbose", "say more; twice for even more")
            .command(
                Command::new("report")
                    .description("Show the latest report.")
                    .example("tool report -o r.txt", "the findings, to a file")
                    .example("tool report summary --json -o -", "the summary as JSON on stdout")
                    .arg(Arg::new("[part]", "what it covers").choices(&["findings", "summary"]).default("findings"))
                    .opt(Opt::new("--json", "as JSON").group("Output"))
                    .opt(Opt::new("-o, --out <file>", "write it there").required().group("Output"))
                    .opt(Opt::new("--since <days>", "how far back").default("14")),
            )
            .command(Command::new("schedule").subcommand_required().command(
                Command::new("install").opt(Opt::new("--every <period>", "how often").choices(&["day", "week"])),
            ))
            .command(Command::new("export").alias("ex").description("Write everything out."))
            .completions();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let m = program.parse(&argv).unwrap_or_else(|exit| exit.exit());
    println!("{:?} verbose={} {:?}", m.path(), m.count("verbose"), m.value("out"));
}
