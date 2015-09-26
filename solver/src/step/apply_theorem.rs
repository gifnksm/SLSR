use std::mem;
use slsr_core::geom::{Geom, Point, Move};

use ::SolverResult;
use ::model::side_map::SideMap;
use ::model::theorem::{Match, Pattern, Theorem, TheoremMatcher, TheoremMatchResult};

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
                Pattern::Hint(h) => hint_theorem[h.hint() as usize].push(theo),
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
                            Pattern::Hint(hint) => hint.point(),
                            _ => panic!()
                        };
                        let matcher = theo.clone().shift(p - o);
                        try!(Self::matches(matcher, side_map, &mut data));
                    }
                }
            }
        }

        for theo in nonhint_theorem {
            let sz = theo.size();
            for r in (1 - sz.0)..(side_map.row() + sz.0 - 1) {
                for c in (1 - sz.1)..(side_map.column() + sz.1 - 1) {
                    let matcher = theo.clone().shift(Move(r, c));
                    try!(Self::matches(matcher, side_map, &mut data));
                }
            }
        }

        let mut pool = TheoremPool { data: data };
        pool.merge_dup();

        Ok(pool)
    }

    pub fn apply_all(&mut self, side_map: &mut SideMap) -> SolverResult<()> {
        let cap = self.data.len();
        for matcher in mem::replace(&mut self.data, Vec::with_capacity(cap)) {
            try!(Self::matches(matcher, side_map, &mut self.data));
        }

        Ok(())
    }

    fn matches<M>(matcher: M,
                  side_map: &mut SideMap,
                  new_matchers: &mut Vec<TheoremMatcher>
                  ) -> SolverResult<()>
        where M: Match
    {
        match try!(matcher.matches(side_map)) {
            TheoremMatchResult::Complete(result) => {
                for pat in &result {
                    pat.apply(side_map);
                }
            }
            TheoremMatchResult::Partial(theo) => {
                new_matchers.push(theo)
            }
            TheoremMatchResult::Conflict => {}
        }

        Ok(())
    }

    fn merge_dup(&mut self) {
        self.data.sort();
        // Merge elements that have same matchers.
        unsafe {
            let len = self.data.len();
            let p = self.data.as_mut_ptr();

            let mut w = 1;
            for r in 1..len {
                let read = p.offset(r as isize);
                let cmp = p.offset((w - 1) as isize);

                match (*cmp).merge(&*read) {
                    Ok(()) => {}
                    Err(()) => {
                        let write = cmp.offset(1);
                        mem::swap(&mut *write, &mut *read);
                        w += 1;
                    }
                }
            }

            self.data.truncate(w);
        }
    }
}
