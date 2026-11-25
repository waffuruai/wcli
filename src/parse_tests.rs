use super::*;
use crate::{Arg, Opt};

fn program() -> Command {
    let formats = ["txt", "json", "md"];
    let format = |f: &str| {
        let others: Vec<&str> = formats.iter().copied().filter(|o| *o != f).collect();
        Opt::new(&format!("--{f}"), f).conflicts(&others)
    };
    Command::new("tool")
        .version("1.2.3")
        .option("-q, --quiet", "say less")
        .command(
            Command::new("report")
                .arg(Arg::new("[part]", "").choices(&["findings", "summary"]).default("findings"))
                .opts(formats.map(format))
                .option("-o, --out <file>", "")
                .opt(Opt::new("--since <days>", "").default("14"))
                .option("--licenses [file]", "")
                .option("--path <dir...>", "")
                .option("-y, --yes", ""),
        )
        .command(Command::new("search").argument("<query>", "").option("--json", ""))
        .command(
            Command::new("rules")
                .option("--update", "")
                .command(Command::new("gen").argument("<sample...>", "").option("--name <rule>", "")),
        )
        .command(Command::new("schedule").subcommand_required().command(
            Command::new("install").opt(Opt::new("--every <period>", "").choices(&["day", "week"]).default("day")),
        ))
        .command(Command::new("export").alias("ex"))
}

fn ok(argv: &[&str]) -> Matches {
    program().parse(argv).unwrap_or_else(|e| panic!("{argv:?}: {}", e.text()))
}

fn err(argv: &[&str]) -> String {
    match program().parse(argv) {
        Err(Exit::Error(t)) => t,
        other => panic!("{argv:?}: {other:?}"),
    }
}

#[test]
fn options_values_and_arguments() {
    let m = ok(&["report", "summary", "--json", "-o", "-", "--since=30", "--path", "a", "b", "--path", "c"]);
    assert_eq!((m.command(), m.path()), ("report", vec!["report"]));
    assert_eq!(m.arg("part"), Some("summary"));
    assert!(m.flag("json") && !m.flag("md"));
    assert_eq!(m.value("out"), Some("-"), "a required value is the next word, a dash too");
    assert_eq!(m.get::<u32>("since").unwrap(), Some(30));
    assert_eq!(m.values("path"), ["a", "b", "c"]);
    let m = ok(&["report"]);
    assert_eq!((m.arg("part"), m.value("since"), m.value("out")), (Some("findings"), Some("14"), None));
    assert_eq!(ok(&["report", "-ofile"]).value("out"), Some("file"));
    assert!(ok(&["report", "-yq"]).flag("quiet"), "short flags run together");
    let m = ok(&["report", "--licenses"]);
    assert!(m.flag("licenses") && m.value("licenses").is_none(), "an optional value left out");
    assert_eq!(ok(&["report", "--licenses", "l.txt"]).value("licenses"), Some("l.txt"));
    assert_eq!(ok(&["report", "--licenses", "--json"]).value("licenses"), None);
    assert!(ok(&["-q", "report"]).flag("quiet"), "a parent's option before the subcommand");
    assert!(ok(&["report", "-q"]).flag("quiet"), "and after it");
    assert_eq!(ok(&["search", "--", "--json"]).arg("query"), Some("--json"), "-- ends the options");
}

#[test]
fn subcommands_nest_and_aliases_answer() {
    let m = ok(&["rules", "gen", "a.bin", "b.bin", "--name", "R"]);
    assert_eq!((m.command(), m.path()), ("gen", vec!["rules", "gen"]));
    assert_eq!((m.args("sample"), m.value("name")), (vec!["a.bin", "b.bin"], Some("R")));
    assert!(ok(&["rules", "--update"]).flag("update"));
    assert_eq!(ok(&["schedule", "install"]).value("every"), Some("day"));
    assert_eq!(ok(&["ex"]).command(), "export");
}

