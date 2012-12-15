use core::pipes::{GenericChan, stream, SharedChan};
use core::task::{spawn_supervised};
use core::either::{Either, Left, Right};

extern mod std;
use std::sort::{merge_sort};

use union_find::{UnionFind, UFValue};
use position::{Position, each_rot4, Cw0Deg, Cw90Deg,
               UP, RIGHT, DOWN, LEFT,
               UP_RIGHT, DOWN_RIGHT, DOWN_LEFT, UP_LEFT};
use board::{Board, CellRelation, Same, Different, UnknownRel, ConflictRel,
            Inside, Outside, UnknownType, ConflictType};

priv fn solve_by_num_place(board: &mut Board) {
    for board.each_pos |p| {
        match board.get_hint(p) {
            Some(0) => {
                board.set_same(p, p.up());
                board.set_same(p, p.left());
                board.set_same(p, p.right());
                board.set_same(p, p.down());
            },
            Some(1) => {},
            Some(2) => {},
            Some(3) => {
                // 3 3 => |3|3|
                for [Cw0Deg, Cw90Deg].each |rot| {
                    let r = p.right_on(*rot);
                    if board.get_hint(r) == Some(3) {
                        board.set_different(p, p.left_on(*rot));
                        board.set_different(p, r);
                        board.set_different(r, r.right_on(*rot));

                        board.set_same(p.up_on(*rot),
                                       p.up_right_on(*rot));
                        board.set_same(p.down_on(*rot),
                                       p.down_right_on(*rot));
                    }
                }

                //       _           _
                // 3    |3    3     |3
                //  3 =>  3|,  2  =>  2
                //        ‾     3       3|
                //                      ‾
                for [Cw0Deg, Cw90Deg].each |rot| {
                    let mut ur = p.up_right_on(*rot);
                    while board.get_hint(ur) == Some(2) {
                        ur = ur.up_right_on(*rot);
                    }
                    if board.get_hint(ur) == Some(3) {
                        board.set_different_around_on(p, [LEFT, DOWN], *rot);
                        board.set_different_around_on(ur, [RIGHT, UP], *rot);

                        board.set_same_all([p.left_on(*rot),
                                            p.down_left_on(*rot),
                                            p.down_on(*rot)]);
                        board.set_same_all([ur.right_on(*rot),
                                            ur.up_right_on(*rot),
                                            ur.up_on(*rot)]);
                    }
                }
            },
            _       => {}
        }
    }
}

priv fn set_by_lines(board: &mut Board, p: Position) -> uint {
    let mut num_same = 0;
    let mut num_different = 0;
    let mut unknown = ~[];
    for [UP, RIGHT, DOWN, LEFT].each |d| {
        match board.get_cell_relation(p, p.shift(*d)) {
            Same        => num_same += 1,
            Different   => num_different += 1,
            UnknownRel  => unknown.push(*d),
            ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
        }
    }

    if num_different == 3 {
        board.set_same_around(p, unknown);
        num_same += unknown.len();
        unknown = ~[];
    }

    match board.get_hint(p) {
        Some(x) => {
            if num_different == x {
                board.set_same_around(p, unknown);
                num_same += unknown.len();
                unknown = ~[];
            }
            if num_same == 4 - x {
                board.set_different_around(p, unknown);
                num_different += unknown.len();
                unknown = ~[];
            }
        }
        None => {}
    }

    let mut num_unknown = unknown.len();
    for [UP_RIGHT, DOWN_RIGHT, DOWN_LEFT, UP_LEFT].each |d| {
        match board.get_cell_relation(p, p.shift(*d)) {
            Same        => num_same += 1,
            Different   => num_different += 1,
            UnknownRel  => num_unknown += 1,
            ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
        }
    }
    return num_unknown;
}

