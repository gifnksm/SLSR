use std::iter::FromIterator;
use std::mem;
use union_find::{Union, UnionFind, UnionResult, QuickFindUf as Uf};
use slsr_core::board::{Edge, Side};
use slsr_core::geom::{CellId, Geom, Point, Size, Move, OUTSIDE_CELL_ID};

use ::{LogicError, State, SolverResult};
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
    fn new<F>(size: Size, f: F) -> ConnectMap
        where F: FnMut(Point) -> Area
    {
        let len = size.cell_len();
        let it = (0..len).
            map(|id| size.cellid_to_point(CellId::new(id)))
            .map(f);

        ConnectMap {
            size: size,
            uf: FromIterator::from_iter(it)
        }
    }

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

impl Geom for ConnectMap {
    fn size(&self) -> Size { self.size }
}

impl<'a> From<&'a mut SideMap> for ConnectMap {
    fn from(side_map: &'a mut SideMap) -> ConnectMap {
        let mut conn_map = ConnectMap::new(side_map.size(), |p| {
            let cp = side_map.point_to_cellid(p);
            let sum = side_map.hint()[p].unwrap_or(0);

            let mut edge = vec![];
            if cp != OUTSIDE_CELL_ID {
                for &r in &Move::ALL_DIRECTIONS {
                    let cp_r = side_map.point_to_cellid(p + r);
                    if side_map.get_edge(cp, cp_r) == State::Unknown {
                        edge.push(cp_r);
                    }
                }
            } else {
                for r in 0..side_map.row() {
                    for &c in &[0, side_map.column() - 1] {
                        let cp2 = side_map.point_to_cellid(Point(r, c));
                        if side_map.get_edge(cp, cp2) == State::Unknown {
                            edge.push(cp2);
                        }
                    }
                }
                for c in 0..side_map.column() {
                    for &r in &[0, side_map.row() - 1] {
                        let cp2 = side_map.point_to_cellid(Point(r, c));
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
                sum_of_hint: sum as u32,
                size: 1
            }
        });

        for r in 0..side_map.row() {
            for c in 0..side_map.column() {
                let p = Point(r, c);
                let cp = side_map.point_to_cellid(p);
                for &r in &Move::ALL_DIRECTIONS {
                    let p2 = p + r;
                    let cp2 = side_map.point_to_cellid(p2);
                    if side_map.get_edge(cp, cp2) == State::Fixed(Edge::Cross) {
                        conn_map.union(cp, cp2);
                    }
                }
            }
        }
        conn_map
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
            State::Conflict => return Err(LogicError)
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
