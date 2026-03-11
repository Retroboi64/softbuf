use softbuf::{Color, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

const W: u32 = 800;
const H: u32 = 600;

fn main() -> windowed::Result<()> {
    let wc = WindowConfig::new("Title").size(W, H);
    let mut window = Window::new(wc)?;
    let presenter = Presenter::new(&window).expect("present");

    let mut fb = Framebuffer::new(W, H);

    fb.clear(Color::BLACK);

    window.run(|event| match event {
        Event::CloseRequested | Event::KeyDown(Key::Escape) => ControlFlow::Exit,
        Event::RedrawRequested => {
            presenter.present(&fb).ok();
            ControlFlow::Continue
        }
        _ => ControlFlow::Continue,
    })
}
