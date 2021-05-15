use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
  dpi::LogicalSize,
  event::{Event, DeviceEvent, ElementState, KeyboardInput, ScanCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
};
use std::{mem, thread, time};
const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 600;
const DISPLAY_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;


struct CompositorState {
  front_buffer: Pixels,
  back_buffer: Pixels,
  parity: usize // DEBUG only
}

impl CompositorState {
  pub fn swap(self: &mut Self) {
    mem::swap(&mut self.front_buffer, &mut self.back_buffer);
    self.parity = 1-self.parity;
  }
}


fn create_window(
  title: &str, w: u32, h: u32,  scale: f64, event_loop: &EventLoop<()>
) -> winit::window::Window {
  let window = winit::window::WindowBuilder::new()
    .with_title(title)
    .with_resizable(false)
    .build(&event_loop)
    .unwrap();
  let size = LogicalSize::new((w as f64) * scale, (h as f64) * scale);
  window.set_inner_size(size);
  return window;
}

fn compositor_step(state: &mut CompositorState) -> bool {
  let frame = state.back_buffer.get_frame();
  for pix in frame.chunks_exact_mut(4) {
    let color : [u8; 4] = [0x99, 0x99, 0x99, 0xff];
    pix.copy_from_slice(&color);
  }

  // DEBUG front vs back
  for i in DISPLAY_HEIGHT-16..DISPLAY_HEIGHT-4 {
    let color: [u8; 4] = if state.parity == 0 {
      [0xff, 0x00, 0x00, 0xff]
    } else {
      [0x00, 0x00, 0xff, 0xff]
    };
    for j in DISPLAY_WIDTH-16..DISPLAY_WIDTH-4 {
      let ind = 4*(i*DISPLAY_WIDTH + j);
      let pix = frame.get_mut(ind..ind+4).unwrap();
      pix.copy_from_slice(&color);
    }
  }
  
  // FORNOW
  thread::sleep(time::Duration::from_millis(100));
  return true;
}

fn main() -> Result<(), Error> {
  let event_loop = EventLoop::new();
  const WINDOW_WIDTH: u32 = DISPLAY_WIDTH as u32;
  const WINDOW_HEIGHT: u32 = DISPLAY_HEIGHT as u32;
  let scale: f64 = 1.0;
  let window = create_window("Compositor Demo", WINDOW_WIDTH, WINDOW_HEIGHT, scale, &event_loop);
  let window_size = window.inner_size();

  let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
  let front_buffer = Pixels::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface_texture)?;
  let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
  let back_buffer = Pixels::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface_texture)?;

  let mut state = CompositorState {
    front_buffer: front_buffer,
    back_buffer: back_buffer,
    parity: 0
  };

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      Event::RedrawRequested(_) => {
        if state.front_buffer.render().map_err(|e| {println!("err {}", e)}).is_err() {
          *control_flow = ControlFlow::Exit;
          return;
        }
        state.swap();
      },
      Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
        *control_flow = ControlFlow::Exit;
        return;
      },
      Event::MainEventsCleared => {
        let drawn = compositor_step(&mut state);
        if drawn {
          window.request_redraw();
        }
      },
      Event::DeviceEvent{device_id: _, event: DeviceEvent::Key(input)} => {
        // TODO: do something
      }
      _ => ()
    };
  });
}
