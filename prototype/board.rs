use core::io::{Reader, ReaderUtil};

use union_find::{UnionFind};
use position::{Rotation, Position};

use myclone::{MyClone};

type CellId = uint;
const FIXED_CELL_ID: CellId = 0;
type Key = uint;
enum KeyType {
    PosKey, NegKey
}
pub type Hint = Option<uint>;

#[deriving_eq]
pub enum CellType {
    Inside, Outside, UnknownType, ConflictType
}
#[deriving_eq]
pub enum CellRelation {
    Same, Different, UnknownRel, ConflictRel
}

pub struct Board {
    priv width: uint,
    priv height: uint,
    priv uf: UnionFind<uint>,
    priv hint: ~[~[Hint]],
    priv seq: uint,
    priv sum_of_hint: uint
}

pub impl uint : MyClone {
    pure fn clone(&const self) -> uint { *self }
}

pub impl Board : MyClone {
    pure fn clone(&const self) -> Board {
        Board {
            width: self.width, height: self.height,
            uf: self.uf.clone(),
            hint: self.hint.map(|row| row.slice(0, row.len())),
            seq: self.seq,
            sum_of_hint: self.sum_of_hint
        }
    }
}

pub impl Board {
    static pub pure fn new(hint: ~[~[Hint]]) -> ~Board {
        let height = hint.len();
        assert height > 0;

        let width = hint[0].len();
        assert width > 0 && hint.all(|row| row.len() == width);

        assert hint.all(|row| row.all(|c| c.get_default(0) < 4));

        let uf_size = (width + 2) * (height + 2) * 2;
        let sum_of_hint = do hint.foldl(0) |&s, row| {
            do row.foldl(s) |&s, cell| { s + cell.get_default(0) }
        };
        return ~Board {
            width: width, height: height,
            uf: UnionFind::new(uf_size),
            hint: move hint,
            seq: 0,
            sum_of_hint: sum_of_hint
        };
    }

    static pub pure fn from_size(width: uint, height: uint) -> ~Board {
        let hint = vec::from_fn(height, |_i| vec::from_elem(width, None));
        return Board::new(move hint);
    }

    static pub fn from_stream(stream: Reader) -> ~Board {
        let mut lines = ~[];
        let mut width = 0;
        while !stream.eof() {
            let line = stream.read_line();
            if line.len() == 0 { loop; }
            width = uint::max(line.len(), width);
            lines.push(move line);
        }

        let hint = vec::from_fn(lines.len(), |y| {
            let line = &lines[y];
            vec::from_fn(width, |x| {
                if x < line.len() {
                    match line[x] {
                        '0' as u8 => Some(0),
                        '1' as u8 => Some(1),
                        '2' as u8 => Some(2),
                        '3' as u8 => Some(3),
                        _   => None
                    }
                } else {
                    None
                }
            })
        });
        return Board::new(move hint);
    }
}

pub impl Board {
    pub pure fn get_width(&const self) -> uint { self.width }
    pub pure fn get_height(&const self) -> uint { self.height }
    pub pure fn get_seq(&const self) -> uint { self.seq }
    pub pure fn get_sum_of_hint(&const self) -> uint { self.sum_of_hint }

    pub pure fn contains(&const self, p: Position) -> bool {
        return (0 <= p.x && (p.x as uint) < self.width) &&
            (0 <= p.y &&  (p.y as uint) < self.height);
    }

    pub pure fn get_hint(&const self, p: Position) -> Hint {
        if !self.contains(p) { return None; }
        return self.hint[p.y][p.x];
    }

    pub fn each_x(&const self, f: fn(int) -> bool) {
        for int::range(0, self.width as int) |x| {
            if !f(x) { break; }
        }
    }

    pub fn each_y(&const self, f: fn(int) -> bool) {
        for int::range(0, self.height as int) |y| {
            if !f(y) { break; }
        }
    }

    pub fn each_pos(&const self, f: fn(Position) -> bool) {
        for self.each_y |y| {
            for self.each_x |x| {
                if !f(Position::new((x, y))) { break; }
            }
        }
    }

    priv fn to_cellid(&const self, p: Position) -> CellId {
        if !self.contains(p) { return FIXED_CELL_ID; }
        return 1 + (p.x as uint) + self.width * (p.y as uint);
    }

    priv fn to_key(&const self, id: CellId, key_type: KeyType) -> Key {
        return id * 2 + match key_type { PosKey => 0, NegKey => 1 };
    }

    pub fn get_cell_type(&mut self, p: Position) -> CellType {
        match (self.is_inside(p), self.is_outside(p)) {
            (false, false) => UnknownType,
            (true,  false) => Inside,
            (false, true)  => Outside,
            (true,  true)  => ConflictType
        }
    }
    pub fn get_cell_relation(&mut self, p1: Position, p2: Position) -> CellRelation {
        match (self.is_same(p1, p2), self.is_different(p1, p2)) {
            (false, false) => UnknownRel,
            (true,  false) => Same,
            (false, true)  => Different,
            (true,  true)  => ConflictRel
        }
    }
    pub fn get_group(&mut self, p: Position) -> (uint, uint) {
        (self.uf.find(self.to_key(self.to_cellid(p), PosKey)),
         self.uf.find(self.to_key(self.to_cellid(p), NegKey)))
    }
    pub fn get_fixed_group(&mut self) -> (uint, uint) {
        (self.uf.find(self.to_key(FIXED_CELL_ID, NegKey)),
         self.uf.find(self.to_key(FIXED_CELL_ID, NegKey)))
    }

