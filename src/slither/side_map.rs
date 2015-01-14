use union_find::UnionFind;
use hint::{self, Cell, Edge, Hint};
use geom::{Geom, Point, Size, Matrix, LEFT, UP};

const OUTSIDE_CELL_ID: CellId = CellId(0);

#[derive(Copy, Clone, Eq, PartialEq, Show)]
struct CellId(usize);

impl CellId {
    fn key0(self) -> usize { self.0 * 2 }
    fn key1(self) -> usize { self.0 * 2 + 1 }
}

#[derive(Copy, Clone, Eq, PartialEq, Show)]
pub enum Relation {
    Same, Different, Unknown, Conflict
}

#[derive(Copy, Clone, Eq, PartialEq, Show)]
pub enum Side {
    In, Out, Unknown, Conflict
}

#[derive(Clone)]
struct SideMapInner {
    uf: UnionFind,
    revision: u32
}

impl SideMapInner {
    fn new(size: usize) -> SideMapInner {
        SideMapInner { uf: UnionFind::new(size * 2), revision: 0 }
    }

    fn revision(&self) -> u32 { self.revision }

    fn is_same(&mut self, i: CellId, j: CellId) -> bool {
        self.uf.find(i.key0(), j.key0())
    }
    fn is_different(&mut self, i: CellId, j: CellId) -> bool {
        self.uf.find(i.key0(), j.key1())
    }

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

#[derive(Clone)]
pub struct SideMap {
    hint: Matrix<Cell>,
    inner: SideMapInner
}

impl SideMap {
    pub fn new(hint: Matrix<Cell>) -> SideMap {
        let num_cell = (hint.size().0 * hint.size().1 + 1) as usize;
        SideMap {
            hint: hint,
            inner: SideMapInner::new(num_cell)
        }
    }

    pub fn from_hint(hint: &Hint) -> SideMap {
        let mut map = SideMap::new(hint.cell().clone());
        for r in (0 .. hint.row()) {
            for c in (0 .. hint.column()) {
                let p = Point(r, c);
                if let Some(side) = hint.side()[p] {
                    match side {
                        hint::Side::In => map.set_inside(p),
                        hint::Side::Out => map.set_outside(p)
                    };
                }
                if let Some(edge) = hint.edge_h()[p] {
                    match edge {
                        Edge::Cross => map.set_same(p, p + UP),
                        Edge::Line => map.set_different(p, p + UP)
                    };
                }
                if let Some(edge) = hint.edge_v()[p] {
                    match edge {
                        Edge::Cross => map.set_same(p, p + LEFT),
                        Edge::Line => map.set_different(p, p + LEFT)
                    };
                }
            }
            let p = Point(r, hint.column());
            if let Some(edge) = hint.edge_v()[p] {
                match edge {
                    Edge::Cross => map.set_same(p, p + LEFT),
                    Edge::Line => map.set_different(p, p + LEFT)
                };
            }
        }
        for c in (0 .. hint.column()) {
            let p = Point(hint.row(), c);
            if let Some(edge) = hint.edge_h()[p] {
                match edge {
                    Edge::Cross => map.set_same(p, p + UP),
                    Edge::Line => map.set_different(p, p + UP)
                };
            }
        }
        map
    }

    pub fn to_hint(&mut self) -> Hint {
        let mut hint = Hint::new(self.size());
        for r in (0 .. self.row()) {
            for c in (0 .. self.column()) {
                let p = Point(r, c);

                hint[p] = self.hint[p];
                hint.side_mut()[p] = match self.get_side(p) {
                    Side::In => Some(hint::Side::In),
                    Side::Out => Some(hint::Side::Out),
                    Side::Unknown => None,
                    Side::Conflict => panic!()
                };
                hint.edge_h_mut()[p] = match self.get_relation(p, p + UP) {
                    Relation::Same => Some(Edge::Cross),
                    Relation::Different => Some(Edge::Line),
                    Relation::Unknown => None,
                    Relation::Conflict => panic!()
                };
                hint.edge_v_mut()[p] = match self.get_relation(p, p + LEFT) {
                    Relation::Same => Some(Edge::Cross),
                    Relation::Different => Some(Edge::Line),
                    Relation::Unknown => None,
                    Relation::Conflict => panic!()
                };
            }
            let p = Point(r, hint.column());
            hint.edge_v_mut()[p] = match self.get_relation(p, p + LEFT) {
                Relation::Same => Some(Edge::Cross),
                Relation::Different => Some(Edge::Line),
                Relation::Unknown => None,
                Relation::Conflict => panic!()
            };
        }
        for c in (0 .. hint.column()) {
            let p = Point(hint.row(), c);
            hint.edge_h_mut()[p] = match self.get_relation(p, p + UP) {
                Relation::Same => Some(Edge::Cross),
                Relation::Different => Some(Edge::Line),
                Relation::Unknown => None,
                Relation::Conflict => panic!()
            };
        }
        hint
    }

    fn is_outside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.inner.is_same(i, OUTSIDE_CELL_ID)
    }
    fn is_inside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.inner.is_different(i, OUTSIDE_CELL_ID)
    }
    pub fn is_same(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.inner.is_same(i, j)
    }
    pub fn is_different(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.inner.is_different(i, j)
    }

    pub fn get_side(&mut self, p: Point) -> Side {
        match (self.is_inside(p), self.is_outside(p)) {
            (false, false) => Side::Unknown,
            (true,  false) => Side::In,
            (false, true)  => Side::Out,
            (true,  true)  => Side::Conflict
        }
    }
    pub fn get_relation(&mut self, p0: Point, p1: Point) -> Relation {
        match (self.is_same(p0, p1), self.is_different(p0, p1)) {
            (false, false) => Relation::Unknown,
            (true,  false) => Relation::Same,
            (false, true)  => Relation::Different,
            (true,  true)  => Relation::Conflict
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
            Side::In       => self.set_inside(p),
            Side::Out      => self.set_outside(p),
            Side::Unknown  => panic!(),
            Side::Conflict => panic!()
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
    pub fn set_relation(&mut self, p0: Point, p1: Point, rel: Relation) -> bool {
        match rel {
            Relation::Same      => self.set_same(p0, p1),
            Relation::Different => self.set_different(p0, p1),
            Relation::Unknown   => panic!(),
            Relation::Conflict  => panic!()
        }
    }

    pub fn hint(&self) -> &Matrix<Cell> { &self.hint }
    pub fn revision(&self) -> u32 { self.inner.revision() }

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

#[cfg(test)]
mod tests {
    use super::SideMap;
    use geom::{Size, Point};

    #[test]
    fn set_and_check() {
        let mut side_map = SideMap::new(Size(10, 10));
        let p0 = Point(0, 0);
        let p1 = Point(1, 1);
        let p2 = Point(2, 2);

        side_map.set_outside(p0);
        side_map.set_same(p0, p1);

        assert!(side_map.is_outside(p0));
        assert!(side_map.is_outside(p1));
        assert!(!side_map.is_inside(p0));
        assert!(!side_map.is_inside(p1));
        assert!(side_map.is_same(p0, p1));
        assert!(!side_map.is_different(p0, p1));

        assert!(!side_map.is_same(p1, p2));
        assert!(!side_map.is_different(p1, p2));

        side_map.set_inside(p2);
        assert!(!side_map.is_same(p1, p2));
        assert!(side_map.is_different(p1, p2));
    }
}