#[test]
fn help_and_version() {
    let root = program();
    for argv in [&[][..], &["-h"], &["--help"], &["help"]] {
        assert_eq!(root.parse(argv), Err(Exit::Help(root.help_for(&[]).unwrap())), "{argv:?}");
    }
    let report = Exit::Help(root.help_for(&["report"]).unwrap());
    for argv in [&["report", "-h"][..], &["report", "--json", "--help"], &["help", "report"]] {
        assert_eq!(root.parse(argv), Err(report.clone()), "{argv:?}");
    }
    assert_eq!(root.parse(&["rules", "help", "gen"]), Err(Exit::Help(root.help_for(&["rules", "gen"]).unwrap())));
    assert_eq!(root.parse(&["help", "rules", "gen"]), Err(Exit::Help(root.help_for(&["rules", "gen"]).unwrap())));
    assert_eq!(root.parse(&["--version"]), Err(Exit::Version("tool 1.2.3\n".into())));
    assert_eq!(root.parse(&["-V"]), Err(Exit::Version("tool 1.2.3\n".into())));
    assert!(matches!(root.parse(&["help", "nope"]), Err(Exit::Error(_))));
}

#[test]
fn usage_errors_name_the_problem_and_the_closest_names() {
    let e = err(&["report", "--jsn"]);
    assert!(
        e.starts_with("Usage: tool report [options] [part]\nerror: unknown option '--jsn'\n(Did you mean --json?)"),
        "{e}"
    );
    assert!(err(&["reprot"]).contains("error: unknown command 'reprot'\n(Did you mean report?)"));
    assert!(err(&["zzzzzz"]).ends_with("error: unknown command 'zzzzzz'\n"), "nothing close: no suggestion");
    assert!(err(&["report", "-z"]).contains("unknown option '-z'"));
    assert!(err(&["report", "-yz"]).contains("unknown option '-z' in '-yz'"));
    assert!(err(&["report", "--md", "--json"]).contains("option '--json' cannot be used with option '--md'"));
    assert!(err(&["report", "-o"]).contains("option '-o, --out <file>' argument missing"));
    assert!(err(&["report", "--json=x"]).contains("option '--json' takes no value, got 'x'"));
    assert!(
        err(&["report", "nope"])
            .contains("argument 'nope' is invalid for 'part'. Allowed choices are findings, summary.")
    );
    assert!(
        err(&["report", "summary", "x"]).contains("too many arguments for 'report'. Expected 1 argument but got 2: x")
    );
    assert!(err(&["search"]).contains("missing required argument 'query'"));
    assert!(err(&["schedule"]).contains("missing command: one of install"));
    assert!(
        err(&["schedule", "install", "--every", "year"])
            .contains("option '--every <period>' argument 'year' is invalid. Allowed choices are day, week.")
    );
    let m = ok(&["report", "--since", "x"]);
    let Err(Exit::Error(e)) = m.get::<u32>("since") else { panic!() };
    assert!(e.contains("error: option '--since <days>' argument 'x' is invalid"), "{e}");
    assert_eq!(m.error("no").text(), "Usage: tool report [options] [part]\nerror: no\n");
}

#[test]
fn typos_are_close_and_strangers_are_not() {
    assert_eq!(distance("json", "jsno"), 1);
    assert_eq!(distance("kitten", "sitting"), 3);
    assert_eq!(similar("updat", ["update", "upgrade", "json"].into_iter()), ["update"]);
    assert!(similar("x", ["json"].into_iter()).is_empty());
    assert_eq!(did_you_mean(&["a", "b"]), "\n(Did you mean one of a, b?)");
}

#[test]
fn a_flag_is_counted() {
    let p = Command::new("tool").option("-v, --verbose", "more").command(Command::new("run"));
    assert_eq!(p.parse(&["run"]).unwrap().count("verbose"), 0);
    assert_eq!(p.parse(&["run", "-v"]).unwrap().count("verbose"), 1);
    assert_eq!(p.parse(&["run", "-vvv"]).unwrap().count("verbose"), 3);
    assert_eq!(p.parse(&["-v", "run", "--verbose", "-v"]).unwrap().count("verbose"), 3);
    assert_eq!(ok(&["report", "--path", "a", "--path", "b"]).count("path"), 2);
}

