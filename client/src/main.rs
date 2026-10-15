use macroquad::prelude::*;

struct Cat { x: f32, y: f32, dx: f32, dy: f32 }

#[macroquad::main("Midnight Alley")]
async fn main() {
    let mut cats: Vec<Cat> = (0..50).map(|_| Cat {
        x: rand::gen_range(0.0, screen_width()),
        y: rand::gen_range(0.0, screen_height()),
        dx: rand::gen_range(-1.0, 1.0),
        dy: rand::gen_range(-1.0, 1.0),
    }).collect();

    loop {
        clear_background(Color::from_rgba(15, 20, 45, 255));
        for c in cats.iter_mut() {
            c.x = (c.x + c.dx).rem_euclid(screen_width());
            c.y = (c.y + c.dy).rem_euclid(screen_height());
            draw_circle(c.x, c.y, 6.0, ORANGE);
            draw_triangle(vec2(c.x - 6.0, c.y - 3.0), vec2(c.x - 3.0, c.y - 10.0), vec2(c.x, c.y - 5.0), ORANGE);
            draw_triangle(vec2(c.x + 6.0, c.y - 3.0), vec2(c.x + 3.0, c.y - 10.0), vec2(c.x, c.y - 5.0), ORANGE);
        }
        next_frame().await
    }
}
