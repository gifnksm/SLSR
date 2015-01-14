#![warn(bad_style,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, unused_typecasts)]
#![allow(unstable)]

extern crate "union-find" as union_find;
extern crate term;

use std::io::stdio;
use hint::Hint;

mod geom;
mod hint;
mod pprint;
mod side_map;
mod solver;

fn main() {
    let mut hint = Hint::from_reader(stdio::stdin()).unwrap();
    solver::solve(&mut hint);
    let _ = pprint::print(&hint);
}