priv fn set_by_area(board: &mut Board, p: Position) {
    for each_rot4 |rot| {
        let u = p.up_on(rot);
        let r = p.right_on(rot);
        let d = p.down_on(rot);
        let l = p.left_on(rot);
        let ur = p.up_right_on(rot);
        let ul = p.up_left_on(rot);

        if board.is_different(p, u) {
            if board.is_different(p, r) {
                board.set_different(p, ur);
            }
            if board.is_different(r, ur) {
                board.set_same(p, r);
            }
        }

        match board.get_cell_relation(u, r) {
            Same => {
                match board.get_hint(p) {
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
            Different => {
                match board.get_hint(p) {
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
            UnknownRel => {}
            ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
        }

        match board.get_cell_relation(u, ur) {
            Different => {
                match board.get_hint(p) {
                    Some(3) => {
                        board.set_different(p, d);
                        board.set_different(p, l);
                        board.set_same(ur, r);
                    }
                    _ => {}
                }
            }
            Same | UnknownRel => {}
            ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
        }

        match board.get_cell_relation(u, ul) {
            Different => {
                match board.get_hint(p) {
                    Some(3) => {
                        board.set_different(p, d);
                        board.set_different(p, r);
                        board.set_same(ul, l);
                    }
                    _ => {}
                }
            }
            Same | UnknownRel => {}
            ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
        }
    }

    for [Cw0Deg, Cw90Deg].each |rot| {
        let u = p.up_on(*rot);
        let r = p.right_on(*rot);
        let d = p.down_on(*rot);
        let l = p.left_on(*rot);
        if board.is_same(u, d) {
            match board.get_hint(p) {
                Some(1) => {
                    board.set_same(p, u);
                    board.set_different(l, r);
                }
                Some(2) => {
                    board.set_same(l, r);
                    board.set_different(u, l);
                }
                Some(3) => {
                    board.set_different(p, u);
                    board.set_different(l, r);
                }
                _ => {}
            }
        }
        if board.is_different(u, d) {
            match board.get_hint(p) {
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
    }    
}

struct AreaValue {
    coord: Position,
    unknown_rel: ~[Position],
    sum_of_hint: uint,
    size: uint
}

enum Node {
    Ref(Position),
    Value(AreaValue)
}

priv fn solve_by_area_connect(board: &mut Board) -> ~[~[Node]] {
    let mut area = do vec::from_fn(board.get_height()) |y| {
        do vec::from_fn(board.get_width()) |x| {
            let p = Position::new((x as int, y as int));
            let sum = match board.get_hint(p) {
                Some(x) => x,
                None    => 0
            };

            let mut unknown_rel = ~[];
            for p.each_around4 |r| {
                if board.get_cell_relation(p, r) == UnknownRel {
                    unknown_rel += [r];
                }
            }
            Value(AreaValue {
                coord: p,
                unknown_rel: move unknown_rel,
                sum_of_hint: sum,
                size: 1 })
        }
    };

    let mut outside_p = None;
    for board.each_pos |p1| {
        for [p1.up(), p1.left()].each |&p2| {
            if board.is_same(p1, p2) {
                union_area_outside(area, p1, p2, board, &mut outside_p);
            }
        }

        let p2 = p1.right();
        if !board.contains(p2) && board.is_same(p1, p2) {
            union_area_outside(area, p1, p2, board, &mut outside_p);
        }
        let p2 = p1.down();
        if !board.contains(p2) && board.is_same(p1, p2) {
            union_area_outside(area, p1, p2, board, &mut outside_p);
        }
    }

    match outside_p {
        Some(pt) => {
            let pt = find_area(area, pt);
            match area[pt.y][pt.x] {
                Value(ref mut v) => {
                    for board.each_x |x| {
                        for [(x, 0), (x, board.get_height() as int - 1)].each |&t| {
                            let p = Position::new(t);
                            if board.get_cell_relation(pt, p) == UnknownRel {
                                v.unknown_rel += [p];
                            }
                        }
                    }
                    for board.each_y |y| {
                        for [(-1, y), (board.get_width() as int - 1, y)].each |&t| {
                            let p = Position::new(t);
                            if board.get_cell_relation(pt, p) == UnknownRel {
                                v.unknown_rel += [p];
                            }
                        }
                    }
                }
                _ => fail
            }
        },
        None => {}
    }

    let mut seq = board.get_seq();
    loop {
        for board.each_pos |p| {
            let p = find_area(area, p);
            update_area(area, p, board, outside_p);

            match find_union_area(area, p) {
                Some(p1) => {
                    union_area(area, p, p1);
                    board.set_same(p, p1);
                }
                None => {}
            }
        }

        if board.get_seq() == seq { return area; }
        seq = board.get_seq();
    }

    priv pure fn find_area(area: &const[const ~[Node]], p: Position) -> Position {
        match area[p.y][p.x] {
            Ref(pos) => find_area(area, pos),
            Value(_) => p
        }
    }

    priv fn union_area_outside(area: &mut[~[Node]],
                               p1: Position, p2: Position,
                               board: &mut Board,
                               outside_p: &mut Option<Position>) {
        if board.contains(p2) {
            union_area(area, p1, p2);
        } else {
            match outside_p {
                &None => *outside_p = Some(p1),
                &Some(copy pt) => union_area(area, p1, pt)
            }
        }
    }

    priv fn union_area(area: &mut[~[Node]], p1: Position, p2: Position) {
        let mut p1 = find_area(area, p1);
        let mut p2 = find_area(area, p2);
        if p1 == p2 { return; }

        match (area[p1.y][p1.x], area[p2.y][p2.x]) {
            (Value(ref v1), Value(ref v2)) => {
                area[p1.y][p1.x] = Value(AreaValue {
                    coord: p1,
                    unknown_rel: v1.unknown_rel + v2.unknown_rel,
                    sum_of_hint: v1.sum_of_hint + v2.sum_of_hint,
                    size: v1.size + v2.size
                });
                area[p2.y][p2.x] = Ref(p1);
            }
            _ => fail
        }
    }

    priv fn update_area(area: &mut[~[Node]], p: Position,
                        board: &mut Board, outside_p: Option<Position>) {
        let mut (rels, sum_of_hint, size) = match copy area[p.y][p.x] {
            Value(v) => {
                if v.unknown_rel.len() == 0 { return; }
                let rels = merge_sort(
                    do v.unknown_rel.map |&rp| {
                        if board.contains(rp) {
                            find_area(area, rp)
                        } else {
                            match outside_p {
                                Some(pt) => pt,
                                None => Position::new((-1,-1))
                            }
                        }
                    },
                    |p1, p2| p1 <= p2);
                (rels, v.sum_of_hint, v.size)
            },
            _ => return
        };

        let mut union = ~[];
        let mut i_next = 0;
        for uint::range(0, rels.len()) |i_src| {
            if i_next > 0 && rels[i_next - 1] == rels[i_src] { loop; }
            match board.get_cell_relation(p, rels[i_src]) {
                Same => {
                    union += [ rels[i_src] ];
                    loop;
                }
                UnknownRel => {},
                Different => loop,
                ConflictRel => fail fmt!("conflict (seq: %u)", board.get_seq())
            }
            rels[i_next] = rels[i_src];
            i_next += 1;
        }
        rels.truncate(i_next);

        area[p.y][p.x] = Value(AreaValue {
            coord: p,
            unknown_rel: move rels,
            sum_of_hint: sum_of_hint,
            size: size
        });

        for union.each |&p2| { union_area(area, p, p2); }
        if union.is_not_empty() { update_area(area, p, board, outside_p); }
        return;
    }

    priv fn find_union_area(area: &[~[Node]], p: Position)
        -> Option<Position> {
        let p = find_area(area, p);
        match copy area[p.y][p.x] {
            Value(v) => {
                if v.unknown_rel.len() != 1 { return None; }
                return Some(v.unknown_rel[0]);
            },
            _ => fail
        }
    }
}

priv fn solve_by_logic(board: &mut Board) -> ~[~[Node]] {
    let mut area;
    solve_by_num_place(board);

    loop {
        let seq = board.get_seq();
        for board.each_pos |p| {
            if set_by_lines(board, p) == 0 { loop; }
            set_by_area(board, p);
        }
        if board.get_seq() != seq { loop; }

        area = solve_by_area_connect(board);
        if board.get_seq() == seq { break; }
    }

    return area;
}

pub fn solve<T: GenericChan<~Board>>(chan: &T, board: ~Board) {
    let mut board = board;
    let area = solve_by_logic(board);

    let mut inside_area = ~[];
    let mut outside_area = ~[];
    let mut unknown_area = ~[];

    for board.each_pos |p| {
        match area[p.y][p.x] {
            Value(v) => {
                match board.get_cell_type(v.coord) {
                    Inside => inside_area.push(v),
                    Outside => outside_area.push(v),
                    UnknownType => unknown_area.push(v),
                    ConflictType => fail fmt!("conflict (seq: %u)", board.get_seq())
                }
            },
            _ => {}
        }
    }

    if unknown_area.len() == 0 {
        if inside_area.len() != 1 || outside_area.len() != 1  ||
            inside_area[0].sum_of_hint + outside_area[0].sum_of_hint
            != board.get_sum_of_hint() {
            fail fmt!("splited (seq: %u)", board.get_seq())
        }
        chan.send(board);
        return;
    }

    if !inside_area.all(|a| a.unknown_rel.len() > 0) ||
        !outside_area.all(|a| a.unknown_rel.len() > 0)
    {
        fail fmt!("splited (seq: %u)", board.get_seq())
    }

    let mut max_i = 0;
    for unknown_area.eachi |i, v| {
        let max_v = &unknown_area[max_i];
        if v.size > max_v.size ||
            (v.size == max_v.size &&
             v.unknown_rel.len() > max_v.unknown_rel.len()) {
            max_i = i;
        }
    }

    let (port, child_chan) = stream();
    {
        let coord = unknown_area[max_i].coord;
        let child_chan = SharedChan(move child_chan);
        for uint::range(0, 2) |i| {
            let child_chan = child_chan.clone();
            let input = board.clone();
            do spawn_supervised |move child_chan, move input| {
                let mut input = input;
                if i == 0 {
                    input.set_inside(coord);
                } else {
                    input.set_outside(coord);
                }
                solve::<SharedChan<~Board>>(&child_chan, ~input);
            }
        }
    }

    loop {
        match port.try_recv() {
            Some(answer) => chan.send(answer),
            None => break
        }
    }
}

