use std::fmt::{Display, Formatter, Result};
use thousands::Separable;

#[derive(Clone, Copy)]
pub struct Bytes(pub usize);

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    let Bytes(size) = self;
    let kilobytes = size / 1024;

    write!(f, "{}K", kilobytes.separate_with_commas())
  }
}
