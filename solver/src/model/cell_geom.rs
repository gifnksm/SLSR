use slsr_core::geom::{Geom, Point};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CellId(usize);

impl CellId {
    pub fn id(self) -> usize { self.0 }
}

pub const OUTSIDE_CELL_ID: CellId = CellId(0);

pub trait CellGeom: Geom {
    fn cell_id(&self, p: Point) -> CellId {
        if self.contains(p) {
            CellId(self.point_to_index(p) + 1)
        } else {
            OUTSIDE_CELL_ID
        }
    }
}
