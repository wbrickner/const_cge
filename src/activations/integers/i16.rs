pub const fn linear(x: i16)        -> i16 { x }
pub       fn threshold(x: i16)     -> i16 {
  // twos complement, leading bit is sign (negative = 0b1, positive = 0b0)
  // sign extend through bit shift, negate, bitwise AND.
  // const SHIFT_AMOUNT: usize = (core::mem::size_of::<i16>() * 8) - 1;

  // !(x >> SHIFT_AMOUNT)
  if x > 0 { 1 } else { 0 }
}
// pub       fn relu(x: i16)          -> i16 { if x > 0.0 { x } else { 0.0 } } 
// pub       fn sign(x: i16)          -> i16 { if x > 0.0 { 1.0 } else if x == 0.0 { 0.0 } else { -1.0 } }