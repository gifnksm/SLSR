use std::cmp;
use slsr_core::puzzle::Side;
use slsr_core::geom::{CellId, Geom};

use ::{State, SolverResult};
use ::model::connect_map::ConnectMap;
use ::model::side_map::SideMap;

fn create_conn_graph(conn_map: &mut ConnectMap, exclude_side: Side)
                     -> (Vec<CellId>, Vec<Vec<usize>>)
{
    let mut pts = (0..conn_map.cell_len())
        .map(CellId::new)
        .filter_map(|p| {
            let a = conn_map.get(p);
            if a.coord() == p && a.side() != State::Fixed(exclude_side) {
                Some(p)
            } else {
                None
            }
        }).collect::<Vec<_>>();
    pts.sort();

    let mut verts = vec![None; conn_map.cell_len()];
    for (i, &p) in pts.iter().enumerate() {
        verts[p.id()] = Some(i);
    }

    let graph = pts.iter().map(|&p| {
        conn_map.get(p).unknown_edge().iter()
            .filter_map(|p2| verts[p2.id()])
            .collect::<Vec<_>>()
    }).collect();

    (pts, graph)
}

fn get_articulation(graph: &[Vec<usize>], v: usize) -> (Vec<usize>, Vec<bool>) {
    if graph.is_empty() { return (vec![], vec![]) }

    let mut visited = vec![false; graph.len()];
    let mut ord = vec![0; graph.len()];
    let mut low = vec![0; graph.len()];
    let mut arts = vec![];
    let mut ord_cnt = 0;
    dfs(graph, v, &mut visited, &mut ord, &mut low, &mut ord_cnt, &mut arts);
    return (arts, visited);

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

        for &u in &graph[v] {
            if u == v { continue }

            if !visited[u] {
                dfs(graph, u, visited, ord, low, ord_cnt, arts);

                num_child += 1;
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
}

fn find_disconn_area(conn_map: &mut ConnectMap, pts: &[CellId], visited: &[bool])
                     -> SolverResult<Vec<usize>>
{
    let mut disconn = vec![];
    let mut disconn_sum = 0;
    for (u, &vis) in visited.iter().enumerate() {
        if !vis {
            disconn.push(u);
            disconn_sum += conn_map.get(pts[u]).sum_of_hint();
        }
    }
    if disconn.is_empty() {
        //  All area is connected
        return Ok(vec![])
    }
    if disconn_sum == 0 {
        // Disconnected components does not contain any edges. It is a hole in
        // the exclude_side area.
        return Ok(disconn)
    }

    let mut conn = vec![];
    let mut conn_sum = 0;
    for (u, &vis) in visited.iter().enumerate() {
        if vis {
            conn.push(u);
            conn_sum += conn_map.get(pts[u]).sum_of_hint();
        }
    }
    if conn_sum == 0 {
        // Conencted area does not contain any edges. It is a hole in the
        // exclude_side area.
        return Ok(conn)
    }

    // Graph is splitted into more than two parts, but both parts contain edges.
    // This may be valid in some situation, so, return empty.
    Ok(vec![])
}

fn splits(graph: &[Vec<usize>], v: usize,
          conn_map: &mut ConnectMap, pts: &[CellId], set_side: Side) -> bool {
    if graph.is_empty() { return false }

    let mut contain_cnt = 0;
    let mut visited = vec![false; graph.len()];

    visited[v] = true;

    for &u in &graph[v] {
        if u == v || visited[u] { continue }

        if dfs(graph, u, &mut visited, conn_map, pts, set_side) {
            contain_cnt += 1;
        }
    }

    return contain_cnt > 1;


    fn dfs(graph: &[Vec<usize>], v: usize, visited: &mut [bool],
           conn_map: &mut ConnectMap, pts: &[CellId], set_side: Side) -> bool {
        let mut contains = conn_map.get(pts[v]).side() == State::Fixed(set_side);
        visited[v] = true;

        for &u in &graph[v] {
            if u == v || visited[u] { continue }
            contains |= dfs(graph, u, visited, conn_map, pts, set_side);
        }
        contains
    }
}

pub fn run(side_map: &mut SideMap, conn_map: &mut ConnectMap)
    -> SolverResult<()>
{
    try!(conn_map.sync(side_map));

    let sides = &[(Side::In, Side::Out),
                  (Side::Out, Side::In)];

    for &(set_side, exclude_side) in sides {
        let (pts, graph) = create_conn_graph(conn_map, exclude_side);
        let (arts, visited) = get_articulation(&graph, 0);

        if set_side == Side::Out || conn_map.sum_of_hint() != 0 {
            // If there is no edge in puzzle (sum_of_hint == 0) and set_side ==
            // Side::In, any disconnected area can be loop.
            let disconn = try!(find_disconn_area(conn_map, &pts, &visited));
            for v in disconn {
                side_map.set_side(pts[v], exclude_side);
            }
        }

        for v in arts {
            let p = pts[v];
            if conn_map.get(p).side() == State::Fixed(set_side) {
                continue
            }
            if splits(&graph, v, conn_map, &pts, set_side) {
                side_map.set_side(p, set_side);
            }
        }
    }

    Ok(())
}
