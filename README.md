# SLSR

[![Build Status (Travis CI)](https://travis-ci.org/gifnksm/SLSR.png?branch=master)](https://travis-ci.org/gifnksm/SLSR)
[![Build status (AppVeyor)](https://ci.appveyor.com/api/projects/status/p0nudt9624xhcefo?svg=true)](https://ci.appveyor.com/project/gifnksm/slsr)
[![Coverage Status](https://coveralls.io/repos/gifnksm/SLSR/badge.svg?branch=master&service=github)](https://coveralls.io/github/gifnksm/SLSR?branch=master)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Slither Link Solver written in Rust.

## CLI solver

```
$ cd cli
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
