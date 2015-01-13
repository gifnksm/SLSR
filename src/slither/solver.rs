use std::{cmp, mem};
use std::iter::{self, FromIterator};
use union_find::{UnionFind, UFValue, Merge};
use board::{Board, Relation, Side};
use geom::{Geom, Point, Size, UP, LEFT, RIGHT, DOWN, UCW0, UCW90, UCW180, UCW270};
use hint::Hint;

fn fill_by_num_place(board: &mut Board, hint: &Hint) {
    // Corner points
    let corners = [(Point(0, 0), UCW0),
                   (Point(board.row() - 1, 0), UCW90),
                   (Point(board.row() - 1, board.column() - 1), UCW180),
                   (Point(0, board.column() - 1), UCW270)];
    for &(p, rot) in corners.iter() {
        match hint[p] {
            Some(0) => {}
            Some(1) => {
                board.set_same(p, p + rot * UP);
                board.set_same(p, p + rot * LEFT);
            }
            Some(2) => {
                board.set_different(p + rot * RIGHT, p + rot * (RIGHT + UP));
                board.set_different(p + rot * DOWN,  p + rot * (DOWN + LEFT));
            }
            Some(3) => {
                board.set_different(p, p + rot * UP);
                board.set_different(p, p + rot * LEFT);
            }
            _ => {}
        }
    }

    // All points
    for r in (0 .. board.row()) {
        for c in (0 .. board.column()) {
            let p = Point(r, c);
            match hint[p] {
                Some(0) => {
                    for &rot in [UCW0, UCW90, UCW180, UCW270].iter() {
                        let r  = p + rot * RIGHT;
                        let dr = p + rot * (DOWN + RIGHT);
                        board.set_same(p, r);

                        //           -
                        // 0 3 =>  0x3|
                        //           -
                        if hint[r] == Some(3) {
                            board.set_different(r, r + rot * UP);
                            board.set_different(r, r + rot * (UP + RIGHT));
                            board.set_different(r, r + rot * RIGHT);
                            board.set_different(r, r + rot * (DOWN + RIGHT));
                            board.set_different(r, r + rot * DOWN);
                        }

                        // 0      0x
                        //     => x -
                        //   3     |3
                        if hint[dr] == Some(3) {
                            board.set_different(dr, dr + rot * UP);
                            board.set_different(dr, dr + rot * LEFT);
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
                        if hint[r] == Some(3) {
                            board.set_different(p, p + rot * LEFT);
                            board.set_different(p, r);
                            board.set_different(r, r + rot * RIGHT);

                            board.set_same(p + rot * UP,   r + rot * UP);
                            board.set_same(p + rot * DOWN, r + rot * DOWN);
                            board.set_different(p + rot * UP, r + rot * DOWN);
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
                        while hint[dr] == Some(2) {
                            dr = dr + rot * (DOWN + RIGHT);
                            cnt += 1;
                        }
                        if hint[dr] == Some(3) {
                            board.set_different(p, p + rot * UP);
                            board.set_different(p, p + rot * (UP + LEFT));
                            board.set_different(p, p + rot * LEFT);

                            board.set_different(dr, dr + rot * RIGHT);
                            board.set_different(dr, dr + rot * (RIGHT + DOWN));
                            board.set_different(dr, dr + rot * DOWN);

                            let mut t = p;
                            for _ in (0 .. cnt) {
                                board.set_different(t + rot * RIGHT,
                                                    t + rot * DOWN);
                                t = t + rot * (DOWN + RIGHT);
                            }

                            if hint[p + rot * (RIGHT + RIGHT)] == Some(3) {
                                board.set_same(p + rot * RIGHT, p + rot * (UP + RIGHT));
                            }
                            if hint[p + rot * (DOWN + DOWN)] == Some(3) {
                                board.set_same(p + rot * DOWN, p + rot * (DOWN + LEFT));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn fill_by_line_nums(board: &mut Board, hint: &Hint) {
    for r in (0 .. board.row()) {
        for c in (0 .. board.column()) {
            let p = Point(r, c);
            let mut sames    = [None; 4];
            let mut diffs    = [None; 4];
            let mut unknowns = [None; 4];
            let mut num_same    = 0;
            let mut num_diff    = 0;
            let mut num_unknown = 0;

            for &dir in [UP, RIGHT, DOWN, LEFT].iter() {
                match board.get_relation(p, p + dir) {
                    Relation::Same      => { sames[num_same] = Some(dir); num_same += 1; }
                    Relation::Different => { diffs[num_diff] = Some(dir); num_diff += 1; }
                    Relation::Unknown   => { unknowns[num_unknown] = Some(dir); num_unknown += 1; }
                    _ => panic!() // FIXME
                }
            }

            if num_diff == 3 && num_unknown == 1 {
                board.set_same(p, p + unknowns[0].unwrap());
                continue
            }

            match hint[p] {
                Some(x) if (num_diff as u8) == x => {
                    for i in (0 .. num_unknown) {
                        board.set_same(p, p + unknowns[i].unwrap());
                    }
                }
                Some(x) if (num_same as u8) == 4 - x => {
                    for i in (0 .. num_unknown) {
                        board.set_different(p, p + unknowns[i].unwrap());
                    }
                }
                _ => {}
            }
        }
    }
}

fn fill_by_relation(board: &mut Board, hint: &Hint) {
    for r in (0 .. board.row()) {
        for c in (0 .. board.column()) {
            let p = Point(r, c);

            for &rot in [UCW0, UCW90, UCW180, UCW270].iter() {
                let u = p + rot * UP;
                let d = p + rot * DOWN;
                let r = p + rot * RIGHT;
                let l = p + rot * LEFT;
                let ur = p + rot * (UP + RIGHT);
                let ul = p + rot * (UP + LEFT);

                if board.is_different(p, u) {
                    if board.is_different(p, r) {
                        board.set_different(p, ur);
                    }
                    if board.is_different(r, ur) {
                        board.set_same(p, r);
                    }
                }

                match board.get_relation(u, r) {
                    Relation::Same => {
                        match hint[p] {
                            Some(1) => {
                                board.set_same(p, u);
                                board.set_same(p, r);
                                board.set_different(l, d);
                            }
                            Some(2) => {
                                board.set_same(l, d);
                                board.set_different(u, l);
                            }
                            Some(3) => {
                                board.set_different(p, u);
                                board.set_different(p, r);
                                board.set_different(l, d);
                            }
                            _ => {}
                        }
                    }
                    Relation::Different => {
                        match hint[p] {
                            Some(1) => {
                                board.set_same(p, l);
                                board.set_same(p, d);
                            }
                            Some(2) => {
                                board.set_different(l, d);
                            }
                            Some(3) => {
                                board.set_different(p, l);
                                board.set_different(p, d);
                            }
                            _ => {}
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                match board.get_relation(u, ur) {
                    Relation::Same => {
                        if hint[p] == Some(3) && hint[r] == Some(1) {
                            board.set_different(p, u);
                            board.set_same(r, r + rot * RIGHT);
                            board.set_same(r, r + rot * DOWN);
                        }
                    }
                    Relation::Different => {
                        if hint[p] == Some(3) {
                            board.set_different(p, d);
                            board.set_different(p, l);
                            board.set_same(ur, r);
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                match board.get_relation(u, ul) {
                    Relation::Same => {
                        if hint[p] == Some(3) && hint[l] == Some(1) {
                            board.set_different(p, u);
                            board.set_same(l, l + rot * LEFT);
                            board.set_same(l, l + rot * DOWN);
                        }
                    }
                    Relation::Different => {
                        if hint[p] == Some(3) {
                            board.set_different(p, d);
                            board.set_different(p, r);
                            board.set_same(ul, l);
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

                match board.get_relation(u, d) {
                    Relation::Same => {
                        match hint[p] {
                            Some(1) => {
                                board.set_same(p, u);
                                board.set_different(l, r);
                            }
                            Some(2) => {
                                board.set_different(u, l);
                                board.set_same(l, r);
                            }
                            Some(3) => {
                                board.set_different(p, u);
                                board.set_different(l, r);
                            }
                            _ => {}
                        }
                    }
                    Relation::Different => {
                        match hint[p] {
                            Some(1) => {
                                board.set_same(p, l);
                                board.set_same(p, r);
                            }
                            Some(2) => {
                                board.set_different(l, r);
                            }
                            Some(3) => {
                                board.set_different(p, l);
                                board.set_different(p, r);
                            }
                            _ => {}
                        }
                    }
                    Relation::Unknown => {}
                    Relation::Conflict => panic!()
                }

                if (board.is_different(p, r) || board.is_different(p, d)) &&
                    (board.is_different(dr, r) || board.is_different(dr, d)) {
                        board.set_different(r, d);
                    }
            }
        }
    }
}

#[derive(Show)]
struct Area {
    coord: Point,
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
        let area = Area {
            coord: coord,
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

    fn from_board(board: &mut Board, hint: &Hint) -> ConnectMap {
        let mut map = ConnectMap::new(board.size(), |p| {
            let sum = hint[p].unwrap_or(0);

            let mut rel = vec![];
            if board.contains(p) {
                for &r in [UP, RIGHT, DOWN, LEFT].iter() {
                    if board.get_relation(p, p + r) == Relation::Unknown {
                        rel.push(p + r);
                    }
                }
            } else {
                for r in (0 .. board.row()) {
                    for &c in [0, board.column() - 1].iter() {
                        let p2 = Point(r, c);
                        if board.get_relation(p, p2) == Relation::Unknown {
                            rel.push(p2);
                        }
                    }
                }
                for c in (0 .. board.column()) {
                    for &r in [0, board.row() - 1].iter() {
                        let p2 = Point(r, c);
                        if board.get_relation(p, p2) == Relation::Unknown {
                            rel.push(p2);
                        }
                    }
                }
            }
            rel.sort();
            rel.dedup();

            Area {
                coord: p,
                unknown_rel: rel,
                sum_of_hint: sum as u32,
                size: 1
            }
        });

        for r in (0 .. board.row()) {
            for c in (0 .. board.column()) {
                let p = Point(r, c);
                for &r in [UP, RIGHT, DOWN, LEFT].iter() {
                    let p2 = p + r;
                    if board.get_relation(p, p2) == Relation::Same {
                        map.union(p, p2);
                    }
                }
            }
        }
        map
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

fn filter_rel(board: &mut Board, p: Point, rel: Vec<Point>)
              -> (Vec<Point>, Vec<Point>)
{
    let mut unknown = vec![];
    let mut same = vec![];

    for p2 in rel.into_iter() {
        match board.get_relation(p, p2) {
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

fn update_conn(board: &mut Board, map: &mut ConnectMap, p: Point) -> bool {
    let rel = {
        let a = map.get_mut(p);
        if a.coord != p { return false }
        mem::replace(&mut a.unknown_rel, vec![])
    }.map_in_place(|p| map.get(p).coord);

    let (same, unknown) = filter_rel(board, p, rel);
    {
        let a = map.get_mut(p);
        a.unknown_rel = unknown;
    }

    let mut ret = false;
    for &p2 in same.iter() {
        ret |= map.union(p, p2);
    }
    ret
}

fn create_conn_graph(board: &mut Board, map: &mut ConnectMap, filter_side: Side)
                     -> (Vec<Point>, Vec<Vec<usize>>)
{
    let mut pts = vec![];
    if filter_side != Side::Out {
        pts.push(Point(-1, -1))
    }
    for r in (0 .. board.row()) {
        for c in (0 .. board.column()) {
            let p = Point(r, c);
            if board.get_side(p) == filter_side {
                continue
            }
            if map.get(p).coord == p {
                pts.push(p);
            }
        }
    }

    let mut graph = vec![];
    for &p in pts.iter() {
        let a = map.get(p);
        let edges = a.unknown_rel.iter()
            .filter_map(|&p2| pts.position_elem(&p2))
            .collect::<Vec<_>>();
        graph.push(edges);
    }

    (pts, graph)
}

fn get_articulation(graph: &[Vec<usize>]) -> (Vec<usize>, Vec<bool>) {
    if graph.is_empty() { return (vec![], vec![]) }

    let mut visited = iter::repeat(false).take(graph.len()).collect::<Vec<_>>();
    let mut ord = iter::repeat(0).take(graph.len()).collect::<Vec<_>>();
    let mut low = iter::repeat(0).take(graph.len()).collect::<Vec<_>>();
    let mut arts = vec![];
    let mut ord_cnt = 0;
    dfs(graph, 0, &mut visited[], &mut ord[], &mut low[], &mut ord_cnt, &mut arts);

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

fn check_disconn(board: &mut Board, map: &mut ConnectMap,
                 pts: &[Point], visited: &[bool], filter_side: Side) {
    let mut disconn = vec![];
    for (u, &vis) in visited.iter().enumerate() {
        if !vis { disconn.push(u); }
    }
    if !disconn.is_empty() {
        let mut sum = 0;
        for &v in disconn.iter() {
            sum += map.get(pts[v]).sum_of_hint;
        }
        if sum == 0 {
            for &v in disconn.iter() {
                board.set_side(pts[v], filter_side);
            }
        } else {
            let mut conn = vec![];
            for (u, &vis) in visited.iter().enumerate() {
                if vis { conn.push(u); }
            }
            let mut sum = 0;
            for &v in conn.iter() {
                sum += map.get(pts[v]).sum_of_hint;
            }
            if sum == 0 {
                for &v in conn.iter() {
                    board.set_side(pts[v], filter_side);
                }
            }
        }
    }
}

fn splits(graph: &[Vec<usize>], v: usize,
          board: &mut Board, pts: &[Point], ty: Side) -> bool {
    if graph.is_empty() { return false }

    let mut contain_cnt = 0;
    let mut visited = iter::repeat(false).take(graph.len()).collect::<Vec<_>>();

    visited[v] = true;

    for &u in graph[v].iter() {
        if u == v || visited[u] { continue }

        if dfs(graph, u, &mut visited[], board, pts, ty) {
            contain_cnt += 1;
        }
    }

    fn dfs(graph: &[Vec<usize>], v: usize, visited: &mut [bool],
           board: &mut Board, pts: &[Point], ty: Side) -> bool {
        let mut contains = board.get_side(pts[v]) == ty;
        visited[v] = true;

        for &u in graph[v].iter() {
            if u == v || visited[u] { continue }
            contains |= dfs(graph, u, visited, board, pts, ty);
        }
        contains
    }

    contain_cnt > 1
}

fn fill_by_connection(board: &mut Board, hint: &Hint) {
    let mut map = ConnectMap::from_board(board, hint);

    let mut rev = board.revision();
    loop {
        let mut updated = false;
        for r in (0 .. board.row()) {
            for c in (0 .. board.column()) {
                updated |= update_conn(board, &mut map, Point(r, c));
            }
        }
        updated |= update_conn(board, &mut map, Point(-1, -1));

        if updated {
            debug_assert_eq!(rev, board.revision());
            continue
        }

        for &set_side in [Side::In, Side::Out].iter() {
            let filter_side = if set_side == Side::In {
                Side::Out
            } else {
                Side::In
            };

            let (pts, graph) = create_conn_graph(board, &mut map, filter_side);
            let (arts, visited) = get_articulation(&graph[]);
            check_disconn(board, &mut map, &pts[], &visited[], filter_side);

            for &v in arts.iter() {
                let p = pts[v];

                if board.get_side(p) == set_side {
                    continue
                }
                if splits(&graph[], v, board, &pts[], set_side) {
                    board.set_side(p, set_side);
                }
            }
        }

        if board.revision() != rev {
            rev = board.revision();
            continue
        }

        break
    }
}

fn solve_by_logic_once(board: &mut Board, hint: &Hint) {
    fill_by_num_place(board, hint);
}

fn solve_by_local_property(board: &mut Board, hint: &Hint) {
    fill_by_line_nums(board, hint);
    fill_by_relation(board, hint);
}

fn solve_by_global_property(board: &mut Board, hint: &Hint) {
    fill_by_connection(board, hint);
}

fn solve_by_logic(board: &mut Board, hint: &Hint) {
    let mut local_cnt = 0;
    let mut global_cnt = 0;
    let mut rev = board.revision();

    loop {
        solve_by_local_property(board, hint);
        if board.revision() != rev {
            rev = board.revision();
            local_cnt += 1;
            continue
        }

        solve_by_global_property(board, hint);
        if board.revision() == rev {
            break;
        }

        rev = board.revision();
        global_cnt += 1;
    }

    println!("{} {} {}", rev, local_cnt, global_cnt);
}

pub fn solve(board: &mut Board, hint: &Hint) {
    solve_by_logic_once(board, hint);
    solve_by_logic(board, hint);
}

