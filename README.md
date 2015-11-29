# Srither - Slither Link Solver written in Rust.

[![Build Status (Travis CI)](https://travis-ci.org/gifnksm/srither.svg?branch=master)](https://travis-ci.org/gifnksm/srither)
[![Build status (AppVeyor)](https://ci.appveyor.com/api/projects/status/lkmxu31s0pylrnhd/branch/master?svg=true)](https://ci.appveyor.com/project/gifnksm/srither/branch/master)
[![Coverage Status](https://coveralls.io/repos/gifnksm/srither/badge.svg?branch=master&service=github)](https://coveralls.io/github/gifnksm/srither?branch=master)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## CLI solver

```
$ cd srither-cli
$ cargo run --release -- solve ../puzzle/example.txt
```

## Download puzzles

Downloads puzzles from [janko.at](http://www.janko.at/Raetsel/Slitherlink) and [ナンバーライン問題集](http://www.pro.or.jp/~fuji/java/puzzle/numline).

```
$ ./etc/download_puzzles.sh
```

## ToDos

  * Refactoring
  * Performance improvement
  * Problem generator
  * GUI interface (editor, player, solver)
