use union_find::{UnionFind};
use point::{Point};

type Hint = Option<uint>;

pub struct Board {
    width: uint,
    height: uint,
    sum_of_hint: uint,
    hint: ~[Hint],
    side_map: UnionFind<()>,
    conn_map: UnionFind<()>
}

pub impl Board {
    static pub pure fn new(width: uint, height: uint, hint: ~[Hint]) -> ~Board {
        let map_len = width * height + 1;

        assert hint.len() == width * height;
        assert hint.all(|c| c.get_default(0) < 4);

        let side_map = UnionFind::from_elem(map_len * 2, ());
        let conn_map = UnionFind::from_elem(map_len, ());

        return ~Board {
            width: width,
            height: height,
            sum_of_hint: 0,
            hint: hint,
            side_map: side_map,
            conn_map: conn_map
        };
    }

    pub pure fn get_width(&self) -> uint { self.width }
    pub pure fn get_height(&self) -> uint { self.height }
    pub pure fn get_sum_of_hint(&self) -> uint { self.sum_of_hint }

    pub fn is_inside(&mut self, p: Point) -> bool {
        self.is_different_side_key(self.point_to_key(p), self.get_outside_key())
    }
    pub fn is_outside(&mut self, p: Point) -> bool {
        self.is_same_side_key(self.point_to_key(p), self.get_outside_key())
    }
    pub fn is_same_side(&mut self, p1: Point, p2: Point) -> bool {
        self.is_same_side_key(self.point_to_key(p1), self.point_to_key(p2))
    }
    pub fn is_different_side(&mut self, p1: Point, p2: Point) -> bool {
        self.is_different_side_key(self.point_to_key(p1), self.point_to_key(p2))
    }

    pub pure fn contains(&self, pt: Point) -> bool {
        return 0 <= pt.x && pt.x as uint < self.width &&
            0 <= pt.y && pt.y as uint < self.height;
    }

    priv pure fn get_outside_key(&self) -> uint {
        self.width * self.height
    }

    priv pure fn point_to_key(&self, pt: Point) -> uint {
        if self.contains(pt) {
            return (pt.y as uint) * self.height + (pt.x as uint);
        } return {
            return self.get_outside_key();
        }
    }

    priv pure fn to_pos_side_key(&self, key: uint) -> uint { key * 2 }
    priv pure fn to_neg_side_key(&self, key: uint) -> uint { key * 2 + 1 }

    priv fn is_same_side_key(&mut self, key1: uint, key2: uint) -> bool {
        self.side_map.find(self.to_pos_side_key(key1), self.to_pos_side_key(key2))
    }
    priv fn is_different_side_key(&mut self, key1: uint, key2: uint) -> bool {
        self.side_map.find(self.to_pos_side_key(key1), self.to_neg_side_key(key2))
    }
}
