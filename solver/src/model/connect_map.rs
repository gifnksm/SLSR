use std::iter::FromIterator;
use std::mem;
use union_find::{Union, UnionFind, UnionResult, QuickFindUf as Uf};
use slsr_core::puzzle::{Edge, Hint, Side};
use slsr_core::geom::{CellId, Geom, Point, Table, Move, OUTSIDE_CELL_ID};

use ::{Error, State, SolverResult};
use ::model::side_map::SideMap;

#[derive(Clone, Debug)]
pub struct Area {
    coord: CellId,
    side: State<Side>,
    unknown_edge: Vec<CellId>,
    sum_of_hint: u32,
    size: usize
}

impl Area {
    pub fn coord(&self) -> CellId { self.coord }
    pub fn side(&self) -> State<Side> { self.side }
    pub fn unknown_edge(&self) -> &[CellId] { &self.unknown_edge }
    pub fn sum_of_hint(&self) -> u32 { self.sum_of_hint }
}

impl Area {
    fn new(p: Point, hint: &Table<Hint>, side_map: &mut SideMap) -> Area {
        let cp = hint.point_to_cellid(p);
        let sum = hint[p].unwrap_or(0) as u32;

        let mut edge = vec![];
        if cp != OUTSIDE_CELL_ID {
            for &r in &Move::ALL_DIRECTIONS {
                let cp_r = hint.point_to_cellid(p + r);
                if side_map.get_edge(cp, cp_r) == State::Unknown {
                    edge.push(cp_r);
                }
            }
        } else {
            for r in 0..hint.row() {
                for &c in &[0, hint.column() - 1] {
                    let cp2 = hint.point_to_cellid(Point(r, c));
                    if side_map.get_edge(cp, cp2) == State::Unknown {
                        edge.push(cp2);
                    }
                }
            }
            for c in 0..hint.column() {
                for &r in &[0, hint.row() - 1] {
                    let cp2 = hint.point_to_cellid(Point(r, c));
                    if side_map.get_edge(cp, cp2) == State::Unknown {
                        edge.push(cp2);
                    }
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
            size: 1
        }
    }
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
    sum_of_hint: u32,
    uf: Uf<Area>
}

impl ConnectMap {
    pub fn new(hint: &Table<Hint>, side_map: &mut SideMap) -> ConnectMap {
        let size = hint.size();
        let cell_len = size.cell_len();

        let mut uf = Uf::from_iter(
            (0..cell_len)
                .map(|id| size.cellid_to_point(CellId::new(id)))
                .map(|p| Area::new(p, hint, side_map)));

        let mut sum_of_hint = 0;
        for i in 0..cell_len {
            sum_of_hint += uf.get(i).sum_of_hint;
        }

        let mut conn_map = ConnectMap {
            sum_of_hint: sum_of_hint,
            uf: uf
        };

        for r in 0..size.row() {
            for c in 0..size.column() {
                let p = Point(r, c);
                let cp = size.point_to_cellid(p);
                for &r in &Move::ALL_DIRECTIONS {
                    let p2 = p + r;
                    let cp2 = size.point_to_cellid(p2);
                    if side_map.get_edge(cp, cp2) == State::Fixed(Edge::Cross) {
                        conn_map.union(cp, cp2);
                    }
                }
            }
        }
        conn_map
    }

    pub fn cell_len(&self) -> usize { self.uf.size() }
    pub fn sum_of_hint(&self) -> u32 { self.sum_of_hint }

    pub fn sync(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        let mut updated = true;
        while updated {
            updated = false;
            for i in 0..self.cell_len() {
                let c = CellId::new(i);
                updated |= try!(update_conn(side_map, self, c));
            }
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

fn filter_edge(side_map: &mut SideMap, p: CellId, edge: Vec<CellId>)
               -> SolverResult<(Vec<CellId>, Vec<CellId>)>
{
    let mut unknown = vec![];
    let mut same = vec![];

    for p2 in edge {
        match side_map.get_edge(p, p2) {
            State::Fixed(Edge::Cross) => same.push(p2),
            State::Fixed(Edge::Line) => {}
            State::Unknown => unknown.push(p2),
            State::Conflict => return Err(Error::invalid_board())
        }
    }

    unknown.sort();
    unknown.dedup();
    same.sort();
    same.dedup();
    Ok((same, unknown))
}

fn update_conn(side_map: &mut SideMap, conn_map: &mut ConnectMap, p: CellId)
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
