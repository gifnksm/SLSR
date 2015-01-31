use std::{cmp, fmt, iter, mem};
use board::{Board, Side};
use geom::{Geom, Point, Move};
use solver::connect_map::ConnectMap;
use solver::side_map::SideMap;
use solver::theorem::{Pattern, TheoremMatch, Theorem};
use solver::theorem_define::THEOREM_DEFINE;

mod connect_map;
mod side_map;
mod theorem;
mod theorem_define;

#[derive(Debug)]
pub struct LogicError;

impl fmt::Display for LogicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

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

fn initialize_theorem(side_map: &mut SideMap, theorem: &mut Vec<Theorem>)
                      -> SolverResult<()>
{
    let mut it = THEOREM_DEFINE.iter()
        .flat_map(|th| {
            th.parse::<Theorem>().unwrap().all_rotations().into_iter()
        });

    let mut hint_theorem = [vec![], vec![], vec![], vec![]];
    let mut nonhint_theorem = vec![];

    for theo in it {
        match theo.head() {
            Pattern::Hint(x, _) => hint_theorem[x as usize].push(theo),
            Pattern::Edge(..) => nonhint_theorem.push(theo)
        }
    }

    for r in (0 .. side_map.row()) {
        for c in (0 .. side_map.column()) {
            let p = Point(r, c);
            match side_map.hint()[p] {
                Some(x) => {
                    for theo in hint_theorem[x as usize].iter() {
                        let o = match theo.head() {
                            Pattern::Hint(_, p) => p,
                            _ => panic!()
                        };

                        match try!(theo.clone().shift(p - o).matches(side_map)) {
                            TheoremMatch::Complete(result) => {
                                for pat in result.iter() {
                                    pat.apply(side_map);
                                }
                            }
                            TheoremMatch::Partial(theo) => {
                                theorem.push(theo)
                            }
                            TheoremMatch::Conflict => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for theo in nonhint_theorem.into_iter() {
        let sz = theo.size();
        for r in (1 - sz.0 .. side_map.row() + sz.0 - 1) {
            for c in (1 - sz.1 .. side_map.column() + sz.1 - 1) {
                match try!(theo.clone().shift(Move(r, c)).matches(side_map)) {
                    TheoremMatch::Complete(result) => {
                        for pat in result.iter() {
                            pat.apply(side_map);
                        }
                    }
                    TheoremMatch::Partial(theo) => {
                        theorem.push(theo)
                    }
                    TheoremMatch::Conflict => {}
                }
            }
        }
    }

    Ok(())
}

fn solve_by_theorem(side_map: &mut SideMap, theorem: &mut Vec<Theorem>)
                   -> SolverResult<()>
{
    let mut rev = side_map.revision();
    loop {
        let new_theorem = Vec::with_capacity(theorem.len());

        for theo in mem::replace(theorem, new_theorem).into_iter() {
            match try!(theo.matches(side_map)) {
                TheoremMatch::Complete(result) => {
                    for pat in result.iter() {
                        pat.apply(side_map);
                    }
                }
                TheoremMatch::Partial(theo) => {
                    theorem.push(theo)
                }
                TheoremMatch::Conflict => {}
            }
        }

        if side_map.revision() == rev {
            break;
        }
        rev = side_map.revision()
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

    pts.sort();

    let graph = pts.iter().map(|&p| {
        conn_map.get(p).unknown_edge().iter()
            .filter_map(|&p2| pts.binary_search(&p2).ok())
            .collect::<Vec<_>>()
    }).collect();

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
                     -> SolverResult<Vec<usize>>
{
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
    // This may be valid in some situation, so, return empty.
    Ok(vec![])
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

fn solve_by_connection(side_map: &mut SideMap, conn_map: &mut ConnectMap)
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
                    splits(&graph[], v, conn_map, &pts[], set_side)
                {
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

fn solve_by_logic(
    side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>,
    theorem: &mut Vec<Theorem>)
    -> SolverResult<()>
{
    let mut rev = side_map.revision();

    while !side_map.all_filled() {
        try!(solve_by_theorem(side_map, theorem));

        if side_map.all_filled() {
            return Ok(())
        }
        if side_map.revision() != rev {
            rev = side_map.revision();
            continue
        }

        if conn_map.is_none() {
            *conn_map = Some(ConnectMap::from_side_map(side_map));
        }
        try!(solve_by_connection(side_map, conn_map.as_mut().unwrap()));
        if side_map.revision() == rev {
            break;
        }

        rev = side_map.revision();
    }

    Ok(())
}

fn get_unknown_points(conn_map: &mut ConnectMap) -> Vec<Point> {
    let mut pts = vec![];

    for r in (0 .. conn_map.row()) {
        for c in (0 .. conn_map.column()) {
            let p = Point(r, c);
            let a = conn_map.get(p);
            if p != a.coord() { continue }
            if a.side() != State::Unknown { continue }
            pts.push((p, a.unknown_edge().len()));
        }
    }

    pts.sort_by(|a, b| a.1.cmp(&b.1));
    pts.into_iter().map(|pair| pair.0).collect()
}

fn solve_by_backtracking_one_step(
    side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>,
    theorem: &mut Vec<Theorem>, pts: &[Point])
    -> SolverResult<bool>
{
    let rev = side_map.revision();

    for &p in pts.iter() {
        match side_map.get_side(p) {
            State::Fixed(_) => continue,
            State::Unknown => {}
            State::Conflict => { return Err(LogicError) }
        }

        let mut sm0 = side_map.clone();
        let mut cm0 = conn_map.clone();
        let mut th0 = theorem.clone();
        sm0.set_inside(p);

        if solve_by_logic(&mut sm0, &mut cm0, &mut th0).is_err() {
            side_map.set_outside(p);
            try!(solve_by_logic(side_map, conn_map, theorem));
            continue
        }

        let mut sm1 = side_map.clone();
        let mut cm1 = conn_map.clone();
        let mut th1 = theorem.clone();
        sm1.set_outside(p);

        if solve_by_logic(&mut sm1, &mut cm1, &mut th1).is_err() {
            *side_map = sm0;
            *conn_map = cm0;
            *theorem  = th0;
        }
    }

    Ok(side_map.revision() != rev)
}

fn check_answer(side_map: &mut SideMap, conn_map: &mut Option<ConnectMap>)
                -> SolverResult<()>
{
    if conn_map.is_none() {
        *conn_map = Some(ConnectMap::from_side_map(side_map));
    }
    let conn_map = conn_map.as_mut().unwrap();
    try!(conn_map.sync(side_map));
    if conn_map.count_area() != 2 {
        return Err(LogicError)
    }

    Ok(())
}

pub fn solve(board: &Board) -> Result<Board, LogicError> {
    let mut side_map = SideMap::from_board(board);
    let mut theorem = vec![];
    try!(initialize_theorem(&mut side_map, &mut theorem));

    let mut queue = vec![(side_map, None, theorem)];

    'failure: while let Some((mut side_map, mut conn_map, mut theorem)) = queue.pop() {
        if solve_by_logic(&mut side_map, &mut conn_map, &mut theorem).is_err() {
            continue
        }

        if side_map.all_filled() {
            if check_answer(&mut side_map, &mut conn_map).is_err() {
                continue
            }
            return side_map.to_board()
        }

        assert!(conn_map.is_some());
        let mut pts = get_unknown_points(conn_map.as_mut().unwrap());
        loop {
            match solve_by_backtracking_one_step(
                &mut side_map, &mut conn_map, &mut theorem, &pts[])
            {
                Ok(true) => {
                    if side_map.all_filled() {
                        if check_answer(&mut side_map, &mut conn_map).is_err() {
                            continue
                        }
                        return side_map.to_board()
                    }
                    pts = get_unknown_points(conn_map.as_mut().unwrap());
                }
                Ok(false) => break,
                Err(_) => continue 'failure,
            }
        }

        let p = *pts.last().unwrap();
        let mut side_map_0 = side_map.clone();
        let conn_map_0 = conn_map.clone();
        let mut side_map_1 = side_map;
        let conn_map_1 = conn_map;
        side_map_0.set_outside(p);
        side_map_1.set_inside(p);
        queue.push((side_map_0, conn_map_0, theorem.clone()));
        queue.push((side_map_1, conn_map_1, theorem));
    }

    Err(LogicError)
}

