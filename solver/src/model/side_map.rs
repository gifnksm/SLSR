use union_find::{UnionFind, UnionBySizeRank as Union, QuickFindUf as Uf};
use slsr_core::board::{Board, Hint, Edge, Side};
use slsr_core::geom::{CellId, Geom, Point, Size, Table, Move, OUTSIDE_CELL_ID};

use ::{State, LogicError};

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
    hint: Table<Hint>,
    uf: Uf<Union>,
    revision: u32
}

impl SideMap {
    pub fn new(hint: Table<Hint>) -> SideMap {
        let num_cell = (hint.row() * hint.column() + 1) as usize;
        SideMap {
            hint: hint,
            uf: UnionFind::new(num_cell * 2),
            revision: 0
        }
    }

    pub fn hint(&self) -> &Table<Hint> { &self.hint }
    pub fn revision(&self) -> u32 { self.revision }
    pub fn all_filled(&self) -> bool {
        self.revision() == (self.row() * self.column()) as u32
    }

    pub fn get_side(&mut self, p: CellId) -> State<Side> {
        let q = OUTSIDE_CELL_ID;

        let a = self.uf.find(p.key0());
        let b = self.uf.find(q.key0());
        let c = self.uf.find(q.key1());

        match (a == b, a == c) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Side::Out),
            (false, true)  => State::Fixed(Side::In),
            (true,  true)  => State::Conflict
        }
    }

    pub fn get_edge(&mut self, p0: CellId, p1: CellId) -> State<Edge> {
        let a = self.uf.find(p0.key0());
        let b = self.uf.find(p1.key0());
        let c = self.uf.find(p1.key1());

        match (a == b, a == c) {
            (false, false) => State::Unknown,
            (true,  false) => State::Fixed(Edge::Cross),
            (false, true)  => State::Fixed(Edge::Line),
            (true,  true)  => State::Conflict
        }
    }

    pub fn set_outside(&mut self, p: CellId) -> bool {
        self.set_same(p, OUTSIDE_CELL_ID)
    }
    pub fn set_inside(&mut self, p: CellId) -> bool {
        self.set_different(p, OUTSIDE_CELL_ID)
    }
    pub fn set_side(&mut self, p: CellId, ty: Side) -> bool {
        match ty {
            Side::In => self.set_inside(p),
            Side::Out => self.set_outside(p)
        }
    }

    pub fn set_same(&mut self, p0: CellId, p1: CellId) -> bool {
        let c1 = self.uf.union(p0.key0(), p1.key0());
        let c2 = self.uf.union(p0.key1(), p1.key1());
        if c1 || c2 { self.revision += 1; }
        c1 || c2
    }
    pub fn set_different(&mut self, p0: CellId, p1: CellId) -> bool {
        let c1 = self.uf.union(p0.key0(), p1.key1());
        let c2 = self.uf.union(p0.key1(), p1.key0());
        if c1 || c2 { self.revision += 1 }
        c1 || c2
    }
    pub fn set_edge(&mut self, p0: CellId, p1: CellId, edge: Edge) -> bool {
        match edge {
            Edge::Cross => self.set_same(p0, p1),
            Edge::Line => self.set_different(p0, p1)
        }
    }
}

impl Geom for SideMap {
    fn size(&self) -> Size { self.hint.size() }
}

impl<'a> From<&'a Board> for SideMap {
    fn from(board: &'a Board) -> SideMap {
        let mut map = SideMap::new(board.hint().clone());
        for r in 0..board.row() {
            for c in 0..board.column() {
                let p = Point(r, c);
                let cp = board.point_to_cellid(p);
                let cp_u = board.point_to_cellid(p + Move::UP);
                let cp_l = board.point_to_cellid(p + Move::LEFT);

                if let Some(side) = board.side()[p] {
                    map.set_side(cp, side);
                }
                if let Some(edge) = board.edge_h()[p] {
                    map.set_edge(cp, cp_u, edge);
                }
                if let Some(edge) = board.edge_v()[p] {
                    map.set_edge(cp, cp_l, edge);
                }
            }

            let p = Point(r, board.column());
            let cp = board.point_to_cellid(p);
            let cp_l = board.point_to_cellid(p + Move::LEFT);

            if let Some(edge) = board.edge_v()[p] {
                map.set_edge(cp, cp_l, edge);
            }
        }
        for c in 0..board.column() {
            let p = Point(board.row(), c);
            let cp = board.point_to_cellid(p);
            let cp_u = board.point_to_cellid(p + Move::UP);

            if let Some(edge) = board.edge_h()[p] {
                map.set_edge(cp, cp_u, edge);
            }
        }
        map
    }
}

impl Into<Result<Board, LogicError>> for SideMap {
    fn into(mut self) -> Result<Board, LogicError> {
        let mut board = Board::new(self.size());
        for r in 0..self.row() {
            for c in 0..self.column() {
                let p = Point(r, c);
                let cp = self.point_to_cellid(p);
                let cp_u = self.point_to_cellid(p + Move::UP);
                let cp_l = self.point_to_cellid(p + Move::LEFT);

                board.hint_mut()[p] = self.hint[p];
                board.side_mut()[p] = try!(self.get_side(cp).into());
                board.edge_h_mut()[p] = try!(self.get_edge(cp, cp_u).into());
                board.edge_v_mut()[p] = try!(self.get_edge(cp, cp_l).into());
            }

            let p = Point(r, board.column());
            let cp = self.point_to_cellid(p);
            let cp_l = self.point_to_cellid(p + Move::LEFT);

            board.edge_v_mut()[p] = try!(self.get_edge(cp, cp_l).into());
        }

        for c in 0..board.column() {
            let p = Point(board.row(), c);
            let cp = self.point_to_cellid(p);
            let cp_u = self.point_to_cellid(p + Move::UP);

            board.edge_h_mut()[p] = try!(self.get_edge(cp, cp_u).into());
        }
        Ok(board)
    }
}
