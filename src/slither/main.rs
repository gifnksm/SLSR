#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]
#![allow(unstable)]

extern crate "union-find" as union_find;
extern crate term;

use std::io::stdio;
use geom::Geom;
use hint::Hint;
use board::Board;

mod board;
mod geom;
mod hint;
mod pprint;
mod solver;

fn main() {
    let hint = Hint::from_reader(stdio::stdin()).unwrap();
    let mut board = Board::new(hint.size());
    solver::solve(&mut board, &hint);
    let _ = pprint::print(&mut board, &hint);
}
