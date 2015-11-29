//! Slither Link solver logic.

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

extern crate union_find;
extern crate srither_core;

use std::{fmt, mem};
use std::error::Error as ErrorTrait;

use srither_core::puzzle::Puzzle;
use srither_core::geom::CellId;

use solver::Solver;
use theorem_define::THEOREM_DEFINE;

mod model {
    pub mod connect_map;
    pub mod side_map;
    pub mod theorem;
}
mod step {
    pub mod apply_theorem;
    pub mod connect_analysis;
}
mod theorem_define;
mod solver;

#[derive(Copy, Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Copy, Clone, Debug)]
enum ErrorKind {
    InvalidBoard,
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::InvalidBoard => "invalid board data",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error {
    fn invalid_board() -> Error {
        Error { kind: ErrorKind::InvalidBoard }
    }
}

pub type SolverResult<T> = Result<T, Error>;

enum FillResult<'a> {
    Completed(Solver<'a>),
    Partial(Solver<'a>, Vec<CellId>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<T> {
    Fixed(T),
    Unknown,
    Conflict,
}

impl<T> Into<SolverResult<Option<T>>> for State<T> {
    fn into(self) -> SolverResult<Option<T>> {
        match self {
            State::Fixed(st) => Ok(Some(st)),
            State::Unknown => Ok(None),
            State::Conflict => Err(Error::invalid_board()),
        }
    }
}

fn fill_absolutely_fixed(solver: &mut Solver) -> SolverResult<()> {
    while !solver.all_filled() {
        let rev = solver.revision();

        try!(solver.apply_all_theorem());
        if solver.revision() != rev {
            continue;
        }

        try!(solver.connect_analysis());
        if solver.revision() != rev {
            continue;
        }

        break;
    }

    Ok(())
}

fn fill_by_shallow_backtracking(solver: &mut Solver, pts: &[CellId]) -> SolverResult<bool> {
    let rev = solver.revision();
    let mut solver_in = solver.clone();
    let mut solver_out = solver.clone();

    for &p in pts {
        match solver.get_side(p) {
            State::Fixed(_) => {
                continue;
            }
            State::Unknown => {}
            State::Conflict => {
                return Err(Error::invalid_board());
            }
        }

        solver_in.clone_from(&solver);
        solver_in.set_inside(p);

        if fill_absolutely_fixed(&mut solver_in).is_err() {
            solver.set_outside(p);
            try!(fill_absolutely_fixed(solver));
            continue;
        }

        solver_out.clone_from(&solver);
        solver_out.set_outside(p);

        if fill_absolutely_fixed(&mut solver_out).is_err() {
            mem::swap(solver, &mut solver_in);
            continue;
        }

        solver.mark_common(&mut solver_in, &mut solver_out);
    }

    Ok(solver.revision() != rev)
}

fn fill(mut solver: Solver) -> SolverResult<FillResult> {
    try!(fill_absolutely_fixed(&mut solver));

    if solver.all_filled() {
        return Ok(FillResult::Completed(solver));
    }

    let mut pts = solver.get_unknown_points();
    while try!(fill_by_shallow_backtracking(&mut solver, &pts)) {
        if solver.all_filled() {
            return Ok(FillResult::Completed(solver));
        }
        pts = solver.get_unknown_points();
    }

    Ok(FillResult::Partial(solver, pts))
}

#[derive(Clone, Debug)]
pub struct Solutions<'a> {
    queue: Vec<Solver<'a>>,
}

impl<'a> Solutions<'a> {
    pub fn new(puzzle: &'a Puzzle) -> SolverResult<Solutions<'a>> {
        let theorem = THEOREM_DEFINE.iter().map(|theo| theo.parse().unwrap());
        Ok(Solutions { queue: vec![try!(Solver::new(puzzle, theorem))] })
    }
}

impl<'a> Iterator for Solutions<'a> {
    type Item = Puzzle;

    fn next(&mut self) -> Option<Puzzle> {
        while let Some(solver) = self.queue.pop() {
            let (solver, pts) = match fill(solver) {
                Ok(FillResult::Completed(mut solver)) => {
                    if solver.validate_result().is_err() {
                        continue;
                    }
                    match solver.into() {
                        Ok(result) => return Some(result),
                        Err(_) => continue,
                    }
                }
                Ok(FillResult::Partial(solver, pts)) => (solver, pts),
                Err(_) => continue,
            };
            let p = *pts.last().unwrap();
            let mut solver_in = solver.clone();
            let mut solver_out = solver;
            solver_in.set_inside(p);
            solver_out.set_outside(p);
            self.queue.push(solver_in);
            self.queue.push(solver_out);
        }

        None
    }
}

pub fn solve(puzzle: &Puzzle) -> SolverResult<Puzzle> {
    let mut it = try!(Solutions::new(puzzle));
    if let Some(solution) = it.next() {
        return Ok(solution);
    }

    Err(Error::invalid_board())
}