#[test]
fn a_required_option_must_be_given() {
    let p = Command::new("tool")
        .command(Command::new("send").opt(Opt::new("--to <address>", "whom").required()).option("--cc <address>", ""));
    assert_eq!(p.parse(&["send", "--to", "a@b"]).unwrap().value("to"), Some("a@b"));
    match p.parse(&["send", "--cc", "x"]) {
        Err(Exit::Error(t)) => {
            assert!(
                t.starts_with("Usage: tool send [options]\nerror: missing required option '--to <address>'\n"),
                "{t}"
            )
        },
        other => panic!("{other:?}"),
    }
    assert!(matches!(p.parse(&["send", "--help"]), Err(Exit::Help(_))), "help wins over the check");
    assert!(
        p.help_for(&["send"]).unwrap().contains("  --to <address>  whom (required)\n"),
        "{}",
        p.help_for(&["send"]).unwrap()
    );
}

#[test]
#[should_panic(expected = "a flag can't be required")]
fn a_required_flag_is_a_bug() {
    let _ = Opt::new("--yes", "").required();
}

#[test]
#[should_panic(expected = "required and has a default")]
fn a_required_option_with_a_default_is_a_bug() {
    let _ = Command::new("tool").opt(Opt::new("--to <a>", "").required().default("x"));
}

#[test]
fn required_is_checked_on_the_command_that_runs_only() {
    let p = Command::new("tool").command(
        Command::new("shard")
            .opt(Opt::new("--input <path>", "").required())
            .opt(Opt::new("--family <name>", "").required())
            .command(Command::new("verify").opt(Opt::new("--shards <path...>", "").required())),
    );
    assert!(p.parse(&["shard", "--input", "a", "--family", "x"]).is_ok());
    assert!(
        p.parse(&["shard", "verify", "--shards", "a", "b"]).is_ok(),
        "the parent's required options are optional here"
    );
    assert_eq!(p.parse(&["shard", "verify", "--shards", "a", "--family", "x"]).unwrap().value("family"), Some("x"));
    match p.parse(&["shard", "verify"]) {
        Err(Exit::Error(t)) => assert!(t.contains("missing required option '--shards <path...>'"), "{t}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn exactly_one_of_a_set_must_be_given() {
    let p = Command::new("tool").command(
        Command::new("generate")
            .option("--model <name>", "the catalog model")
            .option("--checkpoint <path>", "a path to weights")
            .option("--json", "")
            .one_of(&["model", "checkpoint"]),
    );
    assert_eq!(p.parse(&["generate", "--model", "m"]).unwrap().value("model"), Some("m"));
    assert_eq!(p.parse(&["generate", "--checkpoint", "p", "--json"]).unwrap().value("checkpoint"), Some("p"));
    match p.parse(&["generate", "--json"]) {
        Err(Exit::Error(t)) => {
            assert!(t.ends_with("error: one of options '--model <name>', '--checkpoint <path>' is required\n"), "{t}")
        },
        other => panic!("{other:?}"),
    }
    match p.parse(&["generate", "--model", "m", "--checkpoint", "p"]) {
        Err(Exit::Error(t)) => assert!(
            t.ends_with("error: option '--checkpoint <path>' cannot be used with option '--model <name>'\n"),
            "{t}"
        ),
        other => panic!("{other:?}"),
    }
    let help = p.help_for(&["generate"]).unwrap();
    assert!(help.contains("  --model <name>       the catalog model (one of --model, --checkpoint)\n"), "{help}");
    assert!(help.contains("  --checkpoint <path>  a path to weights (one of --model, --checkpoint)\n"), "{help}");
    assert!(p.parse(&["generate", "--help"]).is_err_and(|e| matches!(e, Exit::Help(_))));
}

#[test]
#[should_panic(expected = "names an option it doesn't have")]
fn one_of_must_name_declared_options() {
    let _ = Command::new("tool").option("--a <x>", "").one_of(&["a", "b"]);
}

#[test]
#[should_panic(expected = "required and in a one_of set")]
fn one_of_and_required_do_not_mix() {
    let _ = Command::new("tool").opt(Opt::new("--a <x>", "").required()).option("--b <x>", "").one_of(&["a", "b"]);
}
