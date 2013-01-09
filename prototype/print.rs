use io::{Writer, WriterUtil};

use std::term;

use position::{Position};
use board::{Board, Hint,
            CellType, Inside, Outside, UnknownType, ConflictType,
            CellRelation, Same, Different, UnknownRel, ConflictRel};

priv pure fn rel_to_str_v(rel: CellRelation) -> ~str {
    match (rel) {
        Same        => ~" ",
        Different   => ~"|",
        UnknownRel  => ~"/",
        ConflictRel => ~"!"
    }
}

priv pure fn rel_to_str_h(rel: CellRelation) -> ~str {
    match (rel) {
        Same        => ~" ",
        Different   => ~"-",
        UnknownRel  => ~"/",
        ConflictRel => ~"!"
    }
}

priv pure fn hint_to_str(hint: Hint) -> ~str {
    match hint {
        Some(x) => fmt!("%u", x),
        None    => ~" "
    }
}

priv fn set_color(writer: Writer, ty: CellType) {
    match ty {
        Inside  => {
            term::bg(writer, term::color_yellow);
            term::fg(writer, term::color_black);
        }
        Outside => {
            term::bg(writer, term::color_black);
            term::fg(writer, term::color_bright_white);
        }
        UnknownType => {
            term::bg(writer, term::color_bright_white);
            term::fg(writer, term::color_black);
        }
        ConflictType => {
            term::bg(writer, term::color_red);
            term::fg(writer, term::color_bright_white);
        }
    }
}

priv fn print_corner(writer: Writer, board: &mut Board, p: Position) {
    if board.is_same_all([p, p.up(), p.left(), p.up_left()]) {
        set_color(writer, board.get_cell_type(p));
    }
    writer.write_str("+");
    term::reset(writer);
}

priv fn print_edge_h(writer: Writer, board: &mut Board, p: Position) {
    if board.is_same(p, p.up()) {
        set_color(writer, board.get_cell_type(p));
    }
    let rel = board.get_cell_relation(p, p.up());
    let s = rel_to_str_h(rel);
    writer.write_str(s);
    writer.write_str(s);
    term::reset(writer);
}

priv fn print_edge_v(writer: Writer, board: &mut Board, p: Position) {
    if board.is_same(p, p.left()) {
        set_color(writer, board.get_cell_type(p));
    }
    let rel = board.get_cell_relation(p, p.left());
    let s = rel_to_str_v(rel);
    writer.write_str(s);
    term::reset(writer);
}

priv fn print_cell(writer: Writer, board: &mut Board, p: Position) {
    set_color(writer, board.get_cell_type(p));
    writer.write_str(hint_to_str(board.get_hint(p)));
    writer.write_char(' ');
    term::reset(writer);
}

priv fn print_label_row(writer: Writer, board: &mut Board) {
    writer.write_str("  ");
    for board.each_x |x| {
        writer.write_char(' ');
        writer.write_str(fmt!("%2d", x));
    }
    writer.write_char('\n');
}

priv fn print_edge_row(writer: Writer, board: &mut Board, y: int) {
    writer.write_str("  ");
    for board.each_x |x| {
        print_corner(writer, board, Position::new((x, y)));
        print_edge_h(writer, board, Position::new((x, y)));
    }
    print_corner(writer, board, Position::new((board.get_width() as int, y)));
    writer.write_char('\n');
}

priv fn print_cell_row(writer: Writer, board: &mut Board, y: int) {
    writer.write_str(fmt!("%2d", y));
    for board.each_x |x| {
        print_edge_v(writer, board, Position::new((x, y)));
        print_cell(writer, board, Position::new((x, y)));
    }
    print_edge_v(writer, board, Position::new((board.get_width() as int, y)));
    writer.write_str(fmt!("%2d\n", y));
}

pub fn print_board(writer: Writer, board: &mut Board) {
    print_label_row(writer, board);
    for board.each_y |y| {
        print_edge_row(writer, board, y);
        print_cell_row(writer, board, y);
    }
    print_edge_row(writer, board, board.get_height() as int);
    print_label_row(writer, board);
}

