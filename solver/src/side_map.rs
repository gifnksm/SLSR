use union_find::{UnionFind, UnionBySizeRank as Union, QuickFindUf as Uf};
use slsr_core::board::{Board, Hint, Edge, Side};
use slsr_core::geom::{Geom, Point, Size, Matrix, Move};

use super::{State, LogicError};

const OUTSIDE_CELL_ID: CellId = CellId(0);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct CellId(usize);

impl CellId {
    fn key0(self) -> usize { self.0 * 2 }
    fn key1(self) -> usize { self.0 * 2 + 1 }
}

#[derive(Clone, Debug)]
struct SideMapInner {
    uf: Uf<Union>,
    revision: u32
}

impl SideMapInner {
    fn new(size: usize) -> SideMapInner {
        SideMapInner { uf: UnionFind::new(size * 2), revision: 0 }
    }

    fn revision(&self) -> u32 { self.revision }

    fn set_same(&mut self, i: CellId, j: CellId) -> bool {
        let c1 = self.uf.union(i.key0(), j.key0());
        let c2 = self.uf.union(i.key1(), j.key1());
        if c1 || c2 { self.revision += 1 }
        c1 || c2
    }
    fn set_different(&mut self, i: CellId, j: CellId) -> bool {
        let c1 = self.uf.union(i.key0(), j.key1());
        let c2 = self.uf.union(i.key1(), j.key0());
        if c1 || c2 { self.revision += 1 }
        c1 || c2
    }
}

#[derive(Clone, Debug)]
pub struct SideMap {
    hint: Matrix<Hint>,
    inner: SideMapInner
}

impl SideMap {
    pub fn new(hint: Matrix<Hint>) -> SideMap {
        let num_cell = (hint.size().0 * hint.size().1 + 1) as usize;
        SideMap {
            hint: hint,
            inner: SideMapInner::new(num_cell)
        }
    }

    pub fn from_board(board: &Board) -> SideMap {
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

    pub fn to_board(&mut self) -> Result<Board, LogicError> {
        let mut board = Board::new(self.size());
        for r in (0 .. self.row()) {
            for c in (0 .. self.column()) {
                let p = Point(r, c);

                board[p] = self.hint[p];
                board.side_mut()[p] = try!(self.get_side(p).into_option());
                board.edge_h_mut()[p] = try!(self.get_edge(p, p + Move::UP).into_option());
                board.edge_v_mut()[p] = try!(self.get_edge(p, p + Move::LEFT).into_option());
            }
            let p = Point(r, board.column());
            board.edge_v_mut()[p] = try!(self.get_edge(p, p + Move::LEFT).into_option());
        }
        for c in (0 .. board.column()) {
            let p = Point(board.row(), c);
            board.edge_h_mut()[p] = try!(self.get_edge(p, p + Move::UP).into_option());
        }
        Ok(board)
    }

    pub fn get_side(&mut self, p: Point) -> State<Side> {
        let i = self.cell_id(p);
        let j = OUTSIDE_CELL_ID;

        let p = self.inner.uf.find(i.key0());
        let q = self.inner.uf.find(j.key0());
        let r = self.inner.uf.find(j.key1());

        match (p == q, p == r) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Side::Out),
            (false, true)  => State::Fixed(Side::In),
            (true,  true)  => State::Conflict
        }
    }
    pub fn get_edge(&mut self, p0: Point, p1: Point) -> State<Edge> {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);

        let p = self.inner.uf.find(i.key0());
        let q = self.inner.uf.find(j.key0());
        let r = self.inner.uf.find(j.key1());

        match (p == q, p == r) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Edge::Cross),
            (false, true)  => State::Fixed(Edge::Line),
            (true,  true)  => State::Conflict
        }
    }

    pub fn set_outside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.inner.set_same(i, OUTSIDE_CELL_ID)
    }
    pub fn set_inside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.inner.set_different(i, OUTSIDE_CELL_ID)
    }
    pub fn set_side(&mut self, p: Point, ty: Side) -> bool {
        match ty {
            Side::In  => self.set_inside(p),
            Side::Out => self.set_outside(p),
        }
    }

    pub fn set_same(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.inner.set_same(i, j)
    }
    pub fn set_different(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.inner.set_different(i, j)
    }
    pub fn set_edge(&mut self, p0: Point, p1: Point, edge: Edge) -> bool {
        match edge {
            Edge::Cross => self.set_same(p0, p1),
            Edge::Line  => self.set_different(p0, p1)
        }
    }

    pub fn hint(&self) -> &Matrix<Hint> { &self.hint }
    pub fn revision(&self) -> u32 { self.inner.revision() }
    pub fn all_filled(&self) -> bool {
        self.inner.revision() == (self.row() * self.column()) as u32
    }

    fn cell_id(&self, p: Point) -> CellId {
        if self.contains(p) {
            CellId(self.point_to_index(p) + 1)
        } else {
            OUTSIDE_CELL_ID
        }
    }
}

impl Geom for SideMap {
    fn size(&self) -> Size { self.hint.size() }
}

