// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use union_find::{UnionFind, UnionBySizeRank as Union, QuickFindUf as Uf};
use srither_core::puzzle::{Puzzle, Edge, Side};
use srither_core::geom::{CellId, Geom, Move};

use SolverResult;
use model::State;

pub trait Key {
    fn key0(self) -> usize;
    fn key1(self) -> usize;
}

impl Key for CellId {
    fn key0(self) -> usize {
        self.id() * 2
    }
    fn key1(self) -> usize {
        self.id() * 2 + 1
    }
}

impl Into<KeyPair> for CellId {
    fn into(self) -> KeyPair {
        KeyPair(self.key0(), self.key1())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct KeyPair(usize, usize);

impl Key for KeyPair {
    fn key0(self) -> usize {
        self.0
    }

    fn key1(self) -> usize {
        self.1
    }
}

// FIXME: use const fn (OUTSIDE_CELL_ID.key0())
const OUTSIDE_KEY0: usize = 0;
const OUTSIDE_KEY1: usize = 1;

#[derive(Debug)]
pub struct SideMap {
    uf: Uf<Union>,
    revision: u32,
    max_revision: u32,
}

impl Clone for SideMap {
    fn clone(&self) -> SideMap {
        SideMap {
            uf: self.uf.clone(),
            revision: self.revision,
            max_revision: self.max_revision,
        }
    }

    fn clone_from(&mut self, other: &SideMap) {
        self.uf.clone_from(&other.uf);
        self.revision = other.revision;
        self.max_revision = other.max_revision;
    }
}

impl SideMap {
    pub fn new(puzzle: &Puzzle) -> SideMap {
        let num_cell = puzzle.cell_len();
        let max_revision = (puzzle.row() * puzzle.column()) as u32;
        SideMap {
            uf: UnionFind::new(num_cell * 2),
            revision: 0,
            max_revision: max_revision,
        }
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn all_filled(&self) -> bool {
        self.revision() == self.max_revision
    }

    pub fn get_side(&mut self, p: CellId) -> State<Side> {
        let a = self.uf.find(p.key0());
        let b = self.uf.find(OUTSIDE_KEY0);
        let c = self.uf.find(OUTSIDE_KEY1);

        match (a == b, a == c) {
            (false, false) => State::Unknown,
            (true, false) => State::Fixed(Side::Out),
            (false, true) => State::Fixed(Side::In),
            (true, true) => State::Conflict,
        }
    }

    pub fn get_edge<T>(&mut self, p0: T, p1: T) -> State<Edge>
        where T: Key + Copy
    {
        let a = self.uf.find(p0.key0());
        let b = self.uf.find(p1.key0());
        let c = self.uf.find(p1.key1());

        match (a == b, a == c) {
            (false, false) => State::Unknown,
            (true, false) => State::Fixed(Edge::Cross),
            (false, true) => State::Fixed(Edge::Line),
            (true, true) => State::Conflict,
        }
    }

    pub fn set_outside(&mut self, p: CellId) -> bool {
        self.set_same(p, CellId::OUTSIDE)
    }
    pub fn set_inside(&mut self, p: CellId) -> bool {
        self.set_different(p, CellId::OUTSIDE)
    }
    pub fn set_side(&mut self, p: CellId, ty: Side) -> bool {
        match ty {
            Side::In => self.set_inside(p),
            Side::Out => self.set_outside(p),
        }
    }

    pub fn set_same(&mut self, p0: CellId, p1: CellId) -> bool {
        let c1 = self.uf.union(p0.key0(), p1.key0());
        let c2 = self.uf.union(p0.key1(), p1.key1());
        if c1 || c2 {
            self.revision += 1;
        }
        c1 || c2
    }
    pub fn set_different(&mut self, p0: CellId, p1: CellId) -> bool {
        let c1 = self.uf.union(p0.key0(), p1.key1());
        let c2 = self.uf.union(p0.key1(), p1.key0());
        if c1 || c2 {
            self.revision += 1
        }
        c1 || c2
    }
    pub fn set_edge(&mut self, p0: CellId, p1: CellId, edge: Edge) -> bool {
        match edge {
            Edge::Cross => self.set_same(p0, p1),
            Edge::Line => self.set_different(p0, p1),
        }
    }

    pub fn complete_puzzle(&mut self, puzzle: &mut Puzzle) -> SolverResult<()> {
        for p in puzzle.points() {
            let cp = puzzle.point_to_cellid(p);
            let cp_u = puzzle.point_to_cellid(p + Move::UP);
            let cp_l = puzzle.point_to_cellid(p + Move::LEFT);

            puzzle.set_side(p, try!(self.get_side(cp).into()));
            puzzle.set_edge_h(p, try!(self.get_edge(cp, cp_u).into()));
            puzzle.set_edge_v(p, try!(self.get_edge(cp, cp_l).into()));
        }

        for p in puzzle.points_in_column(puzzle.column()) {
            let cp = puzzle.point_to_cellid(p);
            let cp_l = puzzle.point_to_cellid(p + Move::LEFT);

            puzzle.set_edge_v(p, try!(self.get_edge(cp, cp_l).into()));
        }

        for p in puzzle.points_in_row(puzzle.row()) {
            let cp = puzzle.point_to_cellid(p);
            let cp_u = puzzle.point_to_cellid(p + Move::UP);

            puzzle.set_edge_h(p, try!(self.get_edge(cp, cp_u).into()));
        }
        Ok(())
    }
}

impl<'a> From<&'a Puzzle> for SideMap {
    fn from(puzzle: &'a Puzzle) -> SideMap {
        let mut map = SideMap::new(puzzle);
        for p in puzzle.points() {
            let cp = puzzle.point_to_cellid(p);
            let cp_u = puzzle.point_to_cellid(p + Move::UP);
            let cp_l = puzzle.point_to_cellid(p + Move::LEFT);

            if let Some(side) = puzzle.side(p) {
                map.set_side(cp, side);
            }
            if let Some(edge) = puzzle.edge_h(p) {
                map.set_edge(cp, cp_u, edge);
            }
            if let Some(edge) = puzzle.edge_v(p) {
                map.set_edge(cp, cp_l, edge);
            }
        }

        for p in puzzle.points_in_column(puzzle.column()) {
            let cp = puzzle.point_to_cellid(p);
            let cp_l = puzzle.point_to_cellid(p + Move::LEFT);

            if let Some(edge) = puzzle.edge_v(p) {
                map.set_edge(cp, cp_l, edge);
            }
        }
        for p in puzzle.points_in_row(puzzle.row()) {
            let cp = puzzle.point_to_cellid(p);
            let cp_u = puzzle.point_to_cellid(p + Move::UP);

            if let Some(edge) = puzzle.edge_h(p) {
                map.set_edge(cp, cp_u, edge);
            }
        }
        map
    }
}
