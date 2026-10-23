//! Interactive demonstration of softbuf's text, clipping, gradients, and scaling modes.
//!
//! Run with: cargo run --example text_and_ui

use softbuf::{Color, Framebuffer, Presenter, Rect, ScaleMode};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

fn main() -> windowed::Result<()> {
    let mut window = Window::new(
        WindowConfig::new("softbuf — Text, UI & Scaling Modes")
            .size(800, 600)
            .resizable(true),
    )?;

    let mut presenter = Presenter::from_window(&window).expect("presenter");
    let mut fb = Framebuffer::new(CANVAS_W, CANVAS_H);

    let scale_modes = [
        ScaleMode::Fit,
        ScaleMode::IntegerScale,
        ScaleMode::Stretch,
        ScaleMode::Center,
    ];
    let mode_names = ["ScaleMode::Fit", "ScaleMode::IntegerScale", "ScaleMode::Stretch", "ScaleMode::Center"];
    let mut current_mode_idx = 0;

    let mut mouse_pos = (0, 0);

    render_canvas(&mut fb, mouse_pos, mode_names[current_mode_idx]);
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
            current_mode_idx = (current_mode_idx + 1) % scale_modes.len();
            let mut opts = *presenter.options();
            opts.scale_mode = scale_modes[current_mode_idx];
            presenter.set_options(opts);

            render_canvas(&mut fb, mouse_pos, mode_names[current_mode_idx]);
            window.request_redraw();
            ControlFlow::Poll
        }

        Event::MouseMove { x, y } => {
            mouse_pos = (x, y);
            render_canvas(&mut fb, mouse_pos, mode_names[current_mode_idx]);
            window.request_redraw();
            ControlFlow::Poll
        }

        Event::Resized { .. } => {
            window.request_redraw();
            ControlFlow::Poll
        }

        Event::RedrawRequested => {
            presenter.present(&fb).ok();
            ControlFlow::Wait
        }

        _ => ControlFlow::Wait,
    })
}

fn render_canvas(fb: &mut Framebuffer, mouse: (i32, i32), mode_name: &str) {
    // 1. Dark backdrop
    fb.clear(Color::from_rgb_hex(0x181825));

    // 2. Header banner with horizontal gradient
    fb.fill_rect_gradient_h(0, 0, CANVAS_W as i32, 50, Color::from_rgb_hex(0x313244), Color::from_rgb_hex(0x181825));
    fb.draw_text_scaled(16, 14, "SOFTBUF UI & TEXT ENGINE", 2, Color::from_rgb_hex(0x89B4FA));

    // 3. UI Card with Rounded Rect and Clipping
    let card_x = 20;
    let card_y = 70;
    let card_w = 280;
    let card_h = 180;
    fb.fill_rounded_rect(card_x, card_y, card_w, card_h, 12, Color::from_rgb_hex(0x1E1E2E));
    fb.draw_rounded_rect(card_x, card_y, card_w, card_h, 12, Color::from_rgb_hex(0x45475A));

    // Nested clipped region inside the card
    fb.push_clip_rect(Rect::new(card_x + 10, card_y + 10, (card_w - 20) as u32, (card_h - 20) as u32));
    fb.draw_text_scaled(card_x + 12, card_y + 12, "Clipped Viewport", 1, Color::from_rgb_hex(0xF9E2AF));
    fb.draw_text(
        card_x + 12,
        card_y + 30,
        "Primitives inside this card\nare safely bounded by the\nclip stack. Lines and text\nnever leak outside!",
        Color::from_rgb_hex(0xCDD6F4),
    );
    // Draw a line that extends outside the card to demonstrate clipping
    fb.draw_line(card_x - 50, card_y + 120, card_x + 400, card_y + 120, Color::from_rgb_hex(0xF38BA8));
    fb.draw_text(card_x + 12, card_y + 130, "<-- Clipped Red Line", Color::from_rgb_hex(0xF38BA8));
    fb.pop_clip_rect();

    // 4. Geometry and curves card
    let geo_x = 320;
    let geo_y = 70;
    let geo_w = 300;
    let geo_h = 180;
    fb.fill_rounded_rect(geo_x, geo_y, geo_w, geo_h, 12, Color::from_rgb_hex(0x1E1E2E));
    fb.draw_rounded_rect(geo_x, geo_y, geo_w, geo_h, 12, Color::from_rgb_hex(0x45475A));
    fb.draw_text(geo_x + 16, geo_y + 14, "Smooth Curves & Shapes", Color::from_rgb_hex(0xA6E3A1));

    // Ellipse and Radial Gradient
    fb.fill_ellipse(geo_x + 60, geo_y + 80, 45, 30, Color::from_rgb_hex(0xCBA6F7));
    fb.draw_ellipse(geo_x + 60, geo_y + 80, 45, 30, Color::WHITE);

    // Radial gradient orb
    fb.fill_circle_gradient(geo_x + 160, geo_y + 80, 30, Color::YELLOW, Color::RED);

    // Cubic Bezier curve
    fb.draw_cubic_bezier(
        (geo_x + 210, geo_y + 120),
        (geo_x + 230, geo_y + 40),
        (geo_x + 270, geo_y + 130),
        (geo_x + 290, geo_y + 50),
        Color::CYAN,
    );

    // 5. Interactive & Status Bar at bottom
    let bot_y = 270;
    fb.fill_rounded_rect(20, bot_y, (CANVAS_W - 40) as i32, 180, 12, Color::from_rgb_hex(0x1E1E2E));
    fb.draw_rounded_rect(20, bot_y, (CANVAS_W - 40) as i32, 180, 12, Color::from_rgb_hex(0x585B70));

    fb.draw_text_scaled(40, bot_y + 20, "CONTROLS & PRESENTATION", 1, Color::from_rgb_hex(0x89DCEB));

    let info_text = format!(
        "Active ScaleMode: {}\n[SPACE] -> Cycle ScaleMode (Fit / Integer / Stretch / Center)\n[RESIZE WINDOW] -> Watch automatic aspect-ratio letterboxing\nMouse Coordinates: ({}, {})\nFrame Dimensions: {}x{}",
        mode_name, mouse.0, mouse.1, CANVAS_W, CANVAS_H
    );
    fb.draw_text(40, bot_y + 44, &info_text, Color::WHITE);

    // Dynamic mouse cursor indicator in canvas coordinates
    fb.fill_circle(mouse.0.clamp(0, CANVAS_W as i32), mouse.1.clamp(0, CANVAS_H as i32), 6, Color::from_rgb_hex(0xF5E0DC));
    fb.draw_circle(mouse.0.clamp(0, CANVAS_W as i32), mouse.1.clamp(0, CANVAS_H as i32), 10, Color::CYAN);
}
