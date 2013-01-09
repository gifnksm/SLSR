use core::either::{Either, Left, Right};

use myclone::{MyClone};

pub trait UFValue {
    static pub pure fn init(key: uint) -> self;
    static pub pure fn union(x: self, y: self) -> Either<self, self>;
}

impl uint : UFValue {
    static pub pure fn init(_key: uint) -> uint { 1 }
    static pub pure fn union(x: uint, y: uint) -> Either<uint, uint> {
        if x > y {
            Left(x + y)
        } else {
            Right(x + y)
        }
    }
}

priv enum UFNode<V> {
    UFKey(uint), UFNValue(V)
}

impl<V: MyClone> UFNode<V> : MyClone {
    pure fn clone(&const self) -> UFNode<V> {
        match self {
            &UFKey(k)        => UFKey(k),
            &UFNValue(ref v) => UFNValue(v.clone())
        }
    }
}

impl<V: Copy> UFNode<V> {
    pub pure fn get_key(&self) -> uint { match self { &UFKey(k) => k, _ => fail } }
    pub pure fn get_value(&self) -> V { match self { &UFNValue(v) => v, _ => fail } }
}

pub struct UnionFind<V> {
    priv data: ~[UFNode<V>]
}

impl<V: MyClone> UnionFind<V> : MyClone {
    pure fn clone(&const self) -> UnionFind<V> {
        UnionFind { data: self.data.map(|node| node.clone()) }
    }
}

pub impl<V> UnionFind<V> {
    pub pure fn size(&self) -> uint { self.data.len() }
}

pub impl<V: UFValue> UnionFind<V> {
    static pub pure fn new(n: uint) -> UnionFind<V> {
        UnionFind { data: vec::from_fn(n, |k| UFNValue(UFValue::init(k)))}
    }
}

pub impl<V: UFValue Copy> UnionFind<V> {
    pub fn union(&mut self, x: uint, y: uint) -> bool {
        let x = self.find(x);
        let y = self.find(y);
        if x == y { return false; }

        let x_value = self.data[x].get_value();
        let y_value = self.data[y].get_value();
        match UFValue::union(x_value, y_value) {
            Left(new_x) => {
                self.data[x] = UFNValue(new_x);
                self.data[y] = UFKey(x);
            }
            Right(new_y) => {
                self.data[y] = UFNValue(new_y);
                self.data[x] = UFKey(y);
            }
        }
        return true;
    }

    pub fn find(&mut self, x: uint) -> uint {
        match copy self.data[x] {
            UFNValue(_) => { return x; }
            UFKey(idx) => {
                let idx = self.find(idx);
                self.data[x] = UFKey(idx);
                return idx;
            }
        }
    }

    pub fn get_value(&mut self, x: uint) -> V {
        let key = self.find(x);
        return self.data[key].get_value();
    }

    pub fn set_value(&mut self, x: uint, value: V) {
        let key = self.find(x);
        self.data[key] = UFNValue(value);
    }
}

#[test]
fn test_union_find() {
    let mut uf = UnionFind::new::<uint>(10);
    assert uf.find(0) == uf.find(0);
    assert uf.find(0) != uf.find(1);
    assert uf.find(0) != uf.find(2);
    uf.union(0, 1);
    assert uf.find(0) == uf.find(0);
    assert uf.find(0) == uf.find(1);
    assert uf.find(0) != uf.find(2);
    uf.union(0, 2);
    uf.union(3, 4);
    assert uf.find(0) != uf.find(3);
    uf.union(0, 3);
    assert uf.find(0) == uf.find(3);
}

#[test]
#[should_fail]
fn test_find_overflow() {
    let mut uf = UnionFind::new::<uint>(10);
    uf.find(10);
}

#[test]
#[should_fail]
fn test_union_overflow() {
    let mut uf = UnionFind::new::<uint>(10);
    uf.union(0, 10);
}
