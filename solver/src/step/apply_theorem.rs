use std::mem;
use slsr_core::geom::{Geom, Point, Move};

use ::SolverResult;
use ::model::side_map::SideMap;
use ::model::theorem::{Pattern, Theorem, TheoremMatcher, TheoremMatchResult};

#[derive(Clone, Debug)]
pub struct TheoremPool {
    data: Vec<TheoremMatcher>
}

impl TheoremPool {
    pub fn new<'a, T>(theo_defs: T, side_map: &mut SideMap) -> SolverResult<TheoremPool>
        where T: IntoIterator<Item=Theorem>
    {
        let it = theo_defs
            .into_iter()
            .flat_map(|theo| theo.all_rotations());

        let mut hint_theorem = [vec![], vec![], vec![], vec![]];
        let mut nonhint_theorem = vec![];

        for theo in it {
            match theo.head() {
                Pattern::Hint(x, _) => hint_theorem[x as usize].push(theo),
                _ => nonhint_theorem.push(theo)
            }
        }

        let mut data = vec![];

        for r in 0..side_map.row() {
            for c in 0..side_map.column() {
                let p = Point(r, c);
                if let Some(x) = side_map.hint()[p] {
                    for theo in &hint_theorem[x as usize] {
                        let o = match theo.head() {
                            Pattern::Hint(_, p) => p,
                            _ => panic!()
                        };

                        match try!(theo.clone().shift(p - o).into_matcher().matches(side_map)) {
                            TheoremMatchResult::Complete(result) => {
                                for pat in &result {
                                    pat.apply(side_map);
                                }
                            }
                            TheoremMatchResult::Partial(theo) => {
                                data.push(theo);
                            }
                            TheoremMatchResult::Conflict => {}
                        }
                    }
                }
            }
        }

        for theo in nonhint_theorem {
            let sz = theo.size();
            for r in (1 - sz.0 .. side_map.row() + sz.0 - 1) {
                for c in (1 - sz.1 .. side_map.column() + sz.1 - 1) {
                    let mv = Move(r, c);
                    match try!(theo.clone().shift(mv).into_matcher().matches(side_map)) {
                        TheoremMatchResult::Complete(result) => {
                            for pat in &result {
                                pat.apply(side_map);
                            }
                        }
                        TheoremMatchResult::Partial(theo) => {
                            data.push(theo);
                        }
                        TheoremMatchResult::Conflict => {}
                    }
                }
            }
        }

        // FIXME: Should reduce the elements that has different result but matcher are same.
        data.sort();
        data.dedup();

        Ok(TheoremPool { data: data })
    }

    pub fn apply_all(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        let mut rev = side_map.revision();
        loop {
            let new_theorem = Vec::with_capacity(self.data.len());

            for theo in mem::replace(&mut self.data, new_theorem) {
                match try!(theo.matches(side_map)) {
                    TheoremMatchResult::Complete(result) => {
                        for pat in &result {
                            pat.apply(side_map);
                        }
                    }
                    TheoremMatchResult::Partial(theo) => {
                        self.data.push(theo)
                    }
                    TheoremMatchResult::Conflict => {}
                }
            }

            if side_map.revision() == rev {
                break;
            }
            rev = side_map.revision()
        }

        Ok(())
    }
}
