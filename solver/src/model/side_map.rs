use union_find::{UnionFind, UnionBySizeRank as Union, QuickFindUf as Uf};
use slsr_core::board::{Board, Hint, Edge, Side};
use slsr_core::geom::{Geom, Point, Size, Matrix, Move};

use ::{State, LogicError};
use ::model::cell_geom::{CellId, CellGeom, OUTSIDE_CELL_ID};

trait Key {
    fn key0(self) -> usize;
    fn key1(self) -> usize;
}

impl Key for CellId {
    fn key0(self) -> usize { self.id() * 2 }
    fn key1(self) -> usize { self.id() * 2 + 1 }
}

#[derive(Clone, Debug)]
pub struct SideMap {
    hint: Matrix<Hint>,
    uf: Uf<Union>,
    revision: u32
}

impl SideMap {
    pub fn new(hint: Matrix<Hint>) -> SideMap {
        let num_cell = (hint.size().0 * hint.size().1 + 1) as usize;
        SideMap {
            hint: hint,
            uf: UnionFind::new(num_cell * 2),
            revision: 0
        }
    }

    pub fn to_board(&mut self) -> Result<Board, LogicError> {
        let mut board = Board::new(self.size());
        for r in (0 .. self.row()) {
            for c in (0 .. self.column()) {
                let p = Point(r, c);
                let p_u = p + Move::UP;
                let p_l = p + Move::LEFT;

                board[p] = self.hint[p];
                board.side_mut()[p] = try!(self.get_side(p).into());
                board.edge_h_mut()[p] = try!(self.get_edge(p, p_u).into());
                board.edge_v_mut()[p] = try!(self.get_edge(p, p_l).into());
            }

            let p = Point(r, board.column());
            let p_l = p + Move::LEFT;
            board.edge_v_mut()[p] = try!(self.get_edge(p, p_l).into());
        }
        for c in (0 .. board.column()) {
            let p = Point(board.row(), c);
            let p_u = p + Move::UP;
            board.edge_h_mut()[p] = try!(self.get_edge(p, p_u).into());
        }
        Ok(board)
    }

    pub fn hint(&self) -> &Matrix<Hint> { &self.hint }
    pub fn revision(&self) -> u32 { self.revision }
    pub fn all_filled(&self) -> bool {
        self.revision() == (self.row() * self.column()) as u32
    }
}

impl Geom for SideMap {
    fn size(&self) -> Size { self.hint.size() }
}
impl CellGeom for SideMap { }

pub trait SideMapAccess<T> {
    fn get_side(&mut self, p: T) -> State<Side>;
    fn get_edge(&mut self, p0: T, p1: T) -> State<Edge>;

    fn set_outside(&mut self, p: T) -> bool;
    fn set_inside(&mut self, p: T) -> bool;
    fn set_side(&mut self, p: T, ty: Side) -> bool {
        match ty {
            Side::In => self.set_inside(p),
            Side::Out => self.set_outside(p)
        }
    }

    fn set_same(&mut self, p0: T, p1: T) -> bool;
    fn set_different(&mut self, p0: T, p1: T) -> bool;
    fn set_edge(&mut self, p0: T, p1: T, edge: Edge) -> bool {
        match edge {
            Edge::Cross => self.set_same(p0, p1),
            Edge::Line => self.set_different(p0, p1)
        }
    }
}

impl SideMapAccess<CellId> for SideMap {
    fn get_side(&mut self, i: CellId) -> State<Side> {
        let j = OUTSIDE_CELL_ID;

        let p = self.uf.find(i.key0());
        let q = self.uf.find(j.key0());
        let r = self.uf.find(j.key1());

        match (p == q, p == r) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Side::Out),
            (false, true)  => State::Fixed(Side::In),
            (true,  true)  => State::Conflict
        }
    }

    fn get_edge(&mut self, i: CellId, j: CellId) -> State<Edge> {
        let p = self.uf.find(i.key0());
        let q = self.uf.find(j.key0());
        let r = self.uf.find(j.key1());

        match (p == q, p == r) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Edge::Cross),
            (false, true)  => State::Fixed(Edge::Line),
            (true,  true)  => State::Conflict
        }
    }

    fn set_outside(&mut self, i: CellId) -> bool {
        self.set_same(i, OUTSIDE_CELL_ID)
    }
    fn set_inside(&mut self, i: CellId) -> bool {
        self.set_different(i, OUTSIDE_CELL_ID)
    }

    fn set_same(&mut self, i: CellId, j: CellId) -> bool {
        let c1 = self.uf.union(i.key0(), j.key0());
        let c2 = self.uf.union(i.key1(), j.key1());
        if c1 || c2 { self.revision += 1; }
        c1 || c2
    }
    fn set_different(&mut self, i: CellId, j: CellId) -> bool {
        let c1 = self.uf.union(i.key0(), j.key1());
        let c2 = self.uf.union(i.key1(), j.key0());
        if c1 || c2 { self.revision += 1 }
        c1 || c2
    }
}

impl SideMapAccess<Point> for SideMap {
    fn get_side(&mut self, p: Point) -> State<Side> {
        let i = self.cell_id(p);
        self.get_side(i)
    }
    fn get_edge(&mut self, p0: Point, p1: Point) -> State<Edge> {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.get_edge(i, j)
    }

    fn set_outside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.set_outside(i)
    }
    fn set_inside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.set_inside(i)
    }

    fn set_same(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.set_same(i, j)
    }
    fn set_different(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.set_different(i, j)
    }
}

impl<'a> From<&'a Board> for SideMap {
    fn from(board: &'a Board) -> SideMap {
        let mut map = SideMap::new(board.hint().clone());
        for r in (0 .. board.row()) {
            for c in (0 .. board.column()) {
                let p = Point(r, c);
                if let Some(side) = board.side()[p] {
                    map.set_side(p, side);
                }
                if let Some(edge) = board.edge_h()[p] {
                    map.set_edge(p, p + Move::UP, edge);
                }
                if let Some(edge) = board.edge_v()[p] {
                    map.set_edge(p, p + Move::LEFT, edge);
                }
            }
            let p = Point(r, board.column());
            if let Some(edge) = board.edge_v()[p] {
                map.set_edge(p, p + Move::LEFT, edge);
            }
        }
        for c in (0 .. board.column()) {
            let p = Point(board.row(), c);
            if let Some(edge) = board.edge_h()[p] {
                map.set_edge(p, p + Move::UP, edge);
            }
        }
        map
    }
}
