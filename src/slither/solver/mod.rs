use std::{cmp, iter};
use board::{Board, Edge, Side};
use geom::{Geom, Point, UP, LEFT, RIGHT, DOWN, UCW0, UCW90, UCW180, UCW270};
use solver::connect_map::ConnectMap;
use solver::side_map::SideMap;

mod connect_map;
mod side_map;

#[derive(Show)]
pub struct LogicError;

type SolverResult<T> = Result<T, LogicError>;

#[derive(Copy, Clone, Show, Eq, PartialEq)]
pub enum State<T> {
    Fixed(T), Unknown, Conflict
}

impl<T> State<T> {
    pub fn into_option(self) -> Result<Option<T>, LogicError> {
        match self {
            State::Fixed(st) => Ok(Some(st)),
            State::Unknown => Ok(None),
            State::Conflict => Err(LogicError)
        }
    }
}

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

fn fill_by_line_nums(side_map: &mut SideMap) -> SolverResult<()> {
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
                match side_map.get_edge(p, p + dir) {
                    State::Fixed(Edge::Cross) => { sames[num_same] = Some(dir); num_same += 1; }
                    State::Fixed(Edge::Line)  => { diffs[num_diff] = Some(dir); num_diff += 1; }
                    State::Unknown            => { unknowns[num_unknown] = Some(dir); num_unknown += 1; }
                    State::Conflict => return Err(LogicError)
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
    Ok(())
}

fn fill_by_edge(side_map: &mut SideMap) -> SolverResult<()> {
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

                match side_map.get_edge(u, r) {
                    State::Fixed(Edge::Cross) => {
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
                    State::Fixed(Edge::Line) => {
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
                    State::Unknown => {}
                    State::Conflict => return Err(LogicError)
                }

                match side_map.get_edge(u, ur) {
                    State::Fixed(Edge::Cross) => {
                        if side_map.hint()[p] == Some(3) &&
                            side_map.hint()[r] == Some(1) {
                            side_map.set_different(p, u);
                            side_map.set_same(r, r + rot * RIGHT);
                            side_map.set_same(r, r + rot * DOWN);
                        }
                    }
                    State::Fixed(Edge::Line) => {
                        if side_map.hint()[p] == Some(3) {
                            side_map.set_different(p, d);
                            side_map.set_different(p, l);
                            side_map.set_same(ur, r);
                        }
                    }
                    State::Unknown => {}
                    State::Conflict => return Err(LogicError)
                }

                match side_map.get_edge(u, ul) {
                    State::Fixed(Edge::Cross) => {
                        if side_map.hint()[p] == Some(3) &&
                            side_map.hint()[l] == Some(1) {
                            side_map.set_different(p, u);
                            side_map.set_same(l, l + rot * LEFT);
                            side_map.set_same(l, l + rot * DOWN);
                        }
                    }
                    State::Fixed(Edge::Line) => {
                        if side_map.hint()[p] == Some(3) {
                            side_map.set_different(p, d);
                            side_map.set_different(p, r);
                            side_map.set_same(ul, l);
                        }
                    }
                    State::Unknown => {}
                    State::Conflict => return Err(LogicError)
                }
            }

            for &rot in [UCW0, UCW90].iter() {
                let u = p + rot * UP;
                let d = p + rot * DOWN;
                let r = p + rot * RIGHT;
                let l = p + rot * LEFT;
                let dr = p + rot * (DOWN + RIGHT);

                match side_map.get_edge(u, d) {
                    State::Fixed(Edge::Cross) => {
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
                    State::Fixed(Edge::Line) => {
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
                    State::Unknown => {}
                    State::Conflict => return Err(LogicError)
                }

                if (side_map.is_different(p, r) || side_map.is_different(p, d)) &&
                    (side_map.is_different(dr, r) || side_map.is_different(dr, d)) {
                        side_map.set_different(r, d);
                    }
            }
        }
    }
    Ok(())
}

fn create_conn_graph(conn_map: &mut ConnectMap, filter_side: Side)
                     -> (Vec<Point>, Vec<Vec<usize>>)
{
    let mut pts = vec![];
    if filter_side != Side::Out {
        pts.push(Point(-1, -1))
    }

    for r in (0 .. conn_map.row()) {
        for c in (0 .. conn_map.column()) {
            let p = Point(r, c);
            let a = conn_map.get(p);
            if a.coord() == p && a.side() != State::Fixed(filter_side) {
                pts.push(p);
            }
        }
    }

    let mut graph = vec![];
    for &p in pts.iter() {
        let a = conn_map.get(p);
        let edges = a.unknown_edge().iter()
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

fn find_disconn_area(conn_map: &mut ConnectMap, pts: &[Point], visited: &[bool])
                     -> SolverResult<Vec<usize>> {
    let mut disconn = vec![];
    for (u, &vis) in visited.iter().enumerate() {
        if !vis { disconn.push(u); }
    }
    if disconn.is_empty() {
        // All area is connected.
        return Ok(disconn)
    }

    let mut sum = 0;
    for &v in disconn.iter() {
        sum += conn_map.get(pts[v]).sum_of_hint();
    }
    if sum == 0 {
        // Disconnected components does not contain any edges. It is a hole in
        // the filter_side area.
        return Ok(disconn)
    }

    let mut conn = vec![];
    for (u, &vis) in visited.iter().enumerate() {
        if vis { conn.push(u); }
    }
    let mut sum = 0;
    for &v in conn.iter() {
        sum += conn_map.get(pts[v]).sum_of_hint();
    }
    if sum == 0 {
        // Conencted area does not contain any edges. It is a hole in the
        // filter_side area.
        return Ok(conn)
    }

    // Graph is splitted into more than two parts, but both parts contain edges.
    // This againsts connectivity rule.
    Err(LogicError)
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
        let mut contains = conn_map.get(pts[v]).side() == State::Fixed(side);
        visited[v] = true;

        for &u in graph[v].iter() {
            if u == v || visited[u] { continue }
            contains |= dfs(graph, u, visited, conn_map, pts, side);
        }
        contains
    }

    contain_cnt > 1
}

fn fill_by_connection(side_map: &mut SideMap, conn_map: &mut ConnectMap)
    -> SolverResult<()>
{
    let mut rev = side_map.revision();
    loop {
        try!(conn_map.sync(side_map));

        for &set_side in [Side::In, Side::Out].iter() {
            let filter_side = if set_side == Side::In {
                Side::Out
            } else {
                Side::In
            };

            let (pts, graph) = create_conn_graph(conn_map, filter_side);
            let (arts, visited) = get_articulation(&graph[], 0);

            let disconn = try!(find_disconn_area(conn_map, &pts[], &visited[]));
            for &v in disconn.iter() {
                side_map.set_side(pts[v], filter_side);
            }
            for &v in arts.iter() {
                let p = pts[v];

                if conn_map.get(p).side() != State::Fixed(set_side) &&
                    splits(&graph[], v, conn_map, &pts[], set_side) {
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
    Ok(())
}

fn solve_by_logic_once(side_map: &mut SideMap) {
    fill_by_num_place(side_map);
}

fn solve_by_local_property(side_map: &mut SideMap) -> SolverResult<()> {
    try!(fill_by_line_nums(side_map));
    try!(fill_by_edge(side_map));
    Ok(())
}

fn solve_by_global_property(side_map: &mut SideMap, conn_map: &mut ConnectMap)
    -> SolverResult<()>
{
    try!(fill_by_connection(side_map, conn_map));
    Ok(())
}

fn solve_by_logic(side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>)
    -> SolverResult<()>
{
    let mut local_cnt = 0;
    let mut global_cnt = 0;
    let mut rev = side_map.revision();

    loop {
        local_cnt += 1;
        try!(solve_by_local_property(side_map));
        if side_map.revision() != rev {
            rev = side_map.revision();
            continue
        }

        if conn_map.is_none() {
            *conn_map = Some(ConnectMap::from_side_map(side_map));
        }
        global_cnt += 1;
        try!(solve_by_global_property(side_map, conn_map.as_mut().unwrap()));
        if side_map.revision() == rev {
            break;
        }

        rev = side_map.revision();
    }

    println!("{} {} {}", rev, local_cnt, global_cnt);
    Ok(())
}

pub fn solve(board: &Board) -> Result<Board, LogicError> {
    let mut side_map = SideMap::from_board(board);
    let mut conn_map = None;

    solve_by_logic_once(&mut side_map);
    try!(solve_by_logic(&mut side_map, &mut conn_map));

    side_map.to_board()
}