    pub fn is_inside(&mut self, p: Position) -> bool {
        return self.is_different_id(self.to_cellid(p), FIXED_CELL_ID);
    }
    pub fn is_outside(&mut self, p: Position) -> bool {
        self.is_same_id(self.to_cellid(p), FIXED_CELL_ID)
    }
    pub fn is_same(&mut self, p1: Position, p2: Position) -> bool {
        self.is_same_id(self.to_cellid(p1), self.to_cellid(p2))
    }
    pub fn is_different(&mut self, p1: Position, p2: Position) -> bool {
        self.is_different_id(self.to_cellid(p1), self.to_cellid(p2))
    }

    pub fn is_same_all(&mut self, ps: &[Position]) -> bool {
        if ps.is_empty() { return true; }
        let base = ps[0];
        return vec::view(ps, 1, ps.len()).all(|p| self.is_same(base, *p));
    }
    pub fn is_same_around(&mut self, base: Position, ds: &[(int, int)]) -> bool {
        ds.all(|d| self.is_same(base, base.shift(*d)))
    }
    pub fn is_same_around_on(&mut self, base: Position,
                             ds: &[(int, int)], rot: Rotation) -> bool {
        ds.all(|d| self.is_same(base, base.shift_on(*d, rot)))
    }

    pub fn is_different_around(&mut self, base: Position, ds: &[(int, int)]) -> bool {
        ds.all(|d| self.is_different(base, base.shift(*d)))
    }
    pub fn is_different_around_on(&mut self, base: Position,
                                  ds: &[(int, int)], rot: Rotation) -> bool {
        ds.all(|d| self.is_different(base, base.shift_on(*d, rot)))
    }


    pub fn set_inside(&mut self, p: Position) {
        assert self.contains(p);
        self.set_different_id(self.to_cellid(p), FIXED_CELL_ID)
    }
    pub fn set_outside(&mut self, p: Position) {
        assert self.contains(p);
        self.set_same_id(self.to_cellid(p), FIXED_CELL_ID)
    }
    pub fn set_same(&mut self, p1: Position, p2: Position) {
        self.set_same_id(self.to_cellid(p1), self.to_cellid(p2))
    }
    pub fn set_different(&mut self, p1: Position, p2: Position) {
        self.set_different_id(self.to_cellid(p1), self.to_cellid(p2))
    }
    pub fn set_hint(&mut self, p: Position, hint: Hint) {
        assert self.contains(p);
        let old = self.hint[p.y][p.x];
        self.hint[p.y][p.x] = hint;
        match hint {
            Some(x) => self.sum_of_hint += x,
            None => {}
        }
        match old {
            Some(x) => self.sum_of_hint -= x,
            None => {}
        }
    }

    pub fn set_same_around(&mut self, base: Position, ds: &[(int, int)]) {
        for ds.each |d| { self.set_same(base, base.shift(*d)); }
    }
    pub fn set_same_around_on(&mut self, base: Position,
                              ds: &[(int, int)], rot: Rotation) {
        for ds.each |d| { self.set_same(base, base.shift_on(*d, rot)); }
    }

    pub fn set_different_around(&mut self, base: Position, ds: &[(int, int)]) {
        for ds.each |d| { self.set_different(base, base.shift(*d)) }
    }
    pub fn set_different_around_on(&mut self, base: Position,
                                   ds: &[(int, int)], rot: Rotation) {
        for ds.each |d| { self.set_different(base, base.shift_on(*d, rot)); }
    }

    pub fn set_same_all(&mut self, ps: &[Position]) {
        if ps.is_empty() { return; }
        let base = ps[0];
        for vec::view(ps, 1, ps.len()).each |p| { self.set_same(base, *p) }
    }

    priv fn set_same_id(&mut self, id1: CellId, id2: CellId) {
        let c1 =
            self.uf.union(self.to_key(id1, PosKey), self.to_key(id2, PosKey));
        let c2 =
            self.uf.union(self.to_key(id1, NegKey), self.to_key(id2, NegKey));
        if c1 || c2 { self.seq += 1; }
    }
    priv fn set_different_id(&mut self, id1: CellId, id2: CellId) {
        let c1 =
            self.uf.union(self.to_key(id1, PosKey), self.to_key(id2, NegKey));
        let c2 =
            self.uf.union(self.to_key(id1, NegKey), self.to_key(id2, PosKey));
        if c1 || c2 { self.seq += 1; }
    }
    priv fn is_same_id(&mut self, id1: CellId, id2: CellId) -> bool {
        self.uf.find(self.to_key(id1, PosKey)) ==
            self.uf.find(self.to_key(id2, PosKey))
    }
    priv fn is_different_id(&mut self, id1: CellId, id2: CellId) -> bool {
        self.uf.find(self.to_key(id1, PosKey)) ==
            self.uf.find(self.to_key(id2, NegKey))
    }
}

#[test]
fn test_board() {
    let mut board = Board::from_size(10, 10);
    let p0 = Position::new((0, 0));
    let p1 = Position::new((1, 1));
    let p2 = Position::new((2, 2));
    board.set_outside(p0);
    board.set_same(p0, p1);
    assert board.is_outside(p0);
    assert board.is_outside(p1);
    assert board.is_same(p0, p1);
    assert !board.is_different(p0, p1);
    assert !board.is_same(p1, p2);
    assert !board.is_different(p1, p2);
    board.set_inside(p2);
    assert !board.is_same(p1, p2);
    assert board.is_different(p1, p2);
}
