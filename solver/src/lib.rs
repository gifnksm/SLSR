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
use slsr_core::geom::{Geom, Point};

use model::connect_map::ConnectMap;
use model::side_map::SideMap;
use step::apply_theorem::TheoremPool;
use theorem_define::THEOREM_DEFINE;

mod model {
    pub mod connect_map;
    pub mod side_map;
    pub mod theorem;
}
mod theorem_define;
mod step {
    pub mod apply_theorem;
    pub mod connect_analysis;
}

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

impl<T> State<T> {
    fn into_option(self) -> Result<Option<T>, LogicError> {
        match self {
            State::Fixed(st) => Ok(Some(st)),
            State::Unknown => Ok(None),
            State::Conflict => Err(LogicError)
        }
    }
}

fn solve_by_logic(
    side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>,
    theorem_pool: &mut TheoremPool)
    -> SolverResult<()>
{
    let mut rev = side_map.revision();

    while !side_map.all_filled() {
        try!(theorem_pool.apply_all(side_map));

        if side_map.all_filled() {
            return Ok(())
        }
        if side_map.revision() != rev {
            rev = side_map.revision();
            continue
        }

        if conn_map.is_none() {
            *conn_map = Some(ConnectMap::from_side_map(side_map));
        }
        try!(step::connect_analysis::run(side_map, conn_map.as_mut().unwrap()));
        if side_map.revision() == rev {
            break;
        }

        rev = side_map.revision();
    }

    Ok(())
}

fn get_unknown_points(conn_map: &mut ConnectMap) -> Vec<Point> {
    let mut pts = vec![];

    for r in (0 .. conn_map.row()) {
        for c in (0 .. conn_map.column()) {
            let p = Point(r, c);
            let a = conn_map.get(p);
            if p != a.coord() { continue }
            if a.side() != State::Unknown { continue }
            pts.push((p, a.unknown_edge().len()));
        }
    }

    pts.sort_by(|a, b| a.1.cmp(&b.1));
    pts.into_iter().map(|pair| pair.0).collect()
}

fn solve_by_backtracking_one_step(
    side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>,
    theorem_pool: &mut TheoremPool, pts: &[Point])
    -> SolverResult<bool>
{
    let rev = side_map.revision();

    for &p in pts {
        match side_map.get_side(p) {
            State::Fixed(_) => continue,
            State::Unknown => {}
            State::Conflict => { return Err(LogicError) }
        }

        let mut sm0 = side_map.clone();
        let mut cm0 = conn_map.clone();
        let mut th0 = theorem_pool.clone();
        sm0.set_inside(p);

        if solve_by_logic(&mut sm0, &mut cm0, &mut th0).is_err() {
            side_map.set_outside(p);
            try!(solve_by_logic(side_map, conn_map, theorem_pool));
            continue
        }

        let mut sm1 = side_map.clone();
        let mut cm1 = conn_map.clone();
        let mut th1 = theorem_pool.clone();
        sm1.set_outside(p);

        if solve_by_logic(&mut sm1, &mut cm1, &mut th1).is_err() {
            *side_map = sm0;
            *conn_map = cm0;
            *theorem_pool  = th0;
        }
    }

    Ok(side_map.revision() != rev)
}

fn check_answer(side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>)
                -> SolverResult<()>
{
    if conn_map.is_none() {
        *conn_map = Some(ConnectMap::from_side_map(side_map));
    }
    let conn_map = conn_map.as_mut().unwrap();
    try!(conn_map.sync(side_map));
    if conn_map.count_area() != 2 {
        return Err(LogicError)
    }

    Ok(())
}

pub fn solve(board: &Board) -> Result<Board, LogicError> {
    let mut side_map = SideMap::from_board(board);
    let it = THEOREM_DEFINE.iter().map(|theo| theo.parse().unwrap());
    let theorem_pool = try!(TheoremPool::new(it, &mut side_map));

    let mut queue = vec![(side_map, None, theorem_pool)];

    'failure: while let Some((mut side_map, mut conn_map, mut theorem_pool)) = queue.pop() {
        if solve_by_logic(&mut side_map, &mut conn_map, &mut theorem_pool).is_err() {
            continue
        }

        if side_map.all_filled() {
            if check_answer(&mut side_map, &mut conn_map).is_err() {
                continue
            }
            return side_map.to_board()
        }

        debug_assert!(conn_map.is_some());
        let mut pts = get_unknown_points(conn_map.as_mut().unwrap());
        loop {
            match solve_by_backtracking_one_step(
                &mut side_map, &mut conn_map, &mut theorem_pool, &pts)
            {
                Ok(true) => {
                    if side_map.all_filled() {
                        if check_answer(&mut side_map, &mut conn_map).is_err() {
                            continue
                        }
                        return side_map.to_board()
                    }
                    pts = get_unknown_points(conn_map.as_mut().unwrap());
                }
                Ok(false) => break,
                Err(_) => continue 'failure,
            }
        }

        let p = *pts.last().unwrap();
        let mut side_map_0 = side_map.clone();
        let conn_map_0 = conn_map.clone();
        let mut side_map_1 = side_map;
        let conn_map_1 = conn_map;
        side_map_0.set_outside(p);
        side_map_1.set_inside(p);
        queue.push((side_map_0, conn_map_0, theorem_pool.clone()));
        queue.push((side_map_1, conn_map_1, theorem_pool));
    }

    Err(LogicError)
}
