#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(duration_millis_float)]

mod camera;
mod core;
mod game;
mod platform;

use crate::game::Game;
use crate::platform::error;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

pub fn start() -> Result<()> {
  platform::init_logging();

  let event_loop = EventLoop::with_user_event().build()?;

  let mut app = App::new(event_loop.create_proxy());
  event_loop.run_app(&mut app)?;

  Ok(())
}

struct AppState {
  window: Arc<Window>,
  game: Game,
}

enum AppEvent {
  SpinWaitWindowInit(Arc<Window>),
  Init(Box<AppState>),
}

struct App {
  state: Option<AppState>,
  event_loop_proxy: EventLoopProxy<AppEvent>,
}

impl App {
  fn new(event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
    App {
      state: None,
      event_loop_proxy,
    }
  }

  fn is_ready(&self) -> bool {
    self.state.is_some()
  }

  fn unwrap(&mut self) -> (&Window, &mut Game) {
    let state = self.state.as_mut().unwrap();
    (&state.window, &mut state.game)
  }
}

impl ApplicationHandler<AppEvent> for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      platform::init_window_attributes(Window::default_attributes().with_title("Opencraft"));

    let window = Arc::new(
      event_loop
        .create_window(window_attributes)
        .expect("could not create window"),
    );
    let _ = window.set_cursor_grab(CursorGrabMode::Confined);
    window.set_cursor_visible(false);

    verify_send_event(
      self
        .event_loop_proxy
        .send_event(AppEvent::SpinWaitWindowInit(window)),
    );
  }

  fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
    match event {
      AppEvent::SpinWaitWindowInit(window) => {
        let event_loop_proxy = self.event_loop_proxy.clone();

        if is_valid_window(&window) {
          platform::run_future(async move {
            let game = Game::new(Arc::clone(&window))
              .await
              .expect("could not initialise game");

            verify_send_event(
              event_loop_proxy.send_event(AppEvent::Init(Box::new(AppState { window, game }))),
            );
          });
        } else {
          platform::run_future(async move {
            platform::sleep(Duration::from_millis(100)).await;

            verify_send_event(event_loop_proxy.send_event(AppEvent::SpinWaitWindowInit(window)));
          });
        }
      }
      AppEvent::Init(app_state) => self.state = Some(*app_state),
    };
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    if !self.is_ready() {
      return;
    }

    let (window, game) = self.unwrap();

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::RedrawRequested => {
        if let Err(err) = game.compose() {
          error!("Error during composition loop: {:?}", err);
          event_loop.exit();
        }
      }
      WindowEvent::Resized(physical_size) => {
        game.resize(physical_size);
      }
      WindowEvent::ScaleFactorChanged { .. } => {
        game.resize(window.inner_size());
      }
      WindowEvent::KeyboardInput {
        event: KeyEvent {
          state,
          physical_key,
          ..
        },
        ..
      } => match state {
        ElementState::Pressed => {
          if let PhysicalKey::Code(code) = physical_key {
            #[allow(clippy::single_match)]
            match code {
              KeyCode::Escape => event_loop.exit(),
              _ => {}
            }

            game.press(code);
          }
        }
        ElementState::Released => {
          if let PhysicalKey::Code(code) = physical_key {
            game.release(code);
          }
        }
      },
      _ => {}
    };
  }

  fn device_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    _device_id: DeviceId,
    event: DeviceEvent,
  ) {
    if !self.is_ready() {
      return;
    }

    let (_, game) = self.unwrap();

    #[allow(clippy::single_match)]
    match event {
      DeviceEvent::MouseMotion { delta: (x, y) } => {
        game.motion(x as f32, y as f32);
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if !self.is_ready() {
      return;
    }

    let (window, _) = self.unwrap();

    window.request_redraw();
  }
}

fn is_valid_window(window: &Window) -> bool {
  (window.inner_size().width > 0) && (window.inner_size().height > 0)
}

fn verify_send_event(result: Result<(), EventLoopClosed<AppEvent>>) {
  if result.is_err() {
    panic!("event loop closed");
  }
}
