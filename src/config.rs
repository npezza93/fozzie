use clap::{error::ErrorKind, value_parser, Arg, ArgAction, ArgMatches, Command, Error};
use regex::Regex;
use terminal_size::{terminal_size, Height, Width};

pub struct Config {
    pub lines: usize,
    pub prompt: String,
    pub show_scores: bool,
    pub query: Option<String>,
    pub delimiter: Option<Regex>,
    pub field: Option<usize>,
    pub output: Option<usize>,
    pub benchmark: bool,
}

impl Config {
    /// Parse the process command-line arguments. Kept for existing library callers.
    // A Default implementation must not parse arguments or exit the process.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::from_args()
    }

    pub fn from_args() -> Self {
        Self::from_matches(&Self::menu().get_matches()).unwrap_or_else(|error| error.exit())
    }

    fn from_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let lines = *matches.get_one::<usize>("lines").unwrap();
        validate_lines(lines, terminal_size().map(|(Width(_), Height(h))| h))?;
        let split = matches.subcommand_matches("split");
        let query = matches.get_one::<String>("query").or_else(|| {
            matches
                .subcommand_matches("benchmark")
                .and_then(|matches| matches.get_one::<String>("query"))
        });

        Ok(Self {
            lines,
            prompt: matches.get_one::<String>("prompt").unwrap().clone(),
            show_scores: matches.get_flag("show-scores"),
            query: query.cloned(),
            delimiter: split.and_then(|matches| matches.get_one::<Regex>("delimiter").cloned()),
            field: split.and_then(|matches| matches.get_one::<usize>("field").copied()),
            output: split.and_then(|matches| matches.get_one::<usize>("output").copied()),
            benchmark: matches.subcommand_name() == Some("benchmark"),
        })
    }

    fn lines_arg() -> Arg {
        Arg::new("lines")
            .short('l')
            .long("lines")
            .value_name("LINES")
            .help("Specify how many lines of results to show")
            .value_parser(parse_positive_lines)
            .default_value("10")
    }

    fn prompt_arg() -> Arg {
        Arg::new("prompt")
            .short('p')
            .long("prompt")
            .value_name("PROMPT")
            .help("Input prompt")
            .default_value("❯ ")
    }

    fn show_scores_arg() -> Arg {
        Arg::new("show-scores")
            .short('s')
            .long("show-scores")
            .help("Show the scores of each match")
            .action(ArgAction::SetTrue)
    }

    fn query_arg() -> Arg {
        Arg::new("query")
            .short('q')
            .long("query")
            .value_name("QUERY")
            .help("Initial search string")
    }

    fn delimiter_arg() -> Arg {
        Arg::new("delimiter")
            .short('d')
            .long("delimiter")
            .value_name("DELIMITER")
            .help("Use to split the line into fields")
            .value_parser(value_parser!(Regex))
            .required(true)
    }

    fn field_arg() -> Arg {
        Arg::new("field")
            .short('f')
            .long("field")
            .value_name("FIELD")
            .help("Field to be matched")
            .value_parser(value_parser!(usize))
            .required(true)
    }

    fn output_arg() -> Arg {
        Arg::new("output")
            .short('o')
            .long("output")
            .value_name("OUTPUT")
            .help("Field to be returned once selected [default: FIELD]")
            .value_parser(value_parser!(usize))
    }

    pub fn menu() -> Command {
        Command::new("fozzie")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .disable_help_subcommand(true)
            .arg(Self::lines_arg())
            .arg(Self::prompt_arg())
            .arg(Self::query_arg())
            .arg(Self::show_scores_arg())
            .subcommand(Self::split_subcommand())
            .subcommand(Self::benchmark_subcommand())
    }

    fn split_subcommand() -> Command {
        Command::new("split")
            .about("Splits lines into fields")
            .disable_version_flag(true)
            .arg(Self::delimiter_arg())
            .arg(Self::field_arg())
            .arg(Self::output_arg())
    }

    fn benchmark_subcommand() -> Command {
        Command::new("benchmark")
            .about("Disables iteractivity to allow benchmarking")
            .disable_version_flag(true)
            .arg(Self::query_arg().required(true))
    }
}

fn parse_positive_lines(value: &str) -> Result<usize, String> {
    let lines = value.parse::<usize>().map_err(|error| error.to_string())?;
    if lines == 0 {
        return Err("The argument 'lines' must be greater than 0".to_string());
    }
    Ok(lines)
}

