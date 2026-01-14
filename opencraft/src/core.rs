use std::mem;

#[allow(dead_code)]
pub mod math;
pub mod poll_on_interval;
pub mod type_conversions;
#[allow(dead_code)]
pub mod work_queue;

pub fn slice_byte_len<T>(slice: &[T]) -> usize {
  mem::size_of_val(slice)
}
