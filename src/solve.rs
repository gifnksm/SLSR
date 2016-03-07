// Copyright (c) 2016 srither developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::io;
use std::fs::File;
use std::io::prelude::*;

use srither_core::puzzle::Puzzle;
use srither_solver::{Solutions, self as solver};

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

fn output(config: &SolveConfig, solution: Puzzle) -> AppResult<()> {
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
