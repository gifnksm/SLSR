use std::iter::FromIterator;
use std::mem;
use union_find::{Union, UnionFind, UnionResult, QuickFindUf as Uf};
use slsr_core::board::{Edge, Side};
use slsr_core::geom::{Geom, Point, Size, Move};

use ::{LogicError, State, SolverResult};
use ::model::cell_geom::{CellId, CellGeom};
use ::model::side_map::{SideMap, SideMapAccess};

#[derive(Clone, Debug)]
pub struct Area {
    coord: Point,
    side: State<Side>,
    unknown_edge: Vec<Point>,
    sum_of_hint: u32,
    size: usize
}

impl Area {
    pub fn coord(&self) -> Point { self.coord }
    pub fn side(&self) -> State<Side> { self.side }
    pub fn unknown_edge(&self) -> &[Point] { &self.unknown_edge }
    pub fn sum_of_hint(&self) -> u32 { self.sum_of_hint }
}

impl Union for Area {
    fn union(lval: Area, rval: Area) -> UnionResult<Area> {
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
            let mut v = Vec::from(lval.unknown_edge);
            v.extend(&rval.unknown_edge);
            v
        };
        let area = Area {
            coord: coord,
            side: side,
            unknown_edge: unknown_edge,
            sum_of_hint: lval.sum_of_hint + rval.sum_of_hint,
            size: lval.size + rval.size
        };
        if lval.size >= rval.size {
            UnionResult::Left(area)
        } else {
            UnionResult::Right(area)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectMap {
    size: Size,
    uf: Uf<Area>
}

impl ConnectMap {
    fn new<F>(size: Size, mut f: F) -> ConnectMap
        where F: FnMut(Point) -> Area
    {
        let len = (size.0 * size.1 + 1) as usize;
        ConnectMap {
            size: size,
            uf: FromIterator::from_iter((0 .. len).map(|i| {
                let p = if i == 0 {
                    Point(-1, -1)
                } else {
                    size.index_to_point(i - 1)
                };
                f(p)
            }))
        }
    }

    pub fn sync(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        let rev = side_map.revision();
        loop {
            let mut updated = false;
            for r in (0 .. side_map.row()) {
                for c in (0 .. side_map.column()) {
                    updated |= try!(update_conn(side_map, self, Point(r, c)));
                }
            }
            updated |= try!(update_conn(side_map, self, Point(-1, -1)));

            if updated {
                debug_assert_eq!(rev, side_map.revision());
                continue
            }

            break
        }
        Ok(())
    }

    pub fn count_area(&mut self) -> usize {
        let mut cnt = 1; // counts (-1, -1)
        for r in (0 .. self.row()) {
            for c in (0 .. self.column()) {
                let p = Point(r, c);
                if p == self.get(p).coord() {
                    cnt += 1;
                }
            }
        }
        cnt
    }
}

impl Geom for ConnectMap {
    fn size(&self) -> Size { self.size }
}

impl CellGeom for ConnectMap {}

pub trait ConnectMapAccess<T> {
    fn union(&mut self, p0: T, p1: T) -> bool;
    fn get(&mut self, p: T) -> &Area;
    fn get_mut(&mut self, p: T) -> &mut Area;
}

impl ConnectMapAccess<CellId> for ConnectMap {
    fn union(&mut self, i: CellId, j: CellId) -> bool {
        self.uf.union(i.id(), j.id())
    }
    fn get(&mut self, i: CellId) -> &Area {
        self.uf.get(i.id())
    }
    fn get_mut(&mut self, i: CellId) -> &mut Area {
        self.uf.get_mut(i.id())
    }
}

impl ConnectMapAccess<Point> for ConnectMap {
    fn union(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.union(i, j)
    }

    fn get(&mut self, p: Point) -> &Area {
        let i = self.cell_id(p);
        self.get(i)
    }

    fn get_mut(&mut self, p: Point) -> &mut Area {
        let i = self.cell_id(p);
        self.get_mut(i)
    }
}

impl<'a> From<&'a mut SideMap> for ConnectMap {
    fn from(side_map: &'a mut SideMap) -> ConnectMap {
        let mut conn_map = ConnectMap::new(side_map.size(), |p| {
            let sum = side_map.hint()[p].unwrap_or(0);

            let mut edge = vec![];
            if side_map.contains(p) {
                for &r in &Move::ALL_DIRECTIONS {
                    if side_map.get_edge(p, p + r) == State::Unknown {
                        edge.push(p + r);
                    }
                }
            } else {
                for r in 0 .. side_map.row() {
                    for &c in &[0, side_map.column() - 1] {
                        let p2 = Point(r, c);
                        if side_map.get_edge(p, p2) == State::Unknown {
                            edge.push(p2);
                        }
                    }
                }
                for c in 0 .. side_map.column() {
                    for &r in &[0, side_map.row() - 1] {
                        let p2 = Point(r, c);
                        if side_map.get_edge(p, p2) == State::Unknown {
                            edge.push(p2);
                        }
                    }
                }
            }
            edge.sort();
            edge.dedup();

            Area {
                coord: p,
                side: side_map.get_side(p),
                unknown_edge: edge,
                sum_of_hint: sum as u32,
                size: 1
            }
        });

        for r in (0 .. side_map.row()) {
            for c in 0 .. side_map.column() {
                let p = Point(r, c);
                for &r in &Move::ALL_DIRECTIONS {
                    let p2 = p + r;
                    if side_map.get_edge(p, p2) == State::Fixed(Edge::Cross) {
                        conn_map.union(p, p2);
                    }
                }
            }
        }
        conn_map
    }
}

fn filter_edge(side_map: &mut SideMap, p: Point, edge: Vec<Point>)
               -> SolverResult<(Vec<Point>, Vec<Point>)>
{
    let mut unknown = vec![];
    let mut same = vec![];

    for p2 in edge {
        match side_map.get_edge(p, p2) {
            State::Fixed(Edge::Cross) => same.push(p2),
            State::Fixed(Edge::Line) => {}
            State::Unknown => unknown.push(p2),
            State::Conflict => return Err(LogicError)
        }
    }

    unknown.sort();
    unknown.dedup();
    same.sort();
    same.dedup();
    Ok((same, unknown))
}

fn update_conn(side_map: &mut SideMap, conn_map: &mut ConnectMap, p: Point)
               -> SolverResult<bool>
{
    let mut edge = {
        let a = conn_map.get_mut(p);
        if a.coord != p { return Ok(false) }
        mem::replace(&mut a.unknown_edge, vec![])
    };
    for p in edge.iter_mut() {
        *p = conn_map.get(*p).coord;
    }

    let (same, unknown) = try!(filter_edge(side_map, p, edge));
    {
        let a = conn_map.get_mut(p);
        a.side = side_map.get_side(p);
        a.unknown_edge = unknown;
    }

    let mut ret = false;
    for &p2 in &same {
        ret |= conn_map.union(p, p2);
    }
    Ok(ret)
}
