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

fn solve_by_logic(solver: &mut Solver) -> SolverResult<()>
{
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

fn solve_by_backtracking_one_step(solver: &mut Solver, pts: &[CellId])
                                  -> SolverResult<bool>
{
    let rev = solver.revision();

    for &p in pts {
        match solver.get_side(p) {
            State::Fixed(_) => continue,
            State::Unknown => {}
            State::Conflict => { return Err(LogicError) }
        }

        let mut solver_0 = solver.clone();
        solver_0.set_inside(p);

        if solve_by_logic(&mut solver_0).is_err() {
            solver.set_outside(p);
            try!(solve_by_logic(solver));
            continue
        }

        let mut solver_1 = solver.clone();
        solver_1.set_outside(p);

        if solve_by_logic(&mut solver_1).is_err() {
            *solver = solver_0;
        }
    }

    Ok(solver.revision() != rev)
}

pub fn solve(board: &Board) -> Result<Board, LogicError> {
    let theorem = THEOREM_DEFINE.iter().map(|theo| theo.parse().unwrap());

    let mut queue = vec![try!(Solver::new(board, theorem))];

    'failure: while let Some(mut solver) = queue.pop() {
        if solve_by_logic(&mut solver).is_err() {
            continue
        }

        if solver.all_filled() {
            if solver.validate_result().is_err() {
                continue
            }
            return solver.into()
        }

        let mut pts = solver.get_unknown_points();
        loop {
            match solve_by_backtracking_one_step(&mut solver, &pts) {
                Ok(true) => {
                    if solver.all_filled() {
                        if solver.validate_result().is_err() {
                            continue
                        }
                        return solver.into()
                    }
                    pts = solver.get_unknown_points();
                }
                Ok(false) => break,
                Err(_) => continue 'failure,
            }
        }

        let p = *pts.last().unwrap();
        let mut solver_0 = solver.clone();
        let mut solver_1 = solver;
        solver_0.set_outside(p);
        solver_1.set_inside(p);
        queue.push(solver_0);
        queue.push(solver_1);
    }

    Err(LogicError)
}
