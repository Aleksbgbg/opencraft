use anyhow::Result;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

fn main() -> Result<()> {
  let event_loop = EventLoop::new()?;
  let _window = WindowBuilder::new()
    .with_title("Opencraft")
    .build(&event_loop)?;

  #[allow(clippy::single_match)]
  event_loop.run(|event, target| match event {
    Event::WindowEvent { event, .. } => match event {
      WindowEvent::CloseRequested
      | WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            state: ElementState::Pressed,
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            ..
          },
        ..
      } => {
        target.exit();
      }
      _ => {}
    },
    _ => {}
  })?;

  Ok(())
}
