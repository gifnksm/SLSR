# Srither - Slither Link Solver written in Rust.

[![Build Status (Travis CI)](https://travis-ci.org/gifnksm/srither.svg?branch=master)](https://travis-ci.org/gifnksm/srither)
[![Build status (AppVeyor)](https://ci.appveyor.com/api/projects/status/lkmxu31s0pylrnhd/branch/master?svg=true)](https://ci.appveyor.com/project/gifnksm/srither/branch/master)
[![Coverage Status](https://coveralls.io/repos/gifnksm/srither/badge.svg?branch=master&service=github)](https://coveralls.io/github/gifnksm/srither?branch=master)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

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

Downloads puzzles from [janko.at](http://www.janko.at/Raetsel/Slitherlink) and [ナンバーライン問題集](http://www.pro.or.jp/~fuji/java/puzzle/numline).

```
$ ./etc/download_puzzles.sh
```

## ToDos

  * Refactoring
  * Performance improvement
  * Puzzle generator
  * GUI interface (editor, player, solver)
