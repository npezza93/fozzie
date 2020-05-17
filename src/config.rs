use regex::Regex;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, Error};

pub struct Config {
    pub lines: usize,
    pub prompt: String,
    pub show_scores: bool,
    pub query: Option<String>,
    pub delimiter: Option<Regex>,
    pub field: Option<usize>,
    pub output: Option<usize>,
    pub benchmark: bool,
    pub reverse: bool,
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::menu().get_matches();

        let lines       = parse_lines(&matches);
        let prompt      = value_t_or_exit!(matches, "prompt", String);
        let show_scores = matches.is_present("show-scores");
        let reverse     = matches.is_present("reverse");
        let query       = parse_query(&matches);

        let delimiter   = parse_delimiter(&matches);
        let field       = subcommand_usize_value_or_exit(&matches, "field");
        let output      = subcommand_usize_value_or_exit(&matches, "output");

        let benchmark   = match matches.subcommand_name() {
            Some("benchmark") => true, _ => false
        };


        Self {
            lines,
            prompt,
            show_scores,
            reverse,
            query,
            delimiter,
            field,
            output,
            benchmark
        }
    }

    fn lines_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("lines")
            .short("l")
            .long("lines")
            .value_name("LINES")
            .help("Specify how many lines of results to show")
            .takes_value(true)
            .default_value("10")
    }

    fn prompt_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("prompt")
            .short("p")
            .long("prompt")
            .value_name("PROMPT")
            .help("Input prompt")
            .takes_value(true)
            .default_value("❯ ")
    }

    fn show_scores_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("show-scores")
            .short("s")
            .long("show-scores")
            .help("Show the scores of each match")
    }

    fn reverse_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("reverse")
            .short("r")
            .long("reverse")
        .help("Shows the search at the bottom")
    }

    fn query_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("QUERY")
            .help("Initial search string")
            .takes_value(true)
    }

    fn delimiter_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("delimiter")
            .short("d")
            .long("delimiter")
            .value_name("DELIMITER")
            .help("Use to split the line into fields")
            .takes_value(true)
            .required(true)
    }

    fn field_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("field")
            .short("f")
            .long("field")
            .value_name("FIELD")
            .help("Field to be matched")
            .takes_value(true)
            .required(true)
    }

    fn output_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("OUTPUT")
            .help("Field to be returned once selected [default: FIELD]")
            .takes_value(true)
    }

    pub fn menu<'a>() -> App<'a, 'a> {
        App::new("fozzie")
            .version(crate_version!())
            .author(crate_authors!())
            .setting(AppSettings::DisableHelpSubcommand)
            .arg(Self::lines_arg())
            .arg(Self::prompt_arg())
            .arg(Self::query_arg())
            .arg(Self::reverse_arg())
            .arg(Self::show_scores_arg())
            .subcommand(Self::split_subcommand())
            .subcommand(Self::benchmark_subcommand())
    }

    fn split_subcommand<'a>() -> App<'a, 'a> {
        SubCommand::with_name("split")
            .about("Splits lines into fields")
            .setting(AppSettings::DisableVersion)
            .arg(Self::delimiter_arg())
            .arg(Self::field_arg())
            .arg(Self::output_arg())
    }

    fn benchmark_subcommand<'a>() -> App<'a, 'a> {
        SubCommand::with_name("benchmark")
            .about("Disables iteractivity to allow benchmarking")
            .setting(AppSettings::DisableVersion)
            .arg(Self::query_arg().required(true))
    }
}

fn parse_query(matches: &ArgMatches) -> Option<String> {
    if matches.is_present("query") {
        Some(value_t_or_exit!(matches, "query", String))
    } else {
        if let Some(matches) = matches.subcommand_matches("benchmark") {
            if matches.is_present("query") {
                Some(value_t_or_exit!(matches, "query", String))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn optional_usize_value_or_exit(matches: &ArgMatches, field: &str) -> Option<usize> {
    if matches.is_present(field) {
        Some(value_t_or_exit!(matches, field, usize))
    } else {
        None
    }
}

fn subcommand_usize_value_or_exit(matches: &ArgMatches, field: &str) -> Option<usize> {
    if let Some(matches) = matches.subcommand_matches("split") {
        optional_usize_value_or_exit(matches, field)
    } else {
        None
    }
}

fn parse_lines(matches: &ArgMatches) -> usize {
    let lines = value_t_or_exit!(matches, "lines", usize);

    if lines < 1 {
        Error::value_validation_auto(format!("The argument '{}' must be greater than 0", "lines")).exit();
    }
    if let Some((_w, h)) = term_size::dimensions() {
        if h <= lines {
            Error::value_validation_auto(format!("The argument '{}' must be less than {}", "lines", h)).exit();
        }
    }

    lines
}

fn parse_delimiter(matches: &ArgMatches) -> Option<Regex> {
    if let Some(matches) = matches.subcommand_matches("split") {
        let delimiter = value_t_or_exit!(matches, "delimiter", String);

        Some(Regex::new(&delimiter).unwrap())
    } else {
        None
    }
}
