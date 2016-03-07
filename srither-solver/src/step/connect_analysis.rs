// Copyright (c) 2016 srither-solver developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{cmp, usize};
use srither_core::puzzle::Side;
use srither_core::geom::{CellId, Geom};

use SolverResult;
use model::{ConnectMap, SideMap, State};

fn create_conn_graph(conn_map: &mut ConnectMap,
                     exclude_side: Side)
                     -> (Vec<CellId>, Vec<State<Side>>, Vec<Vec<usize>>) {
    let mut pts = vec![];
    let mut sides = vec![];
    for i in 0..conn_map.cell_len() {
        let p = CellId::new(i);
        let a = conn_map.get(p);
        if a.coord() != p || a.side() == State::Fixed(exclude_side) {
            continue;
        }
        pts.push(p);
        sides.push(a.side());
    }

    let mut verts = vec![None; conn_map.cell_len()];
    for (i, &p) in pts.iter().enumerate() {
        verts[p.id()] = Some(i);
    }

    let graph = pts.iter()
                   .map(|&p| {
                       conn_map.get(p)
                               .unknown_edge()
                               .iter()
                               .filter_map(|p2| verts[p2.id()])
                               .collect::<Vec<_>>()
                   })
                   .collect();

    (pts, sides, graph)
}

fn get_articulation(graph: &[Vec<usize>],
                    v: usize,
                    arts: &mut Vec<usize>,
                    gvisited: &mut [bool])
                    -> Vec<bool> {
    let mut visited = vec![false; graph.len()];
    let mut ord = vec![0; graph.len()];
    let mut low = vec![0; graph.len()];
    let mut ord_cnt = 0;
    unsafe {
        dfs(graph,
            v,
            usize::MAX,
            arts,
            gvisited,
            &mut visited,
            &mut ord,
            &mut low,
            &mut ord_cnt);
    }

    return visited;

    unsafe fn dfs(graph: &[Vec<usize>],
                  v: usize,
                  prev: usize,
                  arts: &mut Vec<usize>,
                  gvisited: &mut [bool],
                  visited: &mut [bool],
                  ord: &mut [usize],
                  low: &mut [usize],
                  ord_cnt: &mut usize) {
        debug_assert!(!visited[v]);

        *visited.get_unchecked_mut(v) = true;
        *gvisited.get_unchecked_mut(v) = true;

        let ord_v = *ord_cnt;
        *ord.get_unchecked_mut(v) = ord_v;
        *low.get_unchecked_mut(v) = ord_v;
        *ord_cnt += 1;

        let mut is_articulation = false;
        let mut num_child = 0;

        for &u in graph.get_unchecked(v) {
            if u == v {
                continue;
            }

            if !*visited.get_unchecked(u) {
                dfs(graph, u, v, arts, gvisited, visited, ord, low, ord_cnt);

                let low_u = *low.get_unchecked(u);
                num_child += 1;
                *low.get_unchecked_mut(v) = cmp::min(*low.get_unchecked(v), low_u);
                if ord_v <= low_u {
                    is_articulation = true;
                }
            } else if u != prev {
                *low.get_unchecked_mut(v) = cmp::min(*low.get_unchecked(v), *ord.get_unchecked(u));
            } else {
            }
        }

        if ord_v == 0 {
            is_articulation = num_child > 1;
        }
        if is_articulation {
            arts.push(v);
        }
    }
}

fn find_disconn_area(conn_map: &mut ConnectMap,
                     pts: &[CellId],
                     visited: &[bool])
                     -> SolverResult<Vec<usize>> {
    let mut disconn = vec![];
    let mut disconn_sum = 0;
    for (u, &vis) in visited.iter().enumerate() {
        if !vis {
            disconn.push(u);
            disconn_sum += conn_map.get(pts[u]).sum_of_hint();
        }
    }
    if disconn.is_empty() {
        // All area is connected
        return Ok(vec![]);
    }
    if disconn_sum == 0 {
        // Disconnected components does not contain any edges. It is a hole in
        // the exclude_side area.
        return Ok(disconn);
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
        return Ok(conn);
    }

    // Graph is splitted into more than two parts, but both parts contain edges.
    // This may be valid in some situation, so, return empty.
    Ok(vec![])
}

fn splits(graph: &[Vec<usize>], v: usize, sides: &[State<Side>], set_side: Side) -> bool {
    let mut visited = vec![false; graph.len()];

    unsafe {
        *visited.get_unchecked_mut(v) = true;

        let mut hit = false;
        for u in 0..graph.len() {
            let side = *sides.get_unchecked(u);
            if side != State::Fixed(set_side) {
                continue;
            }
            if *visited.get_unchecked(u) {
                continue;
            }

            if hit {
                return true;
            }

            hit = true;
            visit(graph, u, &mut visited);
        }
    }

    return false;

    unsafe fn visit(graph: &[Vec<usize>], v: usize, visited: &mut [bool]) {
        *visited.get_unchecked_mut(v) = true;

        for &u in graph.get_unchecked(v) {
            if *visited.get_unchecked(u) {
                continue;
            }
            visit(graph, u, visited);
        }
    }
}

pub fn run(side_map: &mut SideMap,
           conn_map: &mut ConnectMap,
           last_rev: &mut Option<u32>)
           -> SolverResult<()> {
    try!(conn_map.sync(side_map));

    if let Some(r) = *last_rev {
        if r == side_map.revision() {
            return Ok(());
        }
    }
    *last_rev = Some(side_map.revision());

    let sides = &[(Side::In, Side::Out), (Side::Out, Side::In)];

    for &(set_side, exclude_side) in sides {
        let (pts, sides, graph) = create_conn_graph(conn_map, exclude_side);

        let mut arts = vec![];
        let mut gvisited = vec![false; graph.len()];

        #[cfg_attr(feature="dev", allow(needless_range_loop))]
        for v in 0..graph.len() {
            if gvisited[v] {
                continue;
            }

            let visited = get_articulation(&graph, v, &mut arts, &mut gvisited);

            if set_side == Side::Out || conn_map.sum_of_hint() != 0 {
                // If there is no edge in puzzle (sum_of_hint == 0) and set_side ==
                // Side::In, any disconnected area can be loop.
                let disconn = try!(find_disconn_area(conn_map, &pts, &visited));
                for v in disconn {
                    side_map.set_side(pts[v], exclude_side);
                }
            }
        }

        for v in arts {
            if sides[v] == State::Fixed(set_side) {
                continue;
            }

            if splits(&graph, v, &sides, set_side) {
                side_map.set_side(pts[v], set_side);
            }
        }
    }

    Ok(())
}
