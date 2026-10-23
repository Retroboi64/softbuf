use softbuf::{Color, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

const W: u32 = 800;
const H: u32 = 600;

fn main() -> windowed::Result<()> {
    let wc = WindowConfig::new("softbuf — simple window").size(W, H);
    let mut window = Window::new(wc)?;
    let mut presenter = Presenter::from_window(&window).expect("presenter");

    let mut fb = Framebuffer::new(W, H);
    fb.clear(Color::from_rgb_hex(0x1E1E2E));
    fb.draw_text_scaled(40, 40, "Hello, softbuf + windowed!", 2, Color::CYAN);
    fb.draw_text(40, 100, "Press Escape to close.", Color::LIGHT_GRAY);

    window.request_redraw();

    window.run(|event, window| match event {
        Event::CloseRequested
        | Event::KeyDown {
            key: Key::Escape, ..
        } => ControlFlow::Exit,

        Event::RedrawRequested => {
            presenter.present(&fb).ok();
            ControlFlow::Wait
        }

        Event::Resized { .. } => {
            window.request_redraw();
            ControlFlow::Poll
        }

        _ => ControlFlow::Wait,
    })
}
