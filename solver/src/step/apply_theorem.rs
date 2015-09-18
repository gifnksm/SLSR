use std::mem;
use slsr_core::geom::{Geom, Point, Move};

use ::SolverResult;
use ::model::side_map::SideMap;
use ::model::theorem::{Pattern, TheoremMatch, Theorem};
use ::theorem_define::THEOREM_DEFINE;

pub fn create_theorem_list(side_map: &mut SideMap) -> SolverResult<Vec<Theorem>>
{
    let it = THEOREM_DEFINE.iter()
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

    let mut theorem = vec![];

    for r in 0 .. side_map.row() {
        for c in 0 .. side_map.column() {
            let p = Point(r, c);
            match side_map.hint()[p] {
                Some(x) => {
                    for theo in &hint_theorem[x as usize] {
                        let o = match theo.head() {
                            Pattern::Hint(_, p) => p,
                            _ => panic!()
                        };

                        match try!(theo.clone().shift(p - o).matches(side_map)) {
                            TheoremMatch::Complete(result) => {
                                for pat in &result {
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

    for theo in nonhint_theorem {
        let sz = theo.size();
        for r in (1 - sz.0 .. side_map.row() + sz.0 - 1) {
            for c in (1 - sz.1 .. side_map.column() + sz.1 - 1) {
                match try!(theo.clone().shift(Move(r, c)).matches(side_map)) {
                    TheoremMatch::Complete(result) => {
                        for pat in &result {
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

    theorem.sort();
    theorem.dedup();

    Ok(theorem)
}

pub fn run(side_map: &mut SideMap, theorem: &mut Vec<Theorem>)
           -> SolverResult<()>
{
    let mut rev = side_map.revision();
    loop {
        let new_theorem = Vec::with_capacity(theorem.len());

        for theo in mem::replace(theorem, new_theorem) {
            match try!(theo.matches(side_map)) {
                TheoremMatch::Complete(result) => {
                    for pat in &result {
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
