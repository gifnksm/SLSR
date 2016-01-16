// Copyright (c) 2016 srither developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Srither link solver command.

#![warn(bad_style)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#![feature(test)]

#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
#![cfg_attr(feature="dev", warn(mut_mut))]
#![cfg_attr(feature="dev", warn(string_add))]
#![cfg_attr(feature="dev", warn(string_add_assign))]

extern crate term;
extern crate argparse;
extern crate test as rustc_test;
extern crate time;

extern crate srither_core;
extern crate srither_solver;

use std::{io, process};
use std::io::prelude::*;

use error::AppResult;
use parse_arg::Config;

mod error;
mod parse_arg;
mod pprint;

mod solve;
mod test;
mod bench;

fn run() -> AppResult<()> {
    match Config::parse() {
        Config::Solve(config) => solve::run(config),
        Config::Test(config) => test::run(config),
        Config::Bench(config) => bench::run(config),
    }
}

fn main() {
    if let Err(e) = run() {
        let _ = writeln!(&mut io::stderr(), "{}", e);
        process::exit(255);
    }
}
