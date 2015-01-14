use std::iter;
use std::ops::{Add, Mul, Index, IndexMut};

#[derive(Clone, Copy, Show, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Point(pub i32, pub i32);
#[derive(Clone, Copy, Show, Eq, PartialEq)]
pub struct Size(pub i32, pub i32);
#[derive(Clone, Copy, Show, Eq, PartialEq)]
pub struct Move(pub i32, pub i32);
#[derive(Clone, Copy, Show, Eq, PartialEq)]
pub struct Rotation(i32, i32, i32, i32);

pub const UP:    Move = Move(-1, 0);
pub const RIGHT: Move = Move(0, 1);
pub const DOWN:  Move = Move(1, 0);
pub const LEFT:  Move = Move(0, -1);

impl Add<Move> for Point {
    type Output = Point;

    fn add(self, other: Move) -> Point {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

impl Add<Move> for Move {
    type Output = Move;

    fn add(self, other: Move) -> Move {
        Move(self.0 + other.0, self.1 + other.1)
    }
}

pub const UCW0:   Rotation = Rotation( 1,  0,  0,  1);
pub const UCW90:  Rotation = Rotation( 0, -1,  1,  0);
pub const UCW180: Rotation = Rotation(-1,  0,  0, -1);
pub const UCW270: Rotation = Rotation( 0,  1, -1,  0);

impl Mul<Rotation> for Rotation {
    type Output = Rotation;

    fn mul(self, other: Rotation) -> Rotation {
        Rotation(self.0 * other.0 + self.1 * other.2,
                 self.0 * other.1 + self.1 * other.3,
                 self.2 * other.0 + self.3 * other.2,
                 self.2 * other.1 + self.3 * other.3)
    }
}

impl Mul<Move> for Rotation {
    type Output = Move;

    fn mul(self, other: Move) -> Move {
        Move(self.0 * other.0 + self.1 * other.1,
             self.2 * other.0 + self.3 * other.1)
    }
}

pub trait Geom {
    fn size(&self) -> Size;
    fn row(&self) -> i32 { self.size().0 }
    fn column(&self) -> i32 { self.size().1 }

    fn contains(&self, p: Point) -> bool {
        let size = self.size();
        0 <= p.0 && p.0 < size.0 &&
            0 <= p.1 && p.1 < size.1
    }

    fn point_to_index(&self, p: Point) -> usize {
        (p.0 * self.column() + p.1) as usize
    }
    fn index_to_point(&self, idx: usize) -> Point {
        Point((idx as i32) / self.column(), (idx as i32) % self.column())
    }
}

impl Geom for Size {
    fn size(&self) -> Size { *self }
}

#[derive(Clone, Show, Eq, PartialEq)]
pub struct Matrix<T> {
    size: Size,
    outside: T,
    data: Vec<T>
}

impl<T> Matrix<T> {
    pub fn new(size: Size, outside: T, data: Vec<T>) -> Matrix<T> {
        assert_eq!((size.0 * size.1) as usize, data.len());
        Matrix {
            size: size, outside: outside, data: data
        }
    }

    pub fn new_empty(size: Size, outside: T, init: T) -> Matrix<T>
        where T: Clone
    {
        let data = iter::repeat(init).take((size.0 * size.1) as usize).collect();
        Matrix::new(size, outside, data)
    }
}

impl<T> Geom for Matrix<T> {
    fn size(&self) -> Size { self.size }
}

impl<T> Index<Point> for Matrix<T> {
    type Output = T;

    fn index(&self, p: &Point) -> &T {
        if self.contains(*p) {
            &self.data[self.point_to_index(*p)]
        } else {
            &self.outside
        }
    }
}

impl<T> IndexMut<Point> for Matrix<T> {
    type Output = T;

    fn index_mut(&mut self, p: &Point) -> &mut T {
        assert!(self.contains(*p));
        let idx = self.point_to_index(*p);
        &mut self.data[idx]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_mat() {
        let mat = [UCW0, UCW90, UCW180, UCW270];
        for i in (0 .. mat.len()) {
            for j in (0 .. mat.len()) {
                assert_eq!(mat[(i + j) % mat.len()], mat[i] * mat[j]);
            }
        }
    }

    #[test]
    fn rotate_point() {
        let mat = [UCW0, UCW90, UCW180, UCW270];
        let vec = [[UP, LEFT, DOWN, RIGHT],
                   [UP + RIGHT, LEFT + UP, DOWN + LEFT, RIGHT + DOWN]];
        for i in (0 .. mat.len()) {
            for v in vec.iter() {
                for j in (0 .. v.len()) {
                    assert_eq!(v[(i + j) % v.len()], mat[i] * v[j]);
                }
            }
        }
    }
}
