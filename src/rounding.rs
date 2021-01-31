pub fn round_huf(n: i32) -> i32 {
  let a = n.abs();
  (match n < 0 {
    true => -1,
    false => 1,
  }) * (match a % 10 {
    x if x == 1 || x == 2 => a - x,
    x if x == 3 || x == 4 => a + (5 - x),
    x if x == 6 || x == 7 => a - (x - 5),
    x if x == 8 || x == 9 => a + (10 - x),
    _ => a,
  })
}
