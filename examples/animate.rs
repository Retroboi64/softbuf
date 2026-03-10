//! Animates bouncing balls that leave color trails.
//!
//! Run with: cargo run --example animate

use softbuf::{Color, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

const W: u32 = 800;
const H: u32 = 600;

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    r: f32,
    color: Color,
}

impl Ball {
    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        if self.x - self.r < 0.0 {
            self.x = self.r;
            self.vx = self.vx.abs();
        }
        if self.x + self.r > W as f32 {
            self.x = W as f32 - self.r;
            self.vx = -self.vx.abs();
        }
        if self.y - self.r < 0.0 {
            self.y = self.r;
            self.vy = self.vy.abs();
        }
        if self.y + self.r > H as f32 {
            self.y = H as f32 - self.r;
            self.vy = -self.vy.abs();
        }
    }
}

fn main() -> windowed::Result<()> {
    let mut window = Window::new(WindowConfig::new("softbuf — bouncing balls").size(W, H))?;

    let presenter = Presenter::new(&window).expect("presenter");
    let mut fb = Framebuffer::filled(W, H, Color::BLACK);

    let mut balls = vec![
        Ball {
            x: 200.0,
            y: 150.0,
            vx: 3.1,
            vy: 2.3,
            r: 24.0,
            color: Color::CYAN,
        },
        Ball {
            x: 400.0,
            y: 300.0,
            vx: -2.5,
            vy: 3.7,
            r: 18.0,
            color: Color::MAGENTA,
        },
        Ball {
            x: 600.0,
            y: 450.0,
            vx: 2.0,
            vy: -2.9,
            r: 30.0,
            color: Color::YELLOW,
        },
        Ball {
            x: 100.0,
            y: 500.0,
            vx: -3.8,
            vy: -2.1,
            r: 14.0,
            color: Color::ORANGE,
        },
    ];

    window.request_redraw();

    window.run(|event| match event {
        Event::CloseRequested | Event::KeyDown(Key::Escape) => ControlFlow::Exit,

        Event::RedrawRequested => {
            // Fade the framebuffer toward black instead of clearing — creates trails
            for p in fb.pixels_mut() {
                let r = ((*p >> 16) & 0xFF).saturating_sub(8);
                let g = ((*p >> 8) & 0xFF).saturating_sub(8);
                let b = (*p & 0xFF).saturating_sub(8);
                *p = 0xFF000000 | (r << 16) | (g << 8) | b;
            }

            for ball in &mut balls {
                ball.update();
                fb.fill_circle(ball.x as i32, ball.y as i32, ball.r as i32, ball.color);
                // White halo for depth
                fb.draw_circle(
                    ball.x as i32,
                    ball.y as i32,
                    ball.r as i32,
                    Color::WHITE.with_alpha(80),
                );
            }

            presenter.present(&fb).ok();
            ControlFlow::Continue
        }

        _ => ControlFlow::Poll,
    })
}
