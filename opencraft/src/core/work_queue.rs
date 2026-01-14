use crate::platform;
use crate::platform::thread;
use arrayvec::ArrayVec;
use flume::{Receiver, Selector, Sender};
use num_enum::IntoPrimitive;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

type Task = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Copy, EnumCount, EnumIter, IntoPrimitive)]
#[repr(usize)]
pub enum Priority {
  High,
  Low,
}

pub struct WorkQueue {
  send_task_with_priority: ArrayVec<Sender<Task>, { Priority::COUNT }>,
}

impl WorkQueue {
  pub fn new() -> Self {
    let mut send_task_with_priority = ArrayVec::new();
    let mut recv_task_with_priority: ArrayVec<Receiver<Task>, { Priority::COUNT }> =
      ArrayVec::new();

    for _ in Priority::iter() {
      let (send_task, recv_task) = flume::unbounded();

      send_task_with_priority.push(send_task);
      recv_task_with_priority.push(recv_task);
    }

    for _ in 0..platform::num_cpus() {
      let recv_task_with_priority = recv_task_with_priority.clone();
      thread::spawn(move || {
        loop {
          let mut selector = Selector::new();
          for recv_task in &recv_task_with_priority {
            selector = selector.recv(recv_task, |result| {
              result.map(|task| {
                task();
              })
            });
          }

          if selector.wait().is_err() {
            break;
          }
        }
      });
    }

    Self {
      send_task_with_priority,
    }
  }

  pub fn schedule(&self, priority: Priority, task: impl FnOnce() + Send + 'static) {
    let priority_level: usize = priority.into();
    self.send_task_with_priority[priority_level]
      .send(Box::new(task))
      .unwrap();
  }
}

pub struct Channel<T> {
  pub send: Sender<T>,
  pub recv: Receiver<T>,
}

impl<T> Channel<T> {
  pub fn unbounded() -> Self {
    let (send, recv) = flume::unbounded();

    Self { send, recv }
  }

  pub fn drain(&self) -> impl Iterator<Item = T> {
    self.recv.drain()
  }
}
