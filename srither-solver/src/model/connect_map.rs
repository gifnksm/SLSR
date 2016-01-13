// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::iter::FromIterator;
use std::mem;
use union_find::{Union, UnionFind, UnionResult, QuickFindUf as Uf};
use srither_core::puzzle::{Edge, Puzzle, Side};
use srither_core::geom::{CellId, Geom, Point, Move};

use {Error, SolverResult};
use model::State;
use model::side_map::SideMap;

#[derive(Debug)]
pub struct Area {
    coord: CellId,
    side: State<Side>,
    unknown_edge: Vec<CellId>,
    sum_of_hint: u32,
    size: usize,
}

impl Clone for Area {
    fn clone(&self) -> Area {
        Area {
            coord: self.coord,
            side: self.side,
            unknown_edge: self.unknown_edge.clone(),
            sum_of_hint: self.sum_of_hint,
            size: self.size,
        }
    }

    fn clone_from(&mut self, other: &Area) {
        self.coord = other.coord;
        self.side = other.side;
        self.unknown_edge.clone_from(&other.unknown_edge);
        self.sum_of_hint = other.sum_of_hint;
        self.size = other.size;
    }
}

impl Area {
    pub fn coord(&self) -> CellId {
        self.coord
    }
    pub fn side(&self) -> State<Side> {
        self.side
    }
    pub fn unknown_edge(&self) -> &[CellId] {
        &self.unknown_edge
    }
    pub fn sum_of_hint(&self) -> u32 {
        self.sum_of_hint
    }
}

impl Area {
    fn new(p: Point, puzzle: &Puzzle, side_map: &mut SideMap) -> Area {
        let cp = puzzle.point_to_cellid(p);
        let sum = puzzle.hint(p).unwrap_or(0) as u32;

        let mut edge = vec![];
        if !cp.is_outside() {
            for &r in &Move::ALL_DIRECTIONS {
                let cp_r = puzzle.point_to_cellid(p + r);
                if side_map.get_edge(cp, cp_r) == State::Unknown {
                    edge.push(cp_r);
                }
            }
        } else {
            let points = puzzle.points_in_column(0)
                               .chain(puzzle.points_in_column(puzzle.column() - 1))
                               .chain(puzzle.points_in_row(0))
                               .chain(puzzle.points_in_row(puzzle.row() - 1));
            for p2 in points {
                let cp2 = puzzle.point_to_cellid(p2);
                if side_map.get_edge(cp, cp2) == State::Unknown {
                    edge.push(cp2);
                }
            }
        }
        edge.sort();
        edge.dedup();

        Area {
            coord: cp,
            side: side_map.get_side(cp),
            unknown_edge: edge,
            sum_of_hint: sum,
            size: 1,
        }
    }
}

impl Union for Area {
    fn union(mut lval: Area, mut rval: Area) -> UnionResult<Area> {
        let coord = if lval.coord < rval.coord {
            lval.coord
        } else {
            rval.coord
        };
        let side = match (lval.side, rval.side) {
            (State::Conflict, _) | (_, State::Conflict) => State::Conflict,
            (State::Unknown, x) | (x, State::Unknown) => x,
            (State::Fixed(l), State::Fixed(r)) => {
                if l == r {
                    State::Fixed(l)
                } else {
                    State::Conflict
                }
            }
        };
        let unknown_edge = {
            if lval.unknown_edge.is_empty() {
                rval.unknown_edge
            } else {
                lval.unknown_edge.append(&mut rval.unknown_edge);
                lval.unknown_edge
            }
        };
        let area = Area {
            coord: coord,
            side: side,
            unknown_edge: unknown_edge,
            sum_of_hint: lval.sum_of_hint + rval.sum_of_hint,
            size: lval.size + rval.size,
        };
        if lval.size >= rval.size {
            UnionResult::Left(area)
        } else {
            UnionResult::Right(area)
        }
    }
}

#[derive(Debug)]
pub struct ConnectMap {
    sum_of_hint: u32,
    uf: Uf<Area>,
}

impl Clone for ConnectMap {
    fn clone(&self) -> ConnectMap {
        ConnectMap {
            sum_of_hint: self.sum_of_hint,
            uf: self.uf.clone(),
        }
    }

    fn clone_from(&mut self, other: &ConnectMap) {
        self.sum_of_hint = other.sum_of_hint;
        self.uf.clone_from(&other.uf);
    }
}

