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

extern crate term;
extern crate argparse;
extern crate rustc_test;
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
