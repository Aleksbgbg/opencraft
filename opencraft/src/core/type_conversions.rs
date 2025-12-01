use wgpu::BufferAddress;

pub trait Coerce<T> {
  fn coerce(self) -> T;
}

pub trait CoerceLossy<T> {
  fn coerce_lossy(self) -> T;
}

pub trait CoerceLossyRound<T> {
  fn coerce_lossy_round(self) -> T;
}

pub trait CoerceLossyFloor<T> {
  fn coerce_lossy_floor(self) -> T;
}

pub trait CoerceLossyCeil<T> {
  fn coerce_lossy_ceil(self) -> T;
}

macro_rules! coerce {
  ($dst:ty, $src:ty) => {
    impl Coerce<$dst> for $src {
      fn coerce(self) -> $dst {
        self.try_into().unwrap()
      }
    }
  };
}

macro_rules! coerce_lossy {
  ($dst:ty, $src:ty) => {
    impl CoerceLossy<$dst> for $src {
      fn coerce_lossy(self) -> $dst {
        self as $dst
      }
    }
  };
}

macro_rules! coerce_lossy_round {
  ($dst:ty, $src:ty) => {
    impl CoerceLossyRound<$dst> for $src {
      fn coerce_lossy_round(self) -> $dst {
        self.round() as $dst
      }
    }
  };
}

macro_rules! coerce_lossy_floor {
  ($dst:ty, $src:ty) => {
    impl CoerceLossyFloor<$dst> for $src {
      fn coerce_lossy_floor(self) -> $dst {
        self.floor() as $dst
      }
    }
  };
}

macro_rules! coerce_lossy_ceil {
  ($dst:ty, $src:ty) => {
    impl CoerceLossyCeil<$dst> for $src {
      fn coerce_lossy_ceil(self) -> $dst {
        self.ceil() as $dst
      }
    }
  };
}

coerce!(u32, usize);
coerce!(i32, usize);
coerce_lossy!(f32, usize);
coerce!(BufferAddress, usize);

coerce!(usize, u32);
coerce!(i32, u32);
coerce_lossy!(f32, u32);

coerce!(usize, i32);
coerce_lossy!(f32, i32);

coerce_lossy!(f32, f64);

coerce_lossy_round!(usize, f32);
coerce_lossy_floor!(usize, f32);
coerce_lossy_ceil!(usize, f32);
coerce_lossy_floor!(u32, f32);
coerce_lossy_ceil!(u32, f32);
coerce_lossy_floor!(i32, f32);
coerce_lossy_round!(u8, f32);
