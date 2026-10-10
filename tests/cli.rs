use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fozzie"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_and_version_exit_successfully_before_opening_a_terminal() {
    for flag in ["-h", "--help"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        for option in [
            "--lines",
            "--prompt",
            "--query",
            "--show-scores",
            "split",
            "benchmark",
        ] {
            assert!(help.contains(option), "missing {option}: {help}");
        }
        assert!(help.contains("[default: 10]"));
        assert!(help.contains("❯ "));
    }
    for flag in ["-V", "--version"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            concat!("fozzie ", env!("CARGO_PKG_VERSION"), "\n")
        );
    }
}

#[test]
fn subcommand_help_preserves_options_without_version_flags() {
    for (command, expected) in [
        (
            "split",
            vec!["--delimiter", "--field", "--output", "[default: FIELD]"],
        ),
        ("benchmark", vec!["--query"]),
    ] {
        let output = run(&[command, "--help"]);
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        for option in expected {
            assert!(help.contains(option));
        }
        assert!(!help.contains("--version"));
    }
}

#[test]
fn malformed_options_report_errors_without_panicking() {
    for args in [
        vec!["--unknown"],
        vec!["--query"],
        vec!["--lines=0"],
        vec!["--lines=abc"],
        vec!["split"],
        vec!["split", "-d", "[", "-f", "1"],
        vec!["benchmark"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("error:"), "{error}");
        assert!(!error.contains("panicked"), "{error}");
    }
}

#[test]
fn build_generates_all_packaged_shell_completions() {
    let directory = std::path::Path::new(env!("CARGO_BIN_EXE_fozzie"))
        .parent()
        .unwrap();
    for filename in ["fozzie.bash", "fozzie.fish", "_fozzie"] {
        let completion = std::fs::read_to_string(directory.join(filename)).unwrap();
        assert!(completion.contains("fozzie"));
        assert!(completion.contains("benchmark"));
        assert!(completion.contains("split"));
        assert!(completion.contains("query"));
        assert!(completion.contains("delimiter"));
    }
}
