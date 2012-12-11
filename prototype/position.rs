// Matrix
// | a b |
// | c d |
pub struct Rotation {
    a: int,
    b: int,
    c: int,
    d: int
}

pub impl Rotation {
    // Matrix * column vector
    pub pure fn mul(v: (int, int)) -> (int, int) {
        let (x, y) = v;
        return (self.a * x + self.b * y, self.c * x + self.d * y);
    }

    pub pure fn mul_mat(m: Rotation) -> Rotation {
        return Rotation {
            a: self.a * m.a + self.b * m.c,
            b: self.a * m.b + self.b * m.d,
            c: self.c * m.a + self.d * m.c,
            d: self.c * m.b + self.d * m.d
        }
    }
}

pub const Ucw0Deg:   Rotation = Rotation { a:  1, b:  0, c:  0, d:  1 };
pub const Ucw90Deg:  Rotation = Rotation { a:  0, b:  1, c: -1, d:  0 };
pub const Ucw180Deg: Rotation = Rotation { a: -1, b:  0, c:  0, d: -1 };
pub const Ucw270Deg: Rotation = Rotation { a:  0, b: -1, c:  1, d:  0 };

pub const Identity: Rotation = Ucw0Deg;
pub const Cw0Deg:   Rotation = Ucw0Deg;
pub const Cw90Deg:  Rotation = Ucw270Deg;
pub const Cw180Deg: Rotation = Ucw180Deg;
pub const Cw270Deg: Rotation = Ucw90Deg;

pub fn each_rot4(f: fn(Rotation) -> bool) {
    if !f(Ucw0Deg) { return; }
    if !f(Ucw90Deg) { return; }
    if !f(Ucw180Deg) { return; }
    if !f(Ucw270Deg) { return; }
}

pub const UP:         (int, int) = ( 0, -1);
pub const UP_RIGHT:   (int, int) = ( 1, -1);
pub const RIGHT:      (int, int) = ( 1,  0);
pub const DOWN_RIGHT: (int, int) = ( 1,  1);
pub const DOWN:       (int, int) = ( 0,  1);
pub const DOWN_LEFT:  (int, int) = (-1,  1);
pub const LEFT:       (int, int) = (-1,  0);
pub const UP_LEFT:    (int, int) = (-1, -1);

// column vector
// | a |
// | b |
pub struct Position {
    x: int,
    y: int
}

impl Position : cmp::Ord {
    pub pure fn lt(&self, other: &Position) -> bool {
        if self.x < other.x { return true; }
        if self.x > other.x { return false; }
        return self.y < other.y;
    }
    pub pure fn le(&self, other: &Position) -> bool {
        if self.x < other.x { return true; }
        if self.x > other.x { return false; }
        return self.y <= other.y;
    }
    pub pure fn ge(&self, other: &Position) -> bool {
        if self.x > other.x { return true; }
        if self.x < other.x { return false; }
        return self.y >= other.y;
    }
    pub pure fn gt(&self, other: &Position) -> bool {
        if self.x > other.x { return true; }
        if self.x < other.x { return false; }
        return self.y > other.y;
    }
}

impl Position : cmp::Eq {
    pub pure fn eq(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }
    pub pure fn ne(&self, other: &Position) -> bool {
        !(self == other)
    }
}

pub impl Position {
    pub pure fn shift(d: (int, int)) -> Position {
        let (dx, dy) = d;
        return Position { x: self.x + dx, y: self.y + dy };
    }

    pub pure fn shift_on(d: (int, int), rot: Rotation) -> Position {
        let (dx, dy) = rot.mul(d);
        return Position { x: self.x + dx, y: self.y + dy };
    }

    pub pure fn up()         -> Position { self.shift(UP) }
    pub pure fn up_right()   -> Position { self.shift(UP_RIGHT) }
    pub pure fn right()      -> Position { self.shift(RIGHT) }
    pub pure fn down_right() -> Position { self.shift(DOWN_RIGHT) }
    pub pure fn down()       -> Position { self.shift(DOWN) }
    pub pure fn down_left()  -> Position { self.shift(DOWN_LEFT) }
    pub pure fn left()       -> Position { self.shift(LEFT) }
    pub pure fn up_left()    -> Position { self.shift(UP_LEFT) }

    pub pure fn up_on(r: Rotation) -> Position {
        self.shift(r.mul(UP))
    }
    pub pure fn up_right_on(r: Rotation) -> Position {
        self.shift(r.mul(UP_RIGHT))
    }
    pub pure fn right_on(r: Rotation) -> Position {
        self.shift(r.mul(RIGHT))
    }
    pub pure fn down_right_on(r: Rotation) -> Position {
        self.shift(r.mul(DOWN_RIGHT))
    }
    pub pure fn down_on(r: Rotation) -> Position {
        self.shift(r.mul(DOWN))
    }
    pub pure fn down_left_on(r: Rotation) -> Position {
        self.shift(r.mul(DOWN_LEFT))
    }
    pub pure fn left_on(r: Rotation) -> Position {
        self.shift(r.mul(LEFT))
    }
    pub pure fn up_left_on(r: Rotation) -> Position {
        self.shift(r.mul(UP_LEFT))
    }

    pub pure fn each_around4(f: fn(Position) -> bool) {
        if !f(self.up())    { return; }
        if !f(self.right()) { return; }
        if !f(self.down())  { return; }
        if !f(self.left())  { return; }
    }

    static pub pure fn new(pos: (int, int)) -> Position {
        let (x, y) = pos;
        return Position { x: x, y: y };
    }
}

#[test]
fn test_rotation() {
    let vs = [UP, UP_LEFT, LEFT, DOWN_LEFT,
              DOWN, DOWN_RIGHT, RIGHT, UP_RIGHT];
    for vs.eachi |i, v| {
        assert Ucw0Deg.mul(*v) == vs[i];
        assert Ucw90Deg.mul(*v) == vs[(i + 2) % vs.len()];
        assert Ucw180Deg.mul(*v) == vs[(i + 4) % vs.len()];
        assert Ucw270Deg.mul(*v) == vs[(i + 6) % vs.len()];
    }
}

