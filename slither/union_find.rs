pub type UFKey = uint;
pub type UFValue<T> = { value: T, size: uint };

pub trait Union {
    pub fn union(&self, other: &self) -> self;
}

impl () : Union {
    pub pure fn union(&self, _other: &()) -> () { () }
}

priv enum UFNode<T: Union> {
    UFKey(UFKey),
    UFValue(UFValue<T>)
}

pub struct UnionFind<T: Union> {
    priv data: ~[UFNode<T>]
}

pub mod UnionFind {
    pub pure fn new<T: Union>(data: ~[T]) -> UnionFind<T> {
        return UnionFind { data: data.map(|&v| UFValue({ value: v, size: 1 })) };
    }

    pub pure fn from_elem<T: Copy Union>(size: uint, init: T) -> UnionFind<T> {
        return UnionFind::new(vec::from_elem(size, init));
    }
}

pub impl<T: Union> UnionFind<T> {
    pub pure fn get_total_size(&self) -> uint { self.data.len() }

    pub fn get_size(&mut self, key: UFKey) -> uint {
        let key = self.find_root_key(key);

        // To avoid error: illegal borrow unlwss pure
        match &self.data[key] { &UFValue(v) => return v.size, _ => fail };
    }

    pub fn union(&mut self, key1: UFKey, key2: UFKey) -> bool {
        let mut key1 = self.find_root_key(key1);
        let mut key2 = self.find_root_key(key2);
        if key1 == key2 { return false; }

        // To avoid error: illegal borrow unlwss pure
        let mut value1 = &match self.data[key1] { UFValue(v) => v, _ => fail };
        let mut value2 = &match self.data[key2] { UFValue(v) => v, _ => fail };

        if value1.size < value2.size {
            key1 <-> key2;
            value1 <-> value2;
        }

        self.data[key1] = UFValue({
            value: value1.value.union(&value2.value),
            size: value1.size + value2.size
        });
        self.data[key2] = UFKey(key1);

        return true;
    }

    pub fn find(&mut self, key1: UFKey, key2: UFKey) -> bool {
        let key1 = self.find_root_key(key1);
        let key2 = self.find_root_key(key2);
        return key1 == key2;
    }

    priv fn find_root_key(&mut self, key: UFKey) -> UFKey {
        let root_key = match self.data[key] {
            UFValue(*) => return key,
            UFKey(key) => self.find_root_key(key)
        };
        self.data[key] = UFKey(root_key);
        return root_key;
    }
}

#[test]
fn test_union_find() {
    let mut uf = UnionFind::from_elem(100, ());
    assert uf.get_size(0) == 1;
    assert uf.get_size(1) == 1;
    assert !uf.find(0, 1);
    assert !uf.find(1, 2);
    assert uf.union(0, 1);
    assert uf.find(0, 1);
    assert uf.get_size(0) == 2;
    assert uf.get_size(1) == 2;
    assert uf.get_size(2) == 1;
    assert !uf.union(0, 1);
    assert uf.get_size(0) == 2;
    assert uf.get_size(1) == 2;
    assert uf.get_size(2) == 1;
    assert uf.union(1, 2);
    assert uf.get_size(0) == 3;
    assert uf.get_size(1) == 3;
    assert uf.get_size(2) == 3;
    assert uf.find(0, 1);
    assert uf.find(2, 1);
}