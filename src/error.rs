// Copyright (c) 2016 srither developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{fmt, io};
use std::error::Error;

use srither_core::puzzle::ParsePuzzleError;
use srither_solver as solver;

#[derive(Debug)]
pub enum AppError {
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

pub type AppResult<T> = Result<T, AppError>;
