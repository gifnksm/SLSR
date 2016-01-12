// Copyright (c) 2016 srither developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fs::File;
use std::io::prelude::*;
use time;
use rustc_test::{self as test, Bencher, DynBenchFn, DynTestName, ShouldPanic, TDynBenchFn,
                 TestDesc, TestDescAndFn};

use srither_core::puzzle::Puzzle;
use srither_solver::{self as solver, Solutions};

use error::AppResult;
use parse_arg::BenchConfig;

struct BenchFn {
    input: String,
    derive_all: bool,
}

impl TDynBenchFn for BenchFn {
    fn run(&self, harness: &mut Bencher) {
        harness.iter(|| solve(&self.input, self.derive_all))
    }
}

impl BenchFn {
    fn new(input: String, derive_all: bool) -> BenchFn {
        BenchFn {
            input: input,
            derive_all: derive_all,
        }
    }
}

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
                              testfn: DynBenchFn(Box::new(BenchFn::new(input, derive_all))),
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
