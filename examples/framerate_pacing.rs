//! Demonstrates high-precision frame pacing across different and ranging framerates.
//!
//! Run with: cargo run --example framerate_pacing

use softbuf::{Color, FramePacer, Framebuffer, Framerate, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

const W: u32 = 800;
const H: u32 = 600;

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: Color,
}

fn main() -> windowed::Result<()> {
    let mut window = Window::new(
        WindowConfig::new("softbuf — Frame Pacing & Ranging Framerates")
            .size(W, H)
            .resizable(true),
    )?;

    let mut presenter = Presenter::from_window(&window).expect("presenter");
    let mut fb = Framebuffer::new(W, H);

    let modes = [
        ("60 FPS (Standard)", Framerate::Fps(60.0)),
        ("30 FPS (Retro / Cinematic)", Framerate::Fps(30.0)),
        ("120 FPS (High Refresh)", Framerate::Fps(120.0)),
        ("144 FPS (Ultra Smooth)", Framerate::Fps(144.0)),
        (
            "Ranging FPS (20 <-> 144 dynamic)",
            Framerate::Ranging {
                min_fps: 20.0,
                max_fps: 144.0,
                cycle_secs: 5.0,
            },
        ),
        ("Uncapped (Benchmark)", Framerate::Uncapped),
    ];
    let mut mode_idx = 0;

    let mut pacer = FramePacer::with_target(modes[mode_idx].1);

    // Initialize particles
    let mut particles = Vec::new();
    for i in 0..40 {
        let angle = (i as f32) * (std::f32::consts::PI * 2.0 / 40.0);
        let speed = 150.0 + (i as f32 % 5.0) * 40.0;
        particles.push(Particle {
            x: W as f32 / 2.0,
            y: H as f32 / 2.0,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            color: Color::rgba(
                ((i * 37) % 256) as u8,
                ((i * 73) % 256) as u8,
                240,
                255,
            ),
        });
    }

    window.request_redraw();

    window.run(|event, window| match event {
        Event::CloseRequested
        | Event::KeyDown {
            key: Key::Escape, ..
        } => ControlFlow::Exit,

        Event::KeyDown {
            key: Key::Space,
            repeat: false,
            ..
        } => {
            mode_idx = (mode_idx + 1) % modes.len();
            pacer.set_target(modes[mode_idx].1);
            ControlFlow::Poll
        }

        Event::RedrawRequested => {
            // Tick pacer: sleeps precisely to pace the frame and yields timing stats
            let stats = pacer.tick();
            let dt = stats.delta_secs;

            // 1. Fade canvas for motion trails
            for p in fb.pixels_mut() {
                let r = ((*p >> 16) & 0xFF).saturating_sub(12);
                let g = ((*p >> 8) & 0xFF).saturating_sub(12);
                let b = (*p & 0xFF).saturating_sub(12);
                *p = 0xFF000000 | (r << 16) | (g << 8) | b;
            }

            // 2. Update and draw particles with delta-time (framerate independent physics)
            for pt in &mut particles {
                pt.x += pt.vx * dt;
                pt.y += pt.vy * dt;

                if pt.x < 10.0 {
                    pt.x = 10.0;
                    pt.vx = pt.vx.abs();
                } else if pt.x > (W - 10) as f32 {
                    pt.x = (W - 10) as f32;
                    pt.vx = -pt.vx.abs();
                }

                if pt.y < 70.0 {
                    pt.y = 70.0;
                    pt.vy = pt.vy.abs();
                } else if pt.y > (H - 120) as f32 {
                    pt.y = (H - 120) as f32;
                    pt.vy = -pt.vy.abs();
                }

                fb.fill_circle(pt.x as i32, pt.y as i32, 6, pt.color);
            }

            // 3. Header & Controls
            fb.fill_rect_gradient_h(0, 0, W as i32, 55, Color::from_rgb_hex(0x181825), Color::from_rgb_hex(0x313244));
            fb.draw_text_scaled(16, 12, "FRAME PACING & RANGING FRAMERATES", 2, Color::CYAN);
            fb.draw_text(16, 38, "[SPACE] -> Cycle Target FPS Mode | [ESC] -> Exit", Color::LIGHT_GRAY);

            // 4. Target Mode Card
            let mode_str = format!("Pacer Target: {}", modes[mode_idx].0);
            fb.draw_text_with_bg(16, 75, &mode_str, Color::YELLOW, Color::rgba(0, 0, 0, 200));

            if let Some(target_fps) = pacer.effective_target_fps() {
                let current_target_str = format!("Instant Target: {:.1} FPS", target_fps);
                fb.draw_text_with_bg(16, 95, &current_target_str, Color::GREEN, Color::rgba(0, 0, 0, 200));
            }

            // 5. Performance HUD & Real-Time Frame Time Graph
            fb.draw_fps_overlay(W as i32 - 320, 16, &stats);

            // Draw frame time graph at bottom right
            let graph_w = 300;
            let graph_h = 80;
            let graph_x = W as i32 - graph_w - 20;
            let graph_y = H as i32 - graph_h - 20;

            fb.draw_fps_graph(
                graph_x,
                graph_y,
                graph_w as u32,
                graph_h as u32,
                pacer.history(),
                Color::CYAN,
                Color::rgba(20, 20, 30, 220),
            );

            fb.draw_text(graph_x + 6, graph_y + 6, "Frame Time Jitter (ms)", Color::GRAY);
            fb.draw_text(graph_x + graph_w - 60, graph_y + 6, "50ms max", Color::DARK_GRAY);

            // 6. Blit to window
            presenter.present(&fb).ok();
            window.request_redraw();
            ControlFlow::Poll
        }

        Event::Resized { .. } => {
            window.request_redraw();
            ControlFlow::Poll
        }

        _ => ControlFlow::Poll,
    })
}
