use crate::platform::Instant;
use std::time::Duration;

pub struct PollOnInterval<T, F = fn() -> T> {
  result: T,
  function: F,
  interval: Duration,
  last: Instant,
}

impl<T, F> PollOnInterval<T, F>
where
  T: Copy,
  F: Fn() -> T,
{
  pub fn new(function: F, interval: Duration) -> Self {
    Self {
      result: function(),
      function,
      interval,
      last: Instant::now(),
    }
  }

  pub fn poll(&mut self) -> T {
    if self.last.elapsed() >= self.interval {
      self.result = (self.function)();
      self.last = Instant::now();
    }

    self.result
  }
}
