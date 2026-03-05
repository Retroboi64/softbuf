use softbuf::{Color, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

fn main() -> windowed::Result<()> {
    let mut window = Window::new(WindowConfig::new("softbuf — shapes").size(800, 600))?;
    let presenter = Presenter::new(&window).expect("presenter");

    let mut fb = Framebuffer::new(800, 600);
    draw_scene(&mut fb);

    window.run(|event| match event {
        Event::CloseRequested | Event::KeyDown(Key::Escape) => ControlFlow::Exit,
        Event::RedrawRequested => {
            presenter.present(&fb).ok();
            ControlFlow::Continue
        }
        _ => ControlFlow::Continue,
    })
}

fn draw_scene(fb: &mut Framebuffer) {
    fb.clear(Color::from_rgb_hex(0x1A1A2E));

    fb.draw_line(0, 0, 799, 599, Color::DARK_GRAY);
    fb.draw_line(799, 0, 0, 599, Color::DARK_GRAY);
    fb.draw_line_aa(10.0, 300.0, 790.0, 300.0, Color::LIGHT_GRAY);

    fb.fill_rect(20, 20, 120, 80, Color::from_rgb_hex(0x16213E));
    fb.draw_rect(20, 20, 120, 80, Color::CYAN);
    fb.fill_rect(160, 20, 120, 80, Color::from_rgb_hex(0x0F3460));
    fb.draw_rect(160, 20, 120, 80, Color::YELLOW);

    fb.fill_circle(100, 200, 60, Color::from_rgb_hex(0xE94560));
    fb.draw_circle(100, 200, 60, Color::WHITE);
    fb.draw_circle_thick(250, 200, 50, 4, Color::ORANGE);
    fb.fill_circle(400, 200, 40, Color::PURPLE.lighten(0.3));

    fb.fill_triangle(500, 50, 650, 50, 575, 180, Color::from_rgb_hex(0x533483));
    fb.draw_triangle(500, 50, 650, 50, 575, 180, Color::WHITE);

    fb.blend_pixel(400, 300, Color::WHITE.with_alpha(128));
    let mut overlay = Framebuffer::filled(200, 100, Color::BLUE.with_alpha(80));
    overlay.draw_rect(0, 0, 200, 100, Color::BLUE);
    fb.blit_blend(&overlay, 300, 400);

    fb.draw_rect(600, 400, 150, 100, Color::GREEN);
    fb.flood_fill(625, 425, Color::from_rgb_hex(0x004400));

    let mut patch = Framebuffer::new(100, 100);
    patch.clear(Color::WHITE);
    patch.fill_circle(50, 50, 45, Color::RED);
    patch.blur();
    fb.blit(&patch, 640, 20);

    for i in (0..800).step_by(50) {
        fb.draw_vline(i, 590, 599, Color::GRAY);
    }
}
