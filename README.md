# Srither - Slither Link Solver written in Rust.

[![Build Status (Travis CI)](https://travis-ci.org/gifnksm/srither.svg?branch=master)](https://travis-ci.org/gifnksm/srither)
[![Build status (AppVeyor)](https://ci.appveyor.com/api/projects/status/lkmxu31s0pylrnhd/branch/master?svg=true)](https://ci.appveyor.com/project/gifnksm/srither/branch/master)
[![Coverage Status](https://coveralls.io/repos/gifnksm/srither/badge.svg?branch=master&service=github)](https://coveralls.io/github/gifnksm/srither?branch=master)

## Solve puzzle

Solve and output the puzzle.

```
$ cargo run --release -- solve ./puzzle/example.txt
```

## Test

Test whether all given puzzles can be solved.

```
$ cargo run --release -- test ./puzzle/**/*.txt
```

## Benchmark

Run the benchmark test that solves the 10 hardest puzzles.

```
$ cargo run --release -- bench --only-hardest 10 ./puzzle/**/*.txt
```

## Download puzzles

Downloads puzzles from [janko.at](http://www.janko.at/Raetsel/Slitherlink), [ナンバーライン問題集](http://www.pro.or.jp/~fuji/java/puzzle/numline) and [nikoli](http://www.nikoli.com/en/puzzles/slitherlink/).

```
$ ./etc/download_puzzles.sh
```

## ToDos

  * Refactoring
  * Performance improvement
  * Puzzle generator
  * GUI interface (editor, player, solver)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
