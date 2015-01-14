use std::{cmp, mem};
use std::iter::{self, FromIterator};
use union_find::{UnionFind, UFValue, Merge};
use board::Board;
use geom::{Geom, Point, Size, UP, LEFT, RIGHT, DOWN, UCW0, UCW90, UCW180, UCW270};
use side_map::{SideMap, Relation, Side};

fn fill_by_num_place(side_map: &mut SideMap) {
    // Corner points
    let corners = [(Point(0, 0), UCW0),
                   (Point(side_map.row() - 1, 0), UCW90),
                   (Point(side_map.row() - 1, side_map.column() - 1), UCW180),
                   (Point(0, side_map.column() - 1), UCW270)];
    for &(p, rot) in corners.iter() {
        match side_map.hint()[p] {
            Some(0) => {}
            Some(1) => {
                side_map.set_same(p, p + rot * UP);
                side_map.set_same(p, p + rot * LEFT);
            }
            Some(2) => {
                side_map.set_different(p + rot * RIGHT, p + rot * (RIGHT + UP));
                side_map.set_different(p + rot * DOWN,  p + rot * (DOWN + LEFT));
            }
            Some(3) => {
                side_map.set_different(p, p + rot * UP);
                side_map.set_different(p, p + rot * LEFT);
            }
            _ => {}
        }
    }

    // All points
    for r in (0 .. side_map.row()) {
        for c in (0 .. side_map.column()) {
            let p = Point(r, c);
            match side_map.hint()[p] {
                Some(0) => {
                    for &rot in [UCW0, UCW90, UCW180, UCW270].iter() {
                        let r  = p + rot * RIGHT;
                        let dr = p + rot * (DOWN + RIGHT);
                        side_map.set_same(p, r);

                        //           -
                        // 0 3 =>  0x3|
                        //           -
                        if side_map.hint()[r] == Some(3) {
                            side_map.set_different(r, r + rot * UP);
                            side_map.set_different(r, r + rot * (UP + RIGHT));
                            side_map.set_different(r, r + rot * RIGHT);
                            side_map.set_different(r, r + rot * (DOWN + RIGHT));
                            side_map.set_different(r, r + rot * DOWN);
                        }

                        // 0      0x
                        //     => x -
                        //   3     |3
                        if side_map.hint()[dr] == Some(3) {
                            side_map.set_different(dr, dr + rot * UP);
                            side_map.set_different(dr, dr + rot * LEFT);
                        }
                    }
                }
                Some(1) => {}
                Some(2) => {}
                Some(3) => {
                    //          x
                    // 3 3 => |3|3|
                    //          x
                    for &rot in [UCW0, UCW90].iter() {
                        let r = p + rot * RIGHT;
                        if side_map.hint()[r] == Some(3) {
                            side_map.set_different(p, p + rot * LEFT);
                            side_map.set_different(p, r);
                            side_map.set_different(r, r + rot * RIGHT);

                            side_map.set_same(p + rot * UP,   r + rot * UP);
                            side_map.set_same(p + rot * DOWN, r + rot * DOWN);
                            side_map.set_different(p + rot * UP, r + rot * DOWN);
                        }
                    }

                    //       -           -
                    // 3    |3    3     |3
                    //  3 =>  3|,  2  =>  2
                    //        -     3       3|
                    //                      -
                    for &rot in [UCW0, UCW90].iter() {
                        let mut dr = p + rot * (DOWN + RIGHT);
                        let mut cnt = 1;
                        while side_map.hint()[dr] == Some(2) {
                            dr = dr + rot * (DOWN + RIGHT);
                            cnt += 1;
                        }
                        if side_map.hint()[dr] == Some(3) {
                            side_map.set_different(p, p + rot * UP);
                            side_map.set_different(p, p + rot * (UP + LEFT));
                            side_map.set_different(p, p + rot * LEFT);

                            side_map.set_different(dr, dr + rot * RIGHT);
                            side_map.set_different(dr, dr + rot * (RIGHT + DOWN));
                            side_map.set_different(dr, dr + rot * DOWN);

                            let mut t = p;
                            for _ in (0 .. cnt) {
                                side_map.set_different(t + rot * RIGHT,
                                                    t + rot * DOWN);
                                t = t + rot * (DOWN + RIGHT);
                            }

                            if side_map.hint()[p + rot * (RIGHT + RIGHT)] == Some(3) {
                                side_map.set_same(p + rot * RIGHT, p + rot * (UP + RIGHT));
                            }
                            if side_map.hint()[p + rot * (DOWN + DOWN)] == Some(3) {
                                side_map.set_same(p + rot * DOWN, p + rot * (DOWN + LEFT));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn fill_by_line_nums(side_map: &mut SideMap) {
    for r in (0 .. side_map.row()) {
        for c in (0 .. side_map.column()) {
            let p = Point(r, c);
            let mut sames    = [None; 4];
            let mut diffs    = [None; 4];
            let mut unknowns = [None; 4];
            let mut num_same    = 0;
            let mut num_diff    = 0;
            let mut num_unknown = 0;

            for &dir in [UP, RIGHT, DOWN, LEFT].iter() {
                match side_map.get_relation(p, p + dir) {
                    Relation::Same      => { sames[num_same] = Some(dir); num_same += 1; }
                    Relation::Different => { diffs[num_diff] = Some(dir); num_diff += 1; }
                    Relation::Unknown   => { unknowns[num_unknown] = Some(dir); num_unknown += 1; }
                    _ => panic!() // FIXME
                }
            }

            if num_diff == 3 && num_unknown == 1 {
                side_map.set_same(p, p + unknowns[0].unwrap());
                continue
            }

            match side_map.hint()[p] {
                Some(x) if (num_diff as u8) == x => {
                    for i in (0 .. num_unknown) {
                        side_map.set_same(p, p + unknowns[i].unwrap());
                    }
                }
                Some(x) if (num_same as u8) == 4 - x => {
                    for i in (0 .. num_unknown) {
                        side_map.set_different(p, p + unknowns[i].unwrap());
                    }
                }
                _ => {}
            }
        }
    }
}

fn fill_by_relation(side_map: &mut SideMap) {
    for r in (0 .. side_map.row()) {
        for c in (0 .. side_map.column()) {
            let p = Point(r, c);

            for &rot in [UCW0, UCW90, UCW180, UCW270].iter() {
                let u = p + rot * UP;
                let d = p + rot * DOWN;
                let r = p + rot * RIGHT;
                let l = p + rot * LEFT;
                let ur = p + rot * (UP + RIGHT);
                let ul = p + rot * (UP + LEFT);

                if side_map.is_different(p, u) {
                    if side_map.is_different(p, r) {
                        side_map.set_different(p, ur);
                    }
                    if side_map.is_different(r, ur) {
                        side_map.set_same(p, r);
                    }
                }

                match side_map.get_relation(u, r) {
                    Relation::Same => {
                        match side_map.hint()[p] {
                            Some(1) => {
                                side_map.set_same(p, u);
                                side_map.set_same(p, r);
                                side_map.set_different(l, d);
                            }
                            Some(2) => {
                                side_map.set_same(l, d);
                                side_map.set_different(u, l);
                            }
                            Some(3) => {
                                side_map.set_different(p, u);
                                side_map.set_different(p, r);
                                side_map.set_different(l, d);
                            }
                            _ => {}
                        }
                    }
                    Relation::Different => {
                        match side_map.hint()[p] {
                            Some(1) => {
                                side_map.set_same(p, l);
                                side_map.set_same(p, d);
                            }
                            Some(2) => {
                                side_map.set_different(l, d);
                            }
                            Some(3) => {
                                side_map.set_different(p, l);
                                side_map.set_different(p, d);
                            }
                            _ => {}
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                match side_map.get_relation(u, ur) {
                    Relation::Same => {
                        if side_map.hint()[p] == Some(3) && 
                            side_map.hint()[r] == Some(1) {
                            side_map.set_different(p, u);
                            side_map.set_same(r, r + rot * RIGHT);
                            side_map.set_same(r, r + rot * DOWN);
                        }
                    }
                    Relation::Different => {
                        if side_map.hint()[p] == Some(3) {
                            side_map.set_different(p, d);
                            side_map.set_different(p, l);
                            side_map.set_same(ur, r);
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                match side_map.get_relation(u, ul) {
                    Relation::Same => {
                        if side_map.hint()[p] == Some(3) &&
                            side_map.hint()[l] == Some(1) {
                            side_map.set_different(p, u);
                            side_map.set_same(l, l + rot * LEFT);
                            side_map.set_same(l, l + rot * DOWN);
                        }
                    }
                    Relation::Different => {
                        if side_map.hint()[p] == Some(3) {
                            side_map.set_different(p, d);
                            side_map.set_different(p, r);
                            side_map.set_same(ul, l);
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }
            }

            for &rot in [UCW0, UCW90].iter() {
                let u = p + rot * UP;
                let d = p + rot * DOWN;
                let r = p + rot * RIGHT;
                let l = p + rot * LEFT;
                let dr = p + rot * (DOWN + RIGHT);

                match side_map.get_relation(u, d) {
                    Relation::Same => {
                        match side_map.hint()[p] {
                            Some(1) => {
                                side_map.set_same(p, u);
                                side_map.set_different(l, r);
                            }
                            Some(2) => {
                                side_map.set_different(u, l);
                                side_map.set_same(l, r);
                            }
                            Some(3) => {
                                side_map.set_different(p, u);
                                side_map.set_different(l, r);
                            }
                            _ => {}
                        }
                    }
                    Relation::Different => {
                        match side_map.hint()[p] {
                            Some(1) => {
                                side_map.set_same(p, l);
                                side_map.set_same(p, r);
                            }
                            Some(2) => {
                                side_map.set_different(l, r);
                            }
                            Some(3) => {
                                side_map.set_different(p, l);
                                side_map.set_different(p, r);
                            }
                            _ => {}
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                if (side_map.is_different(p, r) || side_map.is_different(p, d)) &&
                    (side_map.is_different(dr, r) || side_map.is_different(dr, d)) {
                        side_map.set_different(r, d);
                    }
            }
        }
    }
}

#[derive(Show)]
struct Area {
    coord: Point,
    side: Side,
    unknown_rel: Vec<Point>,
    sum_of_hint: u32,
    size: usize
}

impl UFValue for Area {
    fn merge(lval: Area, rval: Area) -> Merge<Area> {
        let coord = if lval.coord < rval.coord {
            lval.coord
        } else {
            rval.coord
        };
        let side = match (lval.side, rval.side) {
            (Side::In, Side::In)       => Side::In,
            (Side::In, Side::Unknown)  => Side::In,
            (Side::In, _)              => Side::Conflict,
            (Side::Out, Side::Out)     => Side::Out,
            (Side::Out, Side::Unknown) => Side::Out,
            (Side::Out, _)             => Side::Conflict,
            (Side::Unknown, x)         => x,
            (Side::Conflict, _)        => Side::Conflict,
        };
        let area = Area {
            coord: coord,
            side: side,
            unknown_rel: lval.unknown_rel + &rval.unknown_rel[],
            sum_of_hint: lval.sum_of_hint + rval.sum_of_hint,
            size: lval.size + rval.size
        };
        if lval.coord < rval.coord {
            Merge::Left(area)
        } else {
            Merge::Right(area)
        }
    }
}

#[derive(Show)]
struct ConnectMap {
    size: Size,
    uf: UnionFind<Area>
}

impl ConnectMap {
    fn new<F>(size: Size, mut f: F) -> ConnectMap
        where F: FnMut(Point) -> Area
    {
        let len = (size.0 * size.1 + 1) as usize;
        ConnectMap {
            size: size,
            uf: FromIterator::from_iter((0 .. len).map(|i| {
                let p = if i == 0 {
                    Point(-1, -1)
                } else {
                    size.index_to_point(i - 1)
                };
                f(p)
            }))
        }
    }

    fn from_side_map(side_map: &mut SideMap) -> ConnectMap {
        let mut conn_map = ConnectMap::new(side_map.size(), |p| {
            let sum = side_map.hint()[p].unwrap_or(0);

            let mut rel = vec![];
            if side_map.contains(p) {
                for &r in [UP, RIGHT, DOWN, LEFT].iter() {
                    if side_map.get_relation(p, p + r) == Relation::Unknown {
                        rel.push(p + r);
                    }
                }
            } else {
                for r in (0 .. side_map.row()) {
                    for &c in [0, side_map.column() - 1].iter() {
                        let p2 = Point(r, c);
                        if side_map.get_relation(p, p2) == Relation::Unknown {
                            rel.push(p2);
                        }
                    }
                }
                for c in (0 .. side_map.column()) {
                    for &r in [0, side_map.row() - 1].iter() {
                        let p2 = Point(r, c);
                        if side_map.get_relation(p, p2) == Relation::Unknown {
                            rel.push(p2);
                        }
                    }
                }
            }
            rel.sort();
            rel.dedup();

            Area {
                coord: p,
                side: side_map.get_side(p),
                unknown_rel: rel,
                sum_of_hint: sum as u32,
                size: 1
            }
        });

        for r in (0 .. side_map.row()) {
            for c in (0 .. side_map.column()) {
                let p = Point(r, c);
                for &r in [UP, RIGHT, DOWN, LEFT].iter() {
                    let p2 = p + r;
                    if side_map.get_relation(p, p2) == Relation::Same {
                        conn_map.union(p, p2);
                    }
                }
            }
        }
        conn_map
    }

    fn union(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.uf.union(i, j)
    }

    fn find(&mut self, p0: Point, p1: Point) -> bool {
        let i = self.cell_id(p0);
        let j = self.cell_id(p1);
        self.uf.find(i, j)
    }

    fn get(&mut self, p: Point) -> &Area {
        let i = self.cell_id(p);
        self.uf.get(i)
    }

    fn get_mut(&mut self, p: Point) -> &mut Area {
        let i = self.cell_id(p);
        self.uf.get_mut(i)
    }

    fn cell_id(&self, p: Point) -> usize {
        if self.contains(p) {
            self.point_to_index(p) + 1
        } else {
            0
        }
    }
}

impl Geom for ConnectMap {
    fn size(&self) -> Size { self.size }
}

fn filter_rel(side_map: &mut SideMap, p: Point, rel: Vec<Point>)
              -> (Vec<Point>, Vec<Point>)
{
    let mut unknown = vec![];
    let mut same = vec![];

    for p2 in rel.into_iter() {
        match side_map.get_relation(p, p2) {
            Relation::Same => same.push(p2),
            Relation::Different => {}
            Relation::Unknown => unknown.push(p2),
            Relation::Conflict => panic!()
        }
    }

    unknown.sort();
    unknown.dedup();
    same.sort();
    same.dedup();
    (same, unknown)
}

fn update_conn(side_map: &mut SideMap, conn_map: &mut ConnectMap, p: Point) -> bool {
    let rel = {
        let a = conn_map.get_mut(p);
        if a.coord != p { return false }
        mem::replace(&mut a.unknown_rel, vec![])
    }.map_in_place(|p| conn_map.get(p).coord);

    let (same, unknown) = filter_rel(side_map, p, rel);
    {
        let a = conn_map.get_mut(p);
        a.side = side_map.get_side(p);
        a.unknown_rel = unknown;
    }

    let mut ret = false;
    for &p2 in same.iter() {
        ret |= conn_map.union(p, p2);
    }
    ret
}

fn create_conn_graph(conn_map: &mut ConnectMap, filter_side: Side) -> (Vec<Point>, Vec<Vec<usize>>)
{
    let mut pts = vec![];
    if filter_side != Side::Out {
        pts.push(Point(-1, -1))
    }

    for r in (0 .. conn_map.row()) {
        for c in (0 .. conn_map.column()) {
            let p = Point(r, c);
            let a = conn_map.get(p);
            if a.coord == p && a.side != filter_side {
                pts.push(p);
            }
        }
    }

    let mut graph = vec![];
    for &p in pts.iter() {
        let a = conn_map.get(p);
        let edges = a.unknown_rel.iter()
            .filter_map(|&p2| pts.position_elem(&p2))
            .collect::<Vec<_>>();
        graph.push(edges);
    }

    (pts, graph)
}

fn get_articulation(graph: &[Vec<usize>], v: usize) -> (Vec<usize>, Vec<bool>) {
    if graph.is_empty() { return (vec![], vec![]) }

    let mut visited = iter::repeat(false).take(graph.len()).collect::<Vec<_>>();
    let mut ord = iter::repeat(0).take(graph.len()).collect::<Vec<_>>();
    let mut low = iter::repeat(0).take(graph.len()).collect::<Vec<_>>();
    let mut arts = vec![];
    let mut ord_cnt = 0;
    dfs(graph, v, &mut visited[], &mut ord[], &mut low[], &mut ord_cnt, &mut arts);

    fn dfs(graph: &[Vec<usize>],
           v: usize, visited: &mut [bool], ord: &mut [usize], low: &mut [usize], 
           ord_cnt: &mut usize, arts: &mut Vec<usize>) {
        debug_assert!(!visited[v]);

        *ord_cnt += 1;
        visited[v] = true;
        ord[v] = *ord_cnt;
        low[v] = ord[v];

        let mut is_articulation = false;
        let mut num_child = 0;

        for &u in graph[v].iter() {
            if u == v { continue }

            if !visited[u] {
                num_child += 1;
                dfs(graph, u, visited, ord, low, ord_cnt, arts);
                low[v] = cmp::min(low[v], low[u]);
                if ord[v] != 1 && ord[v] <= low[u] {
                    is_articulation = true;
                }
            } else {
                low[v] = cmp::min(low[v], ord[u]);
            }
        }

        if ord[v] == 1 && num_child > 1 {
            is_articulation = true;
        }

        if is_articulation {
            arts.push(v);
        }
    }

    (arts, visited)
}

fn find_disconn_area(conn_map: &mut ConnectMap, pts: &[Point], visited: &[bool]) -> Vec<usize> {
    let mut disconn = vec![];
    for (u, &vis) in visited.iter().enumerate() {
        if !vis { disconn.push(u); }
    }
    if disconn.is_empty() {
        // All area is connected.
        return disconn
    }

    let mut sum = 0;
    for &v in disconn.iter() {
        sum += conn_map.get(pts[v]).sum_of_hint;
    }
    if sum == 0 {
        // Disconnected components does not contain any edges. It is a hole in
        // the filter_side area.
        return disconn;
    }

    let mut conn = vec![];
    for (u, &vis) in visited.iter().enumerate() {
        if vis { conn.push(u); }
    }
    let mut sum = 0;
    for &v in conn.iter() {
        sum += conn_map.get(pts[v]).sum_of_hint;
    }
    if sum == 0 {
        // Conencted area does not contain any edges. It is a hole in the
        // filter_side area.
        return conn
    }

    // Graph is splitted into more than two parts, but both parts contain edges.
    // This againsts connectivity rule.
    panic!()
}

fn splits(graph: &[Vec<usize>], v: usize,
          conn_map: &mut ConnectMap, pts: &[Point], side: Side) -> bool {
    if graph.is_empty() { return false }

    let mut contain_cnt = 0;
    let mut visited = iter::repeat(false).take(graph.len()).collect::<Vec<_>>();

    visited[v] = true;

    for &u in graph[v].iter() {
        if u == v || visited[u] { continue }

        if dfs(graph, u, &mut visited[], conn_map, pts, side) {
            contain_cnt += 1;
        }
    }

    fn dfs(graph: &[Vec<usize>], v: usize, visited: &mut [bool],
           conn_map: &mut ConnectMap, pts: &[Point], side: Side) -> bool {
        let mut contains = conn_map.get(pts[v]).side == side;
        visited[v] = true;

        for &u in graph[v].iter() {
            if u == v || visited[u] { continue }
            contains |= dfs(graph, u, visited, conn_map, pts, side);
        }
        contains
    }

    contain_cnt > 1
}

fn fill_by_connection(side_map: &mut SideMap) {
    let mut conn_map = ConnectMap::from_side_map(side_map);

    let mut rev = side_map.revision();
    loop {
        let mut updated = false;
        for r in (0 .. side_map.row()) {
            for c in (0 .. side_map.column()) {
                updated |= update_conn(side_map, &mut conn_map, Point(r, c));
            }
        }
        updated |= update_conn(side_map, &mut conn_map, Point(-1, -1));

        if updated {
            debug_assert_eq!(rev, side_map.revision());
            continue
        }

        for &set_side in [Side::In, Side::Out].iter() {
            let filter_side = if set_side == Side::In {
                Side::Out
            } else {
                Side::In
            };

            let (pts, graph) = create_conn_graph(&mut conn_map, filter_side);
            let (arts, visited) = get_articulation(&graph[], 0);

            let disconn = find_disconn_area(&mut conn_map, &pts[], &visited[]);
            for &v in disconn.iter() {
                side_map.set_side(pts[v], filter_side);
            }
            for &v in arts.iter() {
                let p = pts[v];

                if conn_map.get(p).side != set_side &&
                    splits(&graph[], v, &mut conn_map, &pts[], set_side) {
                    side_map.set_side(p, set_side);
                }
            }
        }

        if side_map.revision() != rev {
            rev = side_map.revision();
            continue
        }

        break
    }
}

fn solve_by_logic_once(side_map: &mut SideMap) {
    fill_by_num_place(side_map);
}

fn solve_by_local_property(side_map: &mut SideMap) {
    fill_by_line_nums(side_map);
    fill_by_relation(side_map);
}

fn solve_by_global_property(side_map: &mut SideMap) {
    fill_by_connection(side_map);
}

fn solve_by_logic(side_map: &mut SideMap) {
    let mut local_cnt = 0;
    let mut global_cnt = 0;
    let mut rev = side_map.revision();

    loop {
        local_cnt += 1;
        solve_by_local_property(side_map);
        if side_map.revision() != rev {
            rev = side_map.revision();
            continue
        }

        global_cnt += 1;
        solve_by_global_property(side_map);
        if side_map.revision() == rev {
            break;
        }

        rev = side_map.revision();
    }

    println!("{} {} {}", rev, local_cnt, global_cnt);
}

pub fn solve(board: &Board) -> Board {
    let mut side_map = SideMap::from_board(board);

    solve_by_logic_once(&mut side_map);
    solve_by_logic(&mut side_map);

    side_map.to_board()
}

