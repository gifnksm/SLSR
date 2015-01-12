use union_find::UnionFind;
use geom::{Geom, Point, Size};

const OUTSIDE_CELL_ID: CellId = CellId(0);

#[derive(Copy, Clone, Eq, PartialEq, Show)]
struct CellId(usize);

impl CellId {
    fn key0(self) -> usize { self.0 * 2 }
    fn key1(self) -> usize { self.0 * 2 + 1 }
}

#[derive(Copy, Clone, Eq, PartialEq, Show)]
pub enum CellRelation {
    Same, Different, Unknown, Conflict
}

#[derive(Copy, Clone, Eq, PartialEq, Show)]
pub enum CellType {
    Inside, Outside, Unknown, Conflict
}


#[derive(Clone)]
struct SideMap {
    uf: UnionFind,
    revision: u32
}

impl SideMap {
    fn new(size: usize) -> SideMap {
        SideMap { uf: UnionFind::new(size * 2), revision: 0 }
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
pub struct Board {
    size: Size,
    side_map: SideMap
}

impl Board {
    pub fn new(size: Size) -> Board {
        assert!(size.0 > 0 && size.1 > 0);
        let num_cell = (size.0 * size.1 + 1) as usize;
        Board {
            size: size,
            side_map: SideMap::new(num_cell)
        }
    }

    fn is_outside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.side_map.is_same(i, OUTSIDE_CELL_ID)
    }
    fn is_inside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.side_map.is_different(i, OUTSIDE_CELL_ID)
    }
    pub fn is_same(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.side_map.is_same(i, j)
    }
    pub fn is_same_all(&mut self, ps: &[Point]) -> bool {
        match ps {
            [] => true,
            [p0, ps..] => ps.iter().all(|&p| self.is_same(p0, p))
        }
    }
    pub fn is_different(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.side_map.is_different(i, j)
    }

    pub fn set_outside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.side_map.set_same(i, OUTSIDE_CELL_ID)
    }
    pub fn set_inside(&mut self, p: Point) -> bool {
        let i = self.cell_id(p);
        self.side_map.set_different(i, OUTSIDE_CELL_ID)
    }
    pub fn set_type(&mut self, p: Point, ty: CellType) -> bool {
        match ty {
            CellType::Inside  => self.set_inside(p),
            CellType::Outside => self.set_outside(p),
            CellType::Unknown => panic!(),
            CellType::Conflict => panic!()
        }
    }
    pub fn set_same(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.side_map.set_same(i, j)
    }
    pub fn set_different(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.side_map.set_different(i, j)
    }
    pub fn set_relation(&mut self, p0: Point, p1: Point, rel: CellRelation) -> bool {
        match rel {
            CellRelation::Same      => self.set_same(p0, p1),
            CellRelation::Different => self.set_different(p0, p1),
            CellRelation::Unknown   => panic!(),
            CellRelation::Conflict  => panic!()
        }
    }

    pub fn get_relation(&mut self, p0: Point, p1: Point) -> CellRelation {
        match (self.is_same(p0, p1), self.is_different(p0, p1)) {
            (false, false) => CellRelation::Unknown,
            (true,  false) => CellRelation::Same,
            (false, true)  => CellRelation::Different,
            (true,  true)  => CellRelation::Conflict
        }
    }

    pub fn get_type(&mut self, p: Point) -> CellType {
        match (self.is_inside(p), self.is_outside(p)) {
            (false, false) => CellType::Unknown,
            (true,  false) => CellType::Inside,
            (false, true)  => CellType::Outside,
            (true,  true)  => CellType::Conflict
        }
    }

    pub fn revision(&self) -> u32 { self.side_map.revision() }

    fn cell_id(&self, p: Point) -> CellId {
        if self.contains(p) {
            CellId(self.point_to_index(p) + 1)
        } else {
            OUTSIDE_CELL_ID
        }
    }
}

impl Geom for Board {
    fn size(&self) -> Size { self.size }
}

#[cfg(test)]
mod tests {
    use super::Board;
    use geom::{Size, Point};

    #[test]
    fn set_and_check() {
        let mut board = Board::new(Size(10, 10));
        let p0 = Point(0, 0);
        let p1 = Point(1, 1);
        let p2 = Point(2, 2);

        board.set_outside(p0);
        board.set_same(p0, p1);

        assert!(board.is_outside(p0));
        assert!(board.is_outside(p1));
        assert!(!board.is_inside(p0));
        assert!(!board.is_inside(p1));
        assert!(board.is_same(p0, p1));
        assert!(!board.is_different(p0, p1));

        assert!(!board.is_same(p1, p2));
        assert!(!board.is_different(p1, p2));

        board.set_inside(p2);
        assert!(!board.is_same(p1, p2));
        assert!(board.is_different(p1, p2));
    }
}
