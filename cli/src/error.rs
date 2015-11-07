use std::{fmt, io};
use std::error::Error;

use slsr_core::puzzle::ParsePuzzleError;
use slsr_solver as solver;

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
