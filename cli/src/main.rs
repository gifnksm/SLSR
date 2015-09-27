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

extern crate docopt;
extern crate libc;
extern crate rustc_serialize;
extern crate term;
extern crate slsr_core;
extern crate slsr_solver;

use std::{fmt, io, process};
use std::error::Error;
use std::io::prelude::*;

use slsr_core::puzzle::{Puzzle, ParsePuzzleError};
use slsr_solver as solver;

use parse_arg::{Config, OutputType};

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
            AppError::Solver(ref e) => write!(f, "solver error: {}", e)
        }
    }
}

type AppResult<T> = Result<T, AppError>;

fn run() -> AppResult<()> {
    let config = Config::parse();

    let mut input = String::new();
    let _ = try!(io::stdin().read_to_string(&mut input));
    let puzzle = try!(input.parse::<Puzzle>());
    let puzzle = try!(solver::solve(&puzzle));

    match config.output_type {
        OutputType::Pretty(conf) => {
            let _ = pprint::print(&conf, &puzzle);
        }
        OutputType::Raw => {
            print!("{}", puzzle.to_string());
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
