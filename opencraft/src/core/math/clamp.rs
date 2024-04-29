use std::ops::RangeToInclusive;

pub fn end(value: f32, range: RangeToInclusive<f32>) -> f32 {
  value.rem_euclid(range.end)
}
