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

mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use rustc_test::{self as test, DynTestFn, DynTestName, ShouldPanic, TestDesc, TestDescAndFn};

    use slsr_core::puzzle::Puzzle;
    use slsr_solver::{self as solver, Solutions};

    use error::AppResult;
    use parse_arg::TestConfig;

    pub fn run(config: TestConfig) -> AppResult<()> {
        let derive_all = config.derive_all;
        let tests = config.input_files
                          .into_iter()
                          .map(|input| {
                              TestDescAndFn {
                                  desc: TestDesc {
                                      name: DynTestName(input.clone()),
                                      ignore: false,
                                      should_panic: ShouldPanic::No,
                                  },
                                  testfn: DynTestFn(Box::new(move || {
                                      solve(&input, derive_all).unwrap()
                                  })),
                              }
                          })
                          .collect();

        test::test_main(&["".to_string()], tests);

        Ok(())
    }

    fn solve(file: &str, derive_all: bool) -> AppResult<()> {
        let mut buf = String::new();
        let _ = try!(try!(File::open(file)).read_to_string(&mut buf));
        let puzzle = try!(buf.parse::<Puzzle>());

        if derive_all {
            for solution in try!(Solutions::new(&puzzle)) {
                let _ = test::black_box(solution);
            }
        } else {
            let _ = test::black_box(try!(solver::solve(&puzzle)));
        }

        Ok(())
    }
}

mod bench {
    use std::fs::File;
    use std::io::prelude::*;
    use time;
    use rustc_test::{self as test, DynBenchFn, DynTestName, ShouldPanic, TestDesc, TestDescAndFn};

    use slsr_core::puzzle::Puzzle;
    use slsr_solver::{self as solver, Solutions};

    use error::AppResult;
    use parse_arg::BenchConfig;

    pub fn run(config: BenchConfig) -> AppResult<()> {
        let derive_all = config.derive_all;
        let inputs = if let Some(n) = config.only_hardest {
            take_hardest(config.input_files, n, derive_all)
        } else {
            config.input_files
        };
        let tests = inputs.into_iter()
                          .map(|input| {
                              TestDescAndFn {
                                  desc: TestDesc {
                                      name: DynTestName(input.clone()),
                                      ignore: false,
                                      should_panic: ShouldPanic::No,
                                  },
                                  testfn: DynBenchFn(Box::new(move |bencher| {
                                      bencher.iter(|| solve(&input, derive_all))
                                  })),
                              }
                          })
                          .collect();

        test::test_main(&["".to_string(), "--bench".to_string()], tests);

        Ok(())
    }

    fn get_elapse(input: &str, derive_all: bool) -> u64 {
        let start = time::precise_time_ns();
        let _ = test::black_box(solve(input, derive_all));
        time::precise_time_ns() - start
    }

    fn take_hardest(inputs: Vec<String>, n: usize, derive_all: bool) -> Vec<String> {
        let mut inputs = inputs.into_iter()
                               .map(|input| (get_elapse(&input, derive_all), input))
                               .collect::<Vec<_>>();
        inputs.sort_by(|a, b| a.cmp(b).reverse());
        inputs.into_iter()
              .map(|pair| pair.1)
              .take(n)
              .collect()
    }

    fn solve(file: &str, derive_all: bool) -> AppResult<()> {
        let mut buf = String::new();
        let _ = try!(try!(File::open(file)).read_to_string(&mut buf));
        let puzzle = try!(buf.parse::<Puzzle>());

        if derive_all {
            for solution in try!(Solutions::new(&puzzle)) {
                let _ = test::black_box(solution);
            }
        } else {
            let _ = test::black_box(try!(solver::solve(&puzzle)));
        }

        Ok(())
    }
}

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
