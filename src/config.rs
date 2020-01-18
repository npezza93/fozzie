use clap::{App, Arg, ArgMatches, Error, ErrorKind};

pub struct Config {
    pub lines: usize,
    pub prompt: String,
    pub show_scores: bool,
    pub query: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        let matches = Self::menu().get_matches();

        let lines = Self::parse_lines(&matches);
        let prompt = Self::parse_prompt(&matches);
        let query = Self::parse_query(&matches);
        let show_scores = matches.is_present("show-scores");

        Config {
            lines,
            prompt,
            query,
            show_scores,
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

    fn query_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("QUERY")
            .help("Initial search string")
            .takes_value(true)
    }

    fn show_scores_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("show-scores")
            .short("s")
            .long("show-scores")
            .help("Show the scores of each match")
    }

    fn menu<'a>() -> App<'a, 'a> {
        App::new("fozzie")
            .version("0.1.0")
            .author("Nick Pezza")
            .arg(Self::lines_arg())
            .arg(Self::prompt_arg())
            .arg(Self::query_arg())
            .arg(Self::show_scores_arg())
    }

    fn parse_lines(matches: &ArgMatches) -> usize {
        let lines = value_t!(matches, "lines", usize).unwrap_or_else(|e| e.exit());

        if lines < 1 {
            Error::with_description(
                "'lines' must be greater than or equal to 1",
                ErrorKind::InvalidValue,
            )
            .exit();
        }

        lines
    }

    fn parse_query(matches: &ArgMatches) -> Option<String> {
        if matches.is_present("query") {
            Some(value_t!(matches, "query", String).unwrap_or_else(|e| e.exit()))
        } else {
            None
        }
    }

    fn parse_prompt(matches: &ArgMatches) -> String {
        value_t!(matches, "prompt", String).unwrap_or_else(|e| e.exit())
    }
}