fn validate_lines(lines: usize, terminal_height: Option<u16>) -> Result<(), Error> {
    if let Some(height) = terminal_height {
        if usize::from(height) <= lines {
            return Err(Error::raw(
                ErrorKind::ValueValidation,
                format!("The argument 'lines' must be less than {}", height),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Config {
        let matches = Config::menu().try_get_matches_from(args).unwrap();
        Config::from_matches(&matches).unwrap()
    }

    #[test]
    fn command_definition_is_valid() {
        Config::menu().debug_assert();
    }

    #[test]
    fn defaults_are_unchanged() {
        let config = parse(&["fozzie"]);
        assert_eq!(config.lines, 10);
        assert_eq!(config.prompt, "❯ ");
        assert!(!config.show_scores);
        assert!(config.query.is_none());
        assert!(config.delimiter.is_none());
        assert!(config.field.is_none());
        assert!(config.output.is_none());
        assert!(!config.benchmark);
    }

    #[test]
    fn short_and_long_options_are_equivalent() {
        for args in [
            vec!["fozzie", "-l", "1", "-p", "> ", "-s", "-q", "a,b"],
            vec![
                "fozzie",
                "--lines=1",
                "--prompt=> ",
                "--show-scores",
                "--query=a,b",
            ],
        ] {
            let config = parse(&args);
            assert_eq!(config.lines, 1);
            assert_eq!(config.prompt, "> ");
            assert!(config.show_scores);
            assert_eq!(config.query.as_deref(), Some("a,b"));
        }
    }

    #[test]
    fn empty_prompt_and_query_are_allowed() {
        let config = parse(&["fozzie", "-p", "", "-q", ""]);
        assert_eq!(config.prompt, "");
        assert_eq!(config.query.as_deref(), Some(""));
    }

    #[test]
    fn split_preserves_regex_and_optional_output() {
        let config = parse(&["fozzie", "split", "-d", "\\s+", "-f", "0"]);
        assert!(config.delimiter.unwrap().is_match(" "));
        assert_eq!(config.field, Some(0));
        assert_eq!(config.output, None);
        assert!(!config.benchmark);

        let config = parse(&[
            "fozzie",
            "split",
            "--delimiter=:|,",
            "--field=2",
            "--output=1",
        ]);
        assert_eq!(config.delimiter.unwrap().as_str(), ":|,");
        assert_eq!(config.field, Some(2));
        assert_eq!(config.output, Some(1));
    }

    #[test]
    fn benchmark_uses_its_query_unless_top_level_query_is_present() {
        let config = parse(&["fozzie", "benchmark", "-q", "inner"]);
        assert!(config.benchmark);
        assert_eq!(config.query.as_deref(), Some("inner"));
        let config = parse(&["fozzie", "-q", "outer", "benchmark", "-q", "inner"]);
        assert!(config.benchmark);
        assert_eq!(config.query.as_deref(), Some("outer"));
    }

    #[test]
    fn required_subcommand_options_are_enforced() {
        for args in [
            vec!["fozzie", "split"],
            vec!["fozzie", "split", "-d", ":"],
            vec!["fozzie", "split", "-f", "1"],
            vec!["fozzie", "benchmark"],
            vec!["fozzie", "-q", "outer", "benchmark"],
        ] {
            let error = Config::menu().try_get_matches_from(args).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
        }
    }

    #[test]
    fn invalid_typed_values_are_errors() {
        for args in [
            vec!["fozzie", "-l", "0"],
            vec!["fozzie", "-l", "abc"],
            vec!["fozzie", "--lines=-1"],
            vec!["fozzie", "-l", "999999999999999999999999999999999999999"],
            vec!["fozzie", "split", "-d", "[", "-f", "1"],
            vec!["fozzie", "split", "-d", ":", "--field=-1"],
            vec!["fozzie", "split", "-d", ":", "-f", "abc"],
            vec!["fozzie", "split", "-d", ":", "-f", "1", "-o", "abc"],
        ] {
            assert_eq!(
                Config::menu()
                    .try_get_matches_from(args)
                    .unwrap_err()
                    .kind(),
                ErrorKind::ValueValidation
            );
        }
    }

    #[test]
    fn line_limit_reserves_space_for_the_prompt() {
        assert!(validate_lines(10, None).is_ok());
        assert!(validate_lines(9, Some(10)).is_ok());
        assert!(validate_lines(10, Some(10)).is_err());
        assert!(validate_lines(11, Some(10)).is_err());
        assert_eq!(parse_positive_lines("+2").unwrap(), 2);
    }

    #[test]
    fn duplicate_arguments_are_rejected() {
        for args in [
            vec!["fozzie", "-ss"],
            vec!["fozzie", "-q", "one", "-q", "two"],
        ] {
            assert_eq!(
                Config::menu()
                    .try_get_matches_from(args)
                    .unwrap_err()
                    .kind(),
                ErrorKind::ArgumentConflict
            );
        }
    }

    #[test]
    fn top_level_arguments_do_not_become_global() {
        for args in [
            vec!["fozzie", "benchmark", "-q", "test", "-s"],
            vec!["fozzie", "split", "-d", ":", "-f", "1", "-q", "test"],
            vec!["fozzie", "split", "--version"],
        ] {
            assert!(Config::menu().try_get_matches_from(args).is_err());
        }
    }
}