impl ConnectMap {
    pub fn new(puzzle: &Puzzle, side_map: &mut SideMap) -> ConnectMap {
        let cell_len = puzzle.cell_len();

        let mut uf = Uf::from_iter((0..cell_len)
                                       .map(CellId::new)
                                       .map(|id| puzzle.cellid_to_point(id))
                                       .map(|p| Area::new(p, puzzle, side_map)));

        let mut sum_of_hint = 0;
        for i in 0..cell_len {
            sum_of_hint += uf.get(i).sum_of_hint;
        }

        let mut conn_map = ConnectMap {
            sum_of_hint: sum_of_hint,
            uf: uf,
        };

        for p in puzzle.points() {
            let cp = puzzle.point_to_cellid(p);
            for &r in &Move::ALL_DIRECTIONS {
                let p2 = p + r;
                let cp2 = puzzle.point_to_cellid(p2);
                if side_map.get_edge(cp, cp2) == State::Fixed(Edge::Cross) {
                    conn_map.union(cp, cp2);
                }
            }
        }
        conn_map
    }

    pub fn cell_len(&self) -> usize {
        self.uf.size()
    }
    pub fn sum_of_hint(&self) -> u32 {
        self.sum_of_hint
    }

    pub fn sync(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        for i in 0..self.cell_len() {
            let c = CellId::new(i);
            update_conn(side_map, self, c);
        }

        let mut closed_cnt = 0;
        for i in 0..self.cell_len() {
            let c = CellId::new(i);
            if try!(update_area(side_map, self, c)) {
                closed_cnt += 1;
            }
        }

        if closed_cnt > 2 {
            return Err(Error::invalid_board());
        }

        Ok(())
    }

    pub fn count_area(&mut self) -> usize {
        (0..self.cell_len())
            .map(CellId::new)
            .filter(|&c| self.get(c).coord() == c)
            .count()
    }

    pub fn union(&mut self, i: CellId, j: CellId) -> bool {
        self.uf.union(i.id(), j.id())
    }
    pub fn get(&mut self, i: CellId) -> &Area {
        self.uf.get(i.id())
    }
    pub fn get_mut(&mut self, i: CellId) -> &mut Area {
        self.uf.get_mut(i.id())
    }
}

fn update_conn(side_map: &mut SideMap, conn_map: &mut ConnectMap, p: CellId) {
    let mut unknown_edge = {
        let a = conn_map.get_mut(p);
        if a.coord != p {
            return;
        }
        mem::replace(&mut a.unknown_edge, vec![])
    };

    for &p2 in &unknown_edge {
        if side_map.get_edge(p, p2) == State::Fixed(Edge::Cross) {
            conn_map.union(p, p2);
        }
    }

    let mut need_update = None;
    {
        let mut area = conn_map.get_mut(p);
        if area.unknown_edge.is_empty() {
            area.unknown_edge = unknown_edge;
        } else {
            area.unknown_edge.append(&mut unknown_edge);
            if area.coord <= p {
                need_update = Some(area.coord);
            }
        }
    }

    if let Some(coord) = need_update {
        update_conn(side_map, conn_map, coord);
    }
}

fn update_area(side_map: &mut SideMap, conn_map: &mut ConnectMap, p: CellId) -> SolverResult<bool> {
    let mut unknown_edge = {
        let a = conn_map.get_mut(p);
        if a.coord != p {
            return Ok(false);
        }
        mem::replace(&mut a.unknown_edge, vec![])
    };

    unsafe {
        // Assume the elements of unknown_edge is copyable.
        let ptr = unknown_edge.as_mut_ptr();
        let mut w = 0;
        for r in 0..unknown_edge.len() {
            let read = ptr.offset(r as isize);
            match side_map.get_edge(p, *read) {
                State::Fixed(_) => {}
                State::Unknown => {
                    let write = ptr.offset(w as isize);
                    *write = conn_map.get(*read).coord();
                    w += 1;
                }
                State::Conflict => {
                    return Err(Error::invalid_board());
                }
            }
        }
        unknown_edge.truncate(w);
    }

    unknown_edge.sort();
    unknown_edge.dedup();

    let is_closed = unknown_edge.is_empty();

    let mut area = conn_map.get_mut(p);
    area.side = side_map.get_side(p);
    area.unknown_edge = unknown_edge;

    Ok(is_closed)
}
