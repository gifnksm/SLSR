// a & a: same side, a & A: Different side
pub const THEOREM_DEFINE: &'static [&'static str] = &["
+ + ! +x+
 0  ! x0x
+ + ! +x+
", "
+ + + + ! + + + +
        !   | x
+ + + + ! +x+-+x+
 0 3    ! x0x3|
+ + + + ! +x+-+x+
        !   | x
+ + + + ! + + + +
", "
+ + + + ! +x+ + +
 0      ! x0x
+ + + + ! +x+-+ +
   3    !   |3 a
+ + + + ! + + + +
        !    A
+ + + + ! + + + +
", "
+ + + ! + + +
      !   xa
+ + + ! + + +
 3 3  ! |3|3|
+ + + ! + + +
      !   xA
+ + + ! + + +
", "
+ + + ! +-+ +
 2    !  2xa
+ + + ! + + +
 3 3  ! |3|3|
+ + + ! + + +
      !   xA
+ + + ! + + +
", "
+ + + + + ! + + + + +
          !   x
+ + + + + ! +x+-+ + +
   3      !   |3 a
+ + + + + ! + + + + +
     3    !    A 3|
+ + + + + ! + + +-+x+
          !       x
+ + + + + ! + + + + +
", "
+ + + + + + ! + + + + + +
            !   x     x
+ + + + + + ! +x+-+x+-+x+
   3   3    !   |3 a 3|
+ + + + + + ! + + + + + +
     3      !    A|3|A
+ + + + + + ! + +x+-+x+ +
            !     x x
+ + + + + + ! + + + + + +
", "
+ + + + ! + +x+ +
  | |   !   | |
+ +-+ + ! +x+-+x+
        !   x x
+ + + + ! + + + +
", "
+-+ ! +-+
 1  ! x1x
+ + ! +x+
", "
+ + ! +-+
x1x ! x1x
+x+ ! +x+
", "
+ + + ! +x+ +
 2|   ! x2|
+-+ + ! +-+x+
      !   x
+ + + ! + + +
", "
+ + ! +x+
|2| ! |2|
+ + ! +x+
", "
+x+ + ! +x+ +
x2    ! x2|
+ + + ! +-+x+
      !   x
+ + + ! + + +
", "
+x+ ! +x+
 2  ! |2|
+x+ ! +x+
", "
+ + + + ! + +x+ +
  |3|   !   |3|
+ +-+ + ! +x+-+x+
        !   x x
+ + + + ! + + + +
", "
+ +x+ + ! + +x+ +
   3    !   |3|
+ + + + ! +x+-+x+
        !   x x
+ + + + ! + + + +
", "
+ + + ! + + +
      !   x
+-+ + ! +-+x+
  |   !   |
+ + + ! + + +
", "
+ + + ! + + +
      !   x
+-+-+ ! +-+-+
      !   x
+ + + ! + + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + +x+ +
 a 1    !  ax1 b
+ + + + ! + + + +
        !    B
+ + + + ! + + + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
 a 2    !  a 2 A
+ + + + ! + + + +
        !    A
+ + + + ! + + + +
", "
+ + + + ! + + + +
   a    !   xa
+ + + + ! +x+-+ +
 a 3    !  a|3 b
+ + + + ! + + + +
        !    B
+ + + + ! + + + +
", "
+ + + ! + + +
   a  !    a
+ + + ! + + +
 A 1  !  A 1x
+ + + ! + +x+
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
 A 2    !  A 2 b
+ + + + ! + + + +
        !    B
+ + + + ! + + + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
 A 3    !  A 3|
+ + + + ! + +-+x+
        !     x
+ + + + ! + + + +
", "
+ + + ! + + +
  x   !   x
+ + + ! +-+ +
 3 1  !  3 1x
+ + + ! + +x+
", "
+ + + + + ! + + +-+ +
     2x   !    a 2x
+ + + + + ! + + + + +
   3      !   |3 A
+ + + + + ! +x+-+ + +
          !   x
+ + + + + ! + + + + +
", "
+ + + + ! + +-+ +
   2x   !    2x
+ + +-+ ! + + +-+
   3    !   |3
+ + + + ! +x+-+ +
        !   x
+ + + + ! + + + +
", "
+ + + ! + + +
    | !     |
+ + + ! + + +
   3  !   |3
+ + + ! +x+-+
      !   x
+ + + ! + + +
", "
+ +  + + ! + +  + +
      a  !       a
+ +  + + ! + +  + +
   3a    !   |3a
+ +  + + ! +x+--+ +
         !   x
+ +  + + ! + +  + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + +x+ +
   1    !  b 1 B
+ + + + ! + +x+ +
   a    !    a
+ + + + ! + + + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
   2    !  A 2 A
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + +-+ +
   3    !  b 3 B
+ + + + ! + +-+ +
   a    !    a
+ + + + ! + + + +
", "
+ + ! + +
 a  !  a
+ + ! + +
 1  ! x1x
+ + ! + +
 A  !  A
+ + ! + +
", "
+ + + + ! + + + +
   a    !    a
+ + + + ! + + + +
   2    !  b 2 B
+ + + + ! + + + +
   A    !    A
+ + + + ! + + + +
", "
+ + ! + +
 a  !  a
+ + ! + +
 3  ! |3|
+ + ! + +
 A  !  A
+ + ! + +
"];

#[cfg(test)]
mod tests {
    use solver::theorem::Theorem;

    #[test]
    fn parse() {
        for s in super::THEOREM_DEFINE {
            if !s.parse::<Theorem>().is_ok() {
                println!("{:?}", s.parse::<Theorem>());
                println!("{}", s);
            }
            assert!(s.parse::<Theorem>().is_ok());
        }
    }
}
