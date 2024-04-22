use crate::core::math::angle::Radians;
use std::f32::consts::PI;
use std::ops::RangeToInclusive;

fn clamp_end(value: f32, range: RangeToInclusive<f32>) -> f32 {
  value.rem_euclid(range.end)
}

pub fn rotation(angle: Radians) -> Radians {
  Radians::new(clamp_end(angle.value(), ..=(2.0 * PI)))
}
