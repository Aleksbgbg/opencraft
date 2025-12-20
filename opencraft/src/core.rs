use std::mem;

#[allow(dead_code)]
pub mod math;
pub mod type_conversions;

pub fn slice_byte_len<T>(slice: &[T]) -> usize {
  mem::size_of_val(slice)
}
