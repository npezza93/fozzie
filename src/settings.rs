use clap::{Arg, App, ArgMatches, Error, ErrorKind};
use termion::raw::IntoRawMode;
use std::io::{self, BufRead, stdout, stdin};

pub struct Settings {
    pub lines: usize,
    pub prompt: String,
    pub show_scores: bool,
    pub query: Option<String>,
    pub choices: Vec<String>,
    pub stdin: io::Stdin,
}

impl Settings {
    pub fn stdout(&self) -> termion::raw::RawTerminal<io::Stdout> {
        stdout().into_raw_mode().unwrap()
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
        .arg(lines_arg())
        .arg(prompt_arg())
        .arg(query_arg())
        .arg(show_scores_arg())
}

fn parse_lines(matches: &ArgMatches) -> usize {
    let lines = value_t!(matches, "lines", usize).unwrap_or_else(|e| e.exit());

    if lines < 1 {
        Error::with_description(
            "'lines' must be greater than or equal to 1",
            ErrorKind::InvalidValue
        ).exit();
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

fn choices() -> Vec<String> {
    stdin().lock().lines().map(|line| line.unwrap()).collect()
}

pub fn parse() -> Settings {
    let matches = menu().get_matches();

    let lines = parse_lines(&matches);
    let prompt = parse_prompt(&matches);
    let query = parse_query(&matches);
    let show_scores = matches.is_present("show-scores");
    let stdin = stdin();

    Settings { lines, prompt, query, show_scores, stdin, choices: choices() }
}
