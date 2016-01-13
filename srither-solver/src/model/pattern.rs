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

pub enum PatternMatchResult<T> {
    Complete,
    Partial(T),
    Conflict,
}

pub trait Transform {
    fn rotate(self, rot: Rotation) -> Self;
    fn shift(self, d: Move) -> Self;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Pattern {
    Hint(HintPattern),
    Edge(EdgePattern<Point>),
}

impl Pattern {
    pub fn hint(h: u8, p: Point) -> Pattern {
        Pattern::Hint(HintPattern::new(h, p))
    }

    pub fn cross(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(EdgePattern::cross(p0, p1))
    }

    pub fn line(p0: Point, p1: Point) -> Pattern {
        Pattern::Edge(EdgePattern::line(p0, p1))
    }

    pub fn matches(self,
                   puzzle: &Puzzle,
                   side_map: &mut SideMap)
                   -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        match self {
            Pattern::Hint(h) => h.matches(puzzle),
            Pattern::Edge(e) => e.matches(puzzle, side_map),
        }
    }
}

impl Transform for Pattern {
    fn rotate(self, rot: Rotation) -> Pattern {
        match self {
            Pattern::Hint(h) => Pattern::Hint(h.rotate(rot)),
            Pattern::Edge(e) => Pattern::Edge(e.rotate(rot)),
        }
    }

    fn shift(self, d: Move) -> Pattern {
        match self {
            Pattern::Hint(h) => Pattern::Hint(h.shift(d)),
            Pattern::Edge(e) => Pattern::Edge(e.shift(d)),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HintPattern {
    hint: u8,
    point: Point,
}

impl HintPattern {
    fn new(h: u8, p: Point) -> HintPattern {
        HintPattern {
            hint: h,
            point: p,
        }
        .normalized()
    }

    pub fn hint(&self) -> u8 {
        self.hint
    }

    pub fn point(&self) -> Point {
        self.point
    }

    fn normalized(self) -> HintPattern {
        self
    }

    pub fn matches<T>(self, puzzle: &Puzzle) -> SolverResult<PatternMatchResult<T>> {
        if puzzle.hint(self.point) == Some(self.hint) {
            Ok(PatternMatchResult::Complete)
        } else {
            Ok(PatternMatchResult::Conflict)
        }
    }
}

impl Transform for HintPattern {
    fn rotate(mut self, rot: Rotation) -> HintPattern {
        let o = Point(0, 0);
        let p = self.point;
        self.point = o + rot * (p - o);
        self.normalized()
    }

    fn shift(mut self, d: Move) -> HintPattern {
        let p = self.point;
        self.point = p + d;
        self.normalized()
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
    fn cross(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern {
            edge: Edge::Cross,
            points: (p0, p1),
        }
        .normalized()
    }

    fn line(p0: Point, p1: Point) -> EdgePattern<Point> {
        EdgePattern {
            edge: Edge::Line,
            points: (p0, p1),
        }
        .normalized()
    }

    fn normalized(self) -> EdgePattern<Point> {
        let mut points = self.points;
        if self.points.1 < self.points.0 {
            points = (self.points.1, self.points.0);
        }
        EdgePattern {
            edge: self.edge,
            points: points,
        }
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
                   -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        self.to_cellid(puzzle).matches(side_map)
    }
}

impl EdgePattern<CellId> {
    pub fn matches(self,
                   side_map: &mut SideMap)
                   -> SolverResult<PatternMatchResult<EdgePattern<CellId>>> {
        let ps = self.points;
        match side_map.get_edge(ps.0, ps.1) {
            State::Fixed(edg) => {
                if self.edge == edg {
                    Ok(PatternMatchResult::Complete)
                } else {
                    Ok(PatternMatchResult::Conflict)
                }
            }
            State::Unknown => Ok(PatternMatchResult::Partial(self)),
            State::Conflict => Err(Error::invalid_board()),
        }
    }

    pub fn apply(&self, side_map: &mut SideMap) {
        let ps = self.points;
        let _ = side_map.set_edge(ps.0, ps.1, self.edge);
    }
}

impl Transform for EdgePattern<Point> {
    fn rotate(mut self, rot: Rotation) -> EdgePattern<Point> {
        let o = Point(0, 0);
        let ps = self.points;
        self.points = (o + rot * (ps.0 - o), o + rot * (ps.1 - o));
        self.normalized()
    }

    fn shift(mut self, d: Move) -> EdgePattern<Point> {
        let ps = self.points;
        self.points = (ps.0 + d, ps.1 + d);
        self.normalized()
    }
}
