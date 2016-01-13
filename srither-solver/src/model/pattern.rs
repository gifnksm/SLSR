// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use srither_core::puzzle::{Edge, Puzzle};
use srither_core::geom::{CellId, Geom, Point, Rotation, Move};

use {Error, SolverResult};
use model::{SideMap, State};

pub enum MatchResult<T> {
    Complete,
    Partial(T),
    Conflict,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HintPattern {
    hint: u8,
    point: Point,
}

impl HintPattern {
    pub fn new(h: u8, p: Point) -> HintPattern {
        HintPattern {
            hint: h,
            point: p,
        }
    }

    pub fn hint(&self) -> u8 {
        self.hint
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn rotate(self, rot: Rotation) -> HintPattern {
        let o = Point(0, 0);
        let p = self.point;
        Self::new(self.hint, o + rot * (p - o))
    }

    pub fn shift(self, d: Move) -> HintPattern {
        let p = self.point;
        Self::new(self.hint, p + d)
    }

    pub fn matches<T>(self, puzzle: &Puzzle) -> SolverResult<MatchResult<T>> {
        if puzzle.hint(self.point) == Some(self.hint) {
            Ok(MatchResult::Complete)
        } else {
            Ok(MatchResult::Conflict)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EdgePattern<P> {
    edge: Edge,
    points: (P, P),
}

impl<P> EdgePattern<P> {
    pub fn edge(&self) -> Edge {
        self.edge
    }

    pub fn points(&self) -> (P, P)
        where P: Copy
    {
        self.points
    }
}

impl EdgePattern<Point> {
    fn new(edge: Edge, p0: Point, p1: Point) -> EdgePattern<Point> {
        let points = if p0 <= p1 {
            (p0, p1)
        } else {
            (p1, p0)
        };

        EdgePattern {
            edge: edge,
            points: points,
        }
    }

    pub fn cross(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern::new(Edge::Cross, p0, p1)
    }

    pub fn line(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern::new(Edge::Line, p0, p1)
    }

    pub fn rotate(self, rot: Rotation) -> EdgePattern<Point> {
        let o = Point(0, 0);
        let ps = self.points;
        Self::new(self.edge, o + rot * (ps.0 - o), o + rot * (ps.1 - o))
    }

    pub fn shift(self, d: Move) -> EdgePattern<Point> {
        let ps = self.points;
        Self::new(self.edge, ps.0 + d, ps.1 + d)
    }

    pub fn to_cellid(self, puzzle: &Puzzle) -> EdgePattern<CellId> {
        let p0 = puzzle.point_to_cellid(self.points.0);
        let p1 = puzzle.point_to_cellid(self.points.1);
        EdgePattern {
            edge: self.edge,
            points: (p0, p1),
        }
    }

    pub fn matches(self,
                   puzzle: &Puzzle,
                   side_map: &mut SideMap)
                   -> SolverResult<MatchResult<EdgePattern<CellId>>> {
        self.to_cellid(puzzle).matches(side_map)
    }
}

impl EdgePattern<CellId> {
    pub fn matches(self, side_map: &mut SideMap) -> SolverResult<MatchResult<EdgePattern<CellId>>> {
        let ps = self.points;
        match side_map.get_edge(ps.0, ps.1) {
            State::Fixed(edg) => {
                if self.edge == edg {
                    Ok(MatchResult::Complete)
                } else {
                    Ok(MatchResult::Conflict)
                }
            }
            State::Unknown => Ok(MatchResult::Partial(self)),
            State::Conflict => Err(Error::invalid_board()),
        }
    }

    pub fn apply(&self, side_map: &mut SideMap) {
        let ps = self.points;
        let _ = side_map.set_edge(ps.0, ps.1, self.edge);
    }
}
