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

use std::{io, process};
use std::io::prelude::*;

use error::AppResult;
use parse_arg::Config;

mod error;
mod parse_arg;
mod pprint;

mod solve {
    use std::io;
    use std::fs::File;
    use std::io::prelude::*;

    use slsr_core::puzzle::Puzzle;
    use slsr_solver::{self as solver, Solutions};

    use error::AppResult;
    use parse_arg::{OutputMode, SolveConfig};
    use pprint;

    pub fn run(config: SolveConfig) -> AppResult<()> {
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

    fn solve<T: Read>(config: &SolveConfig, input: &mut T) -> AppResult<()> {
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

    fn output(config: &SolveConfig, solution: Puzzle) -> io::Result<()> {
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
}

fn run() -> AppResult<()> {
    match Config::parse() {
        Config::Solve(config) => solve::run(config),
    }
}

fn main() {
    if let Err(e) = run() {
        let _ = writeln!(&mut io::stderr(), "{}", e);
        process::exit(255);
    }
}
