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
extern crate slsr_core;

use std::fmt;
use slsr_core::board::Board;
use slsr_core::geom::CellId;

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
pub struct LogicError;

impl fmt::Display for LogicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

pub type SolverResult<T> = Result<T, LogicError>;

enum FillResult {
    Completed(Solver),
    Partial(Solver, Vec<CellId>)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State<T> {
    Fixed(T), Unknown, Conflict
}

impl<T> Into<Result<Option<T>, LogicError>> for State<T> {
    fn into(self) -> Result<Option<T>, LogicError> {
        match self {
            State::Fixed(st) => Ok(Some(st)),
            State::Unknown => Ok(None),
            State::Conflict => Err(LogicError)
        }
    }
}

fn fill_absolutely_fixed(solver: &mut Solver) -> SolverResult<()> {
    while !solver.all_filled() {
        let rev = solver.revision();

        try!(solver.apply_all_theorem());
        if solver.revision() != rev {
            continue
        }

        try!(solver.connect_analysis());
        if solver.revision() != rev {
            continue
        }

        break
    }

    Ok(())
}

fn fill_by_shallow_backtracking(solver: &mut Solver, pts: &[CellId])
                                -> SolverResult<bool>
{
    let rev = solver.revision();

    for &p in pts {
        match solver.get_side(p) {
            State::Fixed(_) => { continue }
            State::Unknown => {}
            State::Conflict => { return Err(LogicError) }
        }

        let mut solver_in = solver.clone();
        solver_in.set_inside(p);

        if fill_absolutely_fixed(&mut solver_in).is_err() {
            solver.set_outside(p);
            try!(fill_absolutely_fixed(solver));
            continue
        }

        let mut solver_out = solver.clone();
        solver_out.set_outside(p);

        if fill_absolutely_fixed(&mut solver_out).is_err() {
            *solver = solver_in;
        }
    }

    Ok(solver.revision() != rev)
}

fn fill(mut solver: Solver) -> SolverResult<FillResult> {
    try!(fill_absolutely_fixed(&mut solver));

    if solver.all_filled() {
        return Ok(FillResult::Completed(solver))
    }

    let mut pts = solver.get_unknown_points();
    while try!(fill_by_shallow_backtracking(&mut solver, &pts)) {
        if solver.all_filled() {
            return Ok(FillResult::Completed(solver))
        }
        pts = solver.get_unknown_points();
    }

    Ok(FillResult::Partial(solver, pts))
}

pub fn solve(board: &Board) -> Result<Board, LogicError> {
    let theorem = THEOREM_DEFINE.iter().map(|theo| theo.parse().unwrap());
    let mut queue = vec![try!(Solver::new(board, theorem))];

    while let Some(solver) = queue.pop() {
        let (solver,pts) = match fill(solver) {
            Ok(FillResult::Completed(mut solver)) => {
                if solver.validate_result().is_err() {
                    continue
                }
                return solver.into()
            }
            Ok(FillResult::Partial(solver, pts)) => (solver, pts),
            Err(_) => continue
        };

        let p = *pts.last().unwrap();
        let mut solver_in = solver.clone();
        let mut solver_out = solver;
        solver_in.set_inside(p);
        solver_out.set_outside(p);
        queue.push(solver_in);
        queue.push(solver_out);
    }

    Err(LogicError)
}
