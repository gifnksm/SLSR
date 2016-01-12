// Copyright (c) 2016 srither developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fs::File;
use std::io::prelude::*;
use rustc_test::{self as test, DynTestFn, DynTestName, ShouldPanic, TestDesc, TestDescAndFn};

use srither_core::puzzle::Puzzle;
use srither_solver::{self as solver, Solutions};

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
