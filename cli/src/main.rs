#![warn(bad_style)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

extern crate ansi_term;
extern crate argparse;
extern crate slsr_core;
extern crate slsr_solver;

use std::{fmt, io, process};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

use slsr_core::puzzle::{Puzzle, ParsePuzzleError};
use slsr_solver::{self as solver, Solutions};

use parse_arg::{Config, OutputMode};

mod parse_arg;
mod pprint;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    ParsePuzzle(ParsePuzzleError),
    Solver(solver::Error),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

impl From<ParsePuzzleError> for AppError {
    fn from(err: ParsePuzzleError) -> AppError {
        AppError::ParsePuzzle(err)
    }
}

impl From<solver::Error> for AppError {
    fn from(err: solver::Error) -> AppError {
        AppError::Solver(err)
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::Io(ref e) => e.description(),
            AppError::ParsePuzzle(ref e) => e.description(),
            AppError::Solver(ref e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            AppError::Io(ref e) => Some(e),
            AppError::ParsePuzzle(ref e) => Some(e),
            AppError::Solver(ref e) => Some(e),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::Io(ref e) => write!(f, "IO error: {}", e),
            AppError::ParsePuzzle(ref e) => write!(f, "parse puzzle error: {}", e),
            AppError::Solver(ref e) => write!(f, "solver error: {}", e),
        }
    }
}

type AppResult<T> = Result<T, AppError>;

fn output(config: &Config, solution: Puzzle) -> io::Result<()> {
    match config.output_mode {
        OutputMode::Pretty(conf) => {
            try!(pprint::print(&conf, &solution));
        }
        OutputMode::Raw => {
            print!("{}", solution.to_string());
        }
        OutputMode::None => {}
    }

    Ok(())
}

fn solve<T: Read>(config: &Config, input: &mut T) -> AppResult<()> {
    let mut buf = String::new();
    let _ = try!(input.read_to_string(&mut buf));
    let puzzle = try!(buf.parse::<Puzzle>());

    if config.derive_all {
        for solution in try!(Solutions::new(&puzzle)) {
            try!(output(&config, solution));
        }
    } else {
        let solution = try!(solver::solve(&puzzle));
        try!(output(&config, solution));
    }

    Ok(())
}

fn run() -> AppResult<()> {
    let config = Config::parse();

    if config.input_files.is_empty() {
        try!(solve(&config, &mut io::stdin()));
    } else {
        for file in &config.input_files {
            let mut f = try!(File::open(file));
            try!(solve(&config, &mut f));
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        let _ = writeln!(&mut io::stderr(), "{}", e);
        process::exit(255);
    }
}
