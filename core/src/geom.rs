use std::ops::{Add, Mul, Sub, Neg, Index, IndexMut};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Point(pub i32, pub i32);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Size(pub i32, pub i32);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move(pub i32, pub i32);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rotation(i32, i32, i32, i32);

impl Move {
    pub const UP:    Move = Move(-1, 0);
    pub const RIGHT: Move = Move(0, 1);
    pub const DOWN:  Move = Move(1, 0);
    pub const LEFT:  Move = Move(0, -1);

    pub const ALL_DIRECTIONS: [Move; 4] = [
        Move::UP, Move::RIGHT, Move::DOWN, Move::LEFT
    ];
}


impl Add<Move> for Point {
    type Output = Point;

    #[inline]
    fn add(self, other: Move) -> Point {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub<Point> for Point {
    type Output = Move;

    #[inline]
    fn sub(self, other: Point) -> Move {
        Move(self.0 - other.0, self.1 - other.1)
    }
}

impl Add<Move> for Move {
    type Output = Move;

    #[inline]
    fn add(self, other: Move) -> Move {
        Move(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub<Move> for Move {
    type Output = Move;

    #[inline]
    fn sub(self, other: Move) -> Move {
        Move(self.0 - other.0, self.1 - other.1)
    }
}

impl Neg for Move {
    type Output = Move;

    #[inline]
    fn neg(self) -> Move {
        Move(-self.0, -self.1)
    }
}

impl Mul<i32> for Move {
    type Output = Move;

    #[inline]
    fn mul(self, other: i32) -> Move {
        Move(self.0 * other, self.1 * other)
    }
}

impl Rotation {
    pub const UCW0:   Rotation = Rotation( 1,  0,  0,  1);
    pub const UCW90:  Rotation = Rotation( 0, -1,  1,  0);
    pub const UCW180: Rotation = Rotation(-1,  0,  0, -1);
    pub const UCW270: Rotation = Rotation( 0,  1, -1,  0);
    pub const H_FLIP: Rotation = Rotation( 1,  0,  0, -1);
    pub const V_FLIP: Rotation = Rotation(-1,  0,  0,  1);
}

impl Mul<Rotation> for Rotation {
    type Output = Rotation;

    #[inline]
    fn mul(self, other: Rotation) -> Rotation {
        Rotation(self.0 * other.0 + self.1 * other.2,
                 self.0 * other.1 + self.1 * other.3,
                 self.2 * other.0 + self.3 * other.2,
                 self.2 * other.1 + self.3 * other.3)
    }
}

impl Mul<Move> for Rotation {
    type Output = Move;

    #[inline]
    fn mul(self, other: Move) -> Move {
        Move(self.0 * other.0 + self.1 * other.1,
             self.2 * other.0 + self.3 * other.1)
    }
}

pub trait Geom {
    #[inline]
    fn size(&self) -> Size;
    #[inline]
    fn row(&self) -> i32 { self.size().0 }
    #[inline]
    fn column(&self) -> i32 { self.size().1 }

    #[inline]
    fn contains(&self, p: Point) -> bool {
        let size = self.size();
        0 <= p.0 && p.0 < size.0 &&
            0 <= p.1 && p.1 < size.1
    }

    #[inline]
    fn point_to_index(&self, p: Point) -> usize {
        (p.0 * self.column() + p.1) as usize
    }
    #[inline]
    fn index_to_point(&self, idx: usize) -> Point {
        Point((idx as i32) / self.column(), (idx as i32) % self.column())
    }
}

impl Geom for Size {
    #[inline]
    fn size(&self) -> Size { *self }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matrix<T> {
    size: Size,
    outside: T,
    data: Vec<T>
}

impl<T> Matrix<T> {
    #[inline]
    pub fn new(size: Size, outside: T, data: Vec<T>) -> Matrix<T> {
        assert_eq!((size.0 * size.1) as usize, data.len());
        Matrix {
            size: size, outside: outside, data: data
        }
    }

    #[inline]
    pub fn new_empty(size: Size, outside: T, init: T) -> Matrix<T>
        where T: Clone
    {
        let data = vec![init; (size.0 * size.1) as usize];
        Matrix::new(size, outside, data)
    }
}

impl<T> Geom for Matrix<T> {
    #[inline]
    fn size(&self) -> Size { self.size }
}

impl<T> Index<Point> for Matrix<T> {
    type Output = T;

    #[inline]
    fn index(&self, p: Point) -> &T {
        unsafe {
            if self.contains(p) {
                let idx = self.point_to_index(p);
                self.data.get_unchecked(idx)
            } else {
                &self.outside
            }
        }
    }
}

impl<T> IndexMut<Point> for Matrix<T> {
    #[inline]
    fn index_mut(&mut self, p: Point) -> &mut T {
        unsafe {
            assert!(self.contains(p));
            let idx = self.point_to_index(p);
            self.data.get_unchecked_mut(idx)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_mat() {
        let mat = [Rotation::UCW0, Rotation::UCW90, Rotation::UCW180, Rotation::UCW270];
        for i in (0 .. mat.len()) {
            for j in (0 .. mat.len()) {
                assert_eq!(mat[(i + j) % mat.len()], mat[i] * mat[j]);
            }
        }
    }

    #[test]
    fn rotate_point() {
        let mat = [Rotation::UCW0, Rotation::UCW90, Rotation::UCW180, Rotation::UCW270];
        let vec = [[Move::UP, Move::LEFT, Move::DOWN, Move::RIGHT],
                   [Move::UP + Move::RIGHT, Move::LEFT + Move::UP, Move::DOWN + Move::LEFT, Move::RIGHT + Move::DOWN]];
        for i in 0 .. mat.len() {
            for v in &vec {
                for j in (0 .. v.len()) {
                    assert_eq!(v[(i + j) % v.len()], mat[i] * v[j]);
                }
            }
        }
    }
}
