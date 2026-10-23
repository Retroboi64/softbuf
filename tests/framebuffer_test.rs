use softbuf::{Color, Filter, Framebuffer, Rect};

#[test]
fn test_color_conversions_and_blending() {
    let red = Color::rgb(255, 0, 0);
    assert_eq!(red.to_argb(), 0xFFFF0000);
    assert_eq!(Color::from_argb(0xFFFF0000), red);

    let hex_c = Color::from_rgb_hex(0x123456);
    assert_eq!(hex_c.r, 0x12);
    assert_eq!(hex_c.g, 0x34);
    assert_eq!(hex_c.b, 0x56);
    assert_eq!(hex_c.a, 255);

    // Alpha blending test
    let opaque_blue = Color::rgb(0, 0, 255);
    let transparent_red = Color::rgba(255, 0, 0, 0);
    assert_eq!(transparent_red.blend_over(opaque_blue), opaque_blue);

    let opaque_red = Color::rgb(255, 0, 0);
    assert_eq!(opaque_red.blend_over(opaque_blue), opaque_red);

    let semi_white = Color::rgba(255, 255, 255, 128);
    let blended = semi_white.blend_over(Color::BLACK);
    assert!(blended.r > 120 && blended.r < 135);
    assert_eq!(blended.a, 255);
}

#[test]
fn test_rect_geometry() {
    let r1 = Rect::new(10, 10, 50, 50);
    assert!(r1.contains(10, 10));
    assert!(r1.contains(59, 59));
    assert!(!r1.contains(60, 60));
    assert!(!r1.contains(9, 10));

    let r2 = Rect::new(30, 30, 50, 50);
    assert!(r1.intersects(&r2));

    let inter = r1.intersection(&r2).unwrap();
    assert_eq!(inter, Rect::new(30, 30, 30, 30));

    let r3 = Rect::new(100, 100, 20, 20);
    assert!(!r1.intersects(&r3));
    assert_eq!(r1.intersection(&r3), None);

    let union = r1.union(&r2);
    assert_eq!(union, Rect::new(10, 10, 70, 70));
}

#[test]
fn test_framebuffer_set_pixel_unchecked_bug_fix() {
    let mut fb = Framebuffer::new(10, 10);
    fb.set_pixel_unchecked(3, 4, Color::CYAN);
    assert_eq!(fb.get_pixel(3, 4), Color::CYAN);
    assert_eq!(fb.get_pixel(0, 0), Color::BLACK);
}

#[test]
fn test_clipping_stack() {
    let mut fb = Framebuffer::new(100, 100);
    fb.clear(Color::BLACK);

    // Push clip rect [20, 20, 40, 40]
    fb.push_clip_rect(Rect::new(20, 20, 40, 40));

    // Try drawing a line that spans entire width
    fb.draw_line(0, 30, 99, 30, Color::RED);

    // Pixels outside clip should still be black
    assert_eq!(fb.get_pixel(10, 30), Color::BLACK);
    assert_eq!(fb.get_pixel(19, 30), Color::BLACK);
    assert_eq!(fb.get_pixel(60, 30), Color::BLACK);
    assert_eq!(fb.get_pixel(90, 30), Color::BLACK);

    // Pixels inside clip must be RED
    assert_eq!(fb.get_pixel(20, 30), Color::RED);
    assert_eq!(fb.get_pixel(30, 30), Color::RED);
    assert_eq!(fb.get_pixel(59, 30), Color::RED);

    // Pop clip
    fb.pop_clip_rect();
    assert_eq!(fb.clip_rect(), Rect::new(0, 0, 100, 100));
}

#[test]
fn test_triangle_flat_edges_no_division_by_zero() {
    let mut fb = Framebuffer::new(50, 50);

    // Flat bottom triangle
    fb.fill_triangle(10, 10, 40, 10, 25, 30, Color::GREEN);

    // Flat top triangle
    fb.fill_triangle(25, 10, 10, 30, 40, 30, Color::BLUE);

    // Degenerate zero-height triangle (should return gracefully without panicking)
    fb.fill_triangle(10, 20, 20, 20, 30, 20, Color::WHITE);
}

#[test]
fn test_scanline_flood_fill() {
    let mut fb = Framebuffer::new(50, 50);
    fb.clear(Color::BLACK);

    // Draw an enclosed box
    fb.draw_rect(10, 10, 30, 30, Color::WHITE);

    // Flood fill the interior
    fb.flood_fill(15, 15, Color::RED);

    // Center should be red
    assert_eq!(fb.get_pixel(15, 15), Color::RED);
    assert_eq!(fb.get_pixel(25, 25), Color::RED);

    // Border remains white
    assert_eq!(fb.get_pixel(10, 10), Color::WHITE);
    assert_eq!(fb.get_pixel(39, 39), Color::WHITE);

    // Outside remains black
    assert_eq!(fb.get_pixel(5, 5), Color::BLACK);
    assert_eq!(fb.get_pixel(45, 45), Color::BLACK);
}

#[test]
fn test_wu_anti_aliased_lines() {
    let mut fb = Framebuffer::new(60, 60);
    fb.clear(Color::BLACK);

    // Draw AA line with negative and out-of-bounds coordinates
    fb.draw_line_aa(-10.0, 10.0, 70.0, 50.0, Color::WHITE);

    // Pixels along the line should have valid alpha without crashing
    let px = fb.get_pixel(30, 30);
    assert_eq!(px.r, px.g);
    assert_eq!(px.g, px.b);
}

#[test]
fn test_text_rendering() {
    let mut fb = Framebuffer::new(100, 40);
    fb.clear(Color::BLACK);

    fb.draw_text(10, 10, "Hi", Color::YELLOW);

    // 'H' glyph starts around x=10, y=10
    let mut found_yellow = false;
    for y in 10..18 {
        for x in 10..26 {
            if fb.get_pixel(x, y) == Color::YELLOW {
                found_yellow = true;
                break;
            }
        }
    }
    assert!(found_yellow, "Text glyph should write colored pixels");
}

#[test]
fn test_scaled_blit_and_filters() {
    let mut src = Framebuffer::new(2, 2);
    src.set_pixel(0, 0, Color::RED);
    src.set_pixel(1, 0, Color::GREEN);
    src.set_pixel(0, 1, Color::BLUE);
    src.set_pixel(1, 1, Color::WHITE);

    let mut dst = Framebuffer::new(20, 20);
    dst.clear(Color::BLACK);

    // Nearest scaling 2x2 -> 10x10
    dst.blit_scaled(&src, Rect::new(0, 0, 2, 2), Rect::new(0, 0, 10, 10), Filter::Nearest);
    assert_eq!(dst.get_pixel(1, 1), Color::RED);
    assert_eq!(dst.get_pixel(8, 1), Color::GREEN);

    // Bilinear scaling
    dst.blit_scaled(&src, Rect::new(0, 0, 2, 2), Rect::new(10, 10, 10, 10), Filter::Bilinear);
    let center_sample = dst.get_pixel(15, 15);
    assert_ne!(center_sample, Color::BLACK);
}

#[test]
fn test_damage_tracking_diff_rects() {
    let mut fb1 = Framebuffer::new(64, 64);
    fb1.clear(Color::BLACK);

    let mut fb2 = fb1.clone();
    // Change a small region
    fb2.fill_rect(10, 10, 5, 5, Color::CYAN);

    let diffs = fb2.diff_rects(&fb1);
    assert!(!diffs.is_empty(), "Should detect dirty rectangle");
    let bounding = diffs.iter().fold(Rect::default(), |acc, r| acc.union(r));
    assert!(bounding.contains(12, 12));
}
