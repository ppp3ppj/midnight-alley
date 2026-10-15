//! Midnight Alley client: draws the world the Clojure server simulates.
//! It receives JSON snapshots over a WebSocket and sends clicks (drop food) and
//! the B key (toggle broken mode) back.

use std::collections::HashMap;

use macroquad::prelude::*;
use quad_net::web_socket::WebSocket;
use serde::Deserialize;

// ---------------------------------------------------------------- assets

// the rats use the mouse art: 3 frames of 64x64 (walk_a, walk_b, rest), facing left
const CHAR_SHEET: &str = "assets/rat/rat_sheet.png";
const CHAR_PX: f32 = 64.0;
const CHAR_SCALE: f32 = 0.7;

// food art (what the kind old lady drops)
const FOOD_SHEET: &str = "assets/food/snail_shell.png";
const FOOD_PX: f32 = 64.0;
const FOOD_SCALE: f32 = 0.45;

// Kenney "Pixel Platformer: Farm Expansion" packed sheet: 18px tiles, 16 columns, no spacing
const SHEET: &str = "assets/tiles/farm_tilemap.png";
const TILE_PX: f32 = 18.0;
const SHEET_COLS: usize = 16;
const SCALE: f32 = 3.0;
const T: f32 = TILE_PX * SCALE; // on-screen tile size

// tile indices in the sheet
const GRASS_TOP: [usize; 3] = [1, 2, 3];
const DIRT: [usize; 3] = [33, 34, 35];
const HAY: [usize; 3] = [6, 7, 8]; // roof platform: left, middle, right
const PUMPKIN: usize = 4;
const SUNFLOWER: (usize, usize) = (20, 36); // flower head, stem
const GREENHOUSE_DARK: usize = 64; // 4x3 block starting here (rows are +16)
const GREENHOUSE_LIT: usize = 68;

// ---------------------------------------------------------------- server

/// The server's x positions run 0..WORLD_W; we scale them to the window.
const WORLD_W: f32 = 1000.0;

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_SERVER: &str = "127.0.0.1:8080";
#[cfg(target_arch = "wasm32")]
const DEFAULT_SERVER: &str = "same-origin"; // the page's own host; see server/public/js/quad-net.js

/// One snapshot from the server. Keys like "safe?" and "food-count" are Clojure style.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Snapshot {
    #[serde(rename = "safe?")]
    safe: bool,
    night: u32,
    #[serde(rename = "morning?")]
    morning: bool,
    daylight: f32,
    #[serde(rename = "food-count")]
    food_count: i64, // the server's own count: goes negative in broken mode
    food: Vec<FoodView>,
    roofs: Vec<RoofView>,
    rats: Vec<RatView>,
    diary: DiaryView,
    events: Vec<EventView>,
}

#[derive(Deserialize, Default)]
struct FoodView {
    x: f32,
}

#[derive(Deserialize, Default)]
struct RoofView {
    x: f32,
    tier: f32,
}

#[derive(Deserialize, Default)]
struct RatView {
    id: u64,
    x: f32,
    dir: f32,
    hunger: f32,
    state: String, // "wander" | "sleep"
    roof: Option<usize>,
}

#[derive(Deserialize, Default)]
struct DiaryView {
    alive: u32,
    born: u32,
    eaten: u32,
}

#[derive(Deserialize, Default)]
struct EventView {
    #[serde(rename = "type")]
    kind: String,
    x: f32,
    roof: Option<usize>,
}

struct Net {
    ws: Option<WebSocket>,
    last_msg: f64,
    last_try: f64,
}

impl Net {
    fn new() -> Self {
        let mut net = Net { ws: None, last_msg: 0.0, last_try: 0.0 };
        net.try_connect();
        net
    }

    fn server() -> String {
        std::env::var("MIDNIGHT_SERVER").unwrap_or_else(|_| DEFAULT_SERVER.to_string())
    }

    /// The desktop WebSocket panics if nothing is listening, so check the port first.
    #[cfg(not(target_arch = "wasm32"))]
    fn reachable() -> bool {
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;
        Self::server()
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .is_some_and(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok())
    }

    #[cfg(target_arch = "wasm32")]
    fn reachable() -> bool {
        true
    }

    fn try_connect(&mut self) {
        self.last_try = get_time();
        self.last_msg = get_time();
        self.ws = if Self::reachable() {
            WebSocket::connect(format!("ws://{}/ws", Self::server())).ok()
        } else {
            None
        };
    }

    fn connected(&self) -> bool {
        self.ws.is_some() && get_time() - self.last_msg < 3.0
    }

    /// All snapshots that arrived since the last call, oldest first.
    fn poll(&mut self) -> Vec<Snapshot> {
        let mut out = Vec::new();
        if let Some(ws) = self.ws.as_mut() {
            while let Some(bytes) = ws.try_recv() {
                self.last_msg = get_time();
                match serde_json::from_slice::<Snapshot>(&bytes) {
                    Ok(snap) => out.push(snap),
                    Err(e) => eprintln!("bad snapshot: {e}"),
                }
            }
        }
        // nothing for a while (server down or restarted): try again every 3 seconds
        if !self.connected() && get_time() - self.last_try > 3.0 {
            self.try_connect();
        }
        out
    }

    fn send(&self, json: &str) {
        if let Some(ws) = &self.ws {
            if ws.connected() {
                ws.send_text(json); // in the browser, sending before the socket is open would throw
            }
        }
    }
}

// ---------------------------------------------------------------- drawing

/// y of the top of the grass: where rats and food stand
fn ground_y() -> f32 {
    screen_height() - 2.0 * T
}

fn to_screen_x(world_x: f32) -> f32 {
    world_x / WORLD_W * screen_width()
}

fn roof_rect(r: &RoofView) -> Rect {
    Rect::new(to_screen_x(r.x) - 1.5 * T, ground_y() - r.tier * 90.0, 3.0 * T, T)
}

fn draw_tile(sheet: &Texture2D, idx: usize, x: f32, y: f32, tint: Color) {
    let (col, row) = ((idx % SHEET_COLS) as f32, (idx / SHEET_COLS) as f32);
    draw_texture_ex(
        sheet,
        x.round(),
        y.round(),
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(T, T)),
            source: Some(Rect::new(col * TILE_PX, row * TILE_PX, TILE_PX, TILE_PX)),
            ..Default::default()
        },
    );
}

fn draw_rat(tex: &Texture2D, r: &RatView, x: f32, y: f32) {
    // normal when full, tinted red when very hungry
    let t = (r.hunger / 100.0).clamp(0.0, 1.0);
    let tint = Color::new(1.0, 1.0 - 0.6 * t, 1.0 - 0.6 * t, 1.0);
    let size = CHAR_PX * CHAR_SCALE;

    // frame 0/1 = walking (offset by id so rats aren't in sync), 2 = resting
    let sleeping = r.state == "sleep";
    let frame = if sleeping { 2 } else { ((get_time() * 6.0) as usize + r.id as usize) % 2 };

    // little hop while walking, out of step between rats
    let bob = if sleeping { 0.0 } else { ((get_time() * 14.0) as f32 + r.id as f32).sin().abs() * 3.0 };

    draw_texture_ex(
        tex,
        (x - size / 2.0).round(),
        (y - size + 4.0 - bob).round(),
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            source: Some(Rect::new(frame as f32 * CHAR_PX, 0.0, CHAR_PX, CHAR_PX)),
            flip_x: r.dir > 0.0, // the art faces left, so flip when walking right
            ..Default::default()
        },
    );
    if sleeping {
        draw_text("z", x + size / 2.0 - 6.0, y - size + 8.0, 22.0, WHITE);
    }
}

/// One spark of the death explosion. `ring` particles are the expanding shock wave.
struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    color: Color,
    ring: bool,
}

/// "Boom": a flash, an expanding ring and a burst of sparks flying up and out.
fn spawn_boom(particles: &mut Vec<Particle>, at: Vec2) {
    particles.push(Particle { pos: at, vel: Vec2::ZERO, life: 0.35, max_life: 0.35, size: 10.0, color: WHITE, ring: true });
    let palette = [
        Color::from_rgba(255, 220, 80, 255),
        Color::from_rgba(255, 140, 40, 255),
        Color::from_rgba(255, 80, 60, 255),
        Color::from_rgba(120, 120, 130, 255), // smoke
    ];
    for _ in 0..24 {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let speed = rand::gen_range(60.0, 260.0);
        let life = rand::gen_range(0.4, 0.9);
        particles.push(Particle {
            pos: at,
            vel: vec2(angle.cos() * speed, angle.sin() * speed - 80.0),
            life,
            max_life: life,
            size: rand::gen_range(2.0, 5.0),
            color: palette[rand::gen_range(0, palette.len())],
            ring: false,
        });
    }
}

fn draw_particle(p: &Particle) {
    let t = (p.life / p.max_life).clamp(0.0, 1.0); // 1 -> 0 as it fades
    if p.ring {
        let radius = p.size + (1.0 - t) * 55.0;
        draw_circle(p.pos.x, p.pos.y, radius * 0.6, Color::new(1.0, 0.9, 0.6, 0.5 * t)); // flash
        draw_circle_lines(p.pos.x, p.pos.y, radius, 3.0, Color::new(1.0, 1.0, 1.0, t));
    } else {
        draw_circle(p.pos.x, p.pos.y, p.size * (0.4 + 0.6 * t), Color::new(p.color.r, p.color.g, p.color.b, t));
    }
}

fn draw_food(tex: &Texture2D, x: f32, y: f32) {
    let size = FOOD_PX * FOOD_SCALE;
    draw_texture_ex(
        tex,
        (x - size / 2.0).round(),
        (y - size + 4.0).round(),
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() },
    );
}

fn draw_centered(msg: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text(msg, None, size, 1.0);
    draw_text(msg, (screen_width() - dims.width) / 2.0, y, size as f32, color);
}

// ---------------------------------------------------------------- main

#[macroquad::main("Midnight Alley")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let sheet = load_texture(SHEET).await.expect("could not load farm_tilemap.png");
    sheet.set_filter(FilterMode::Nearest); // crisp pixels
    let char_tex = load_texture(CHAR_SHEET).await.expect("could not load rat_sheet.png");
    char_tex.set_filter(FilterMode::Linear); // the art is vector, smooth is fine
    let food_tex = load_texture(FOOD_SHEET).await.expect("could not load snail_shell.png");
    food_tex.set_filter(FilterMode::Linear);

    let mut net = Net::new();
    let mut snap = Snapshot::default();
    let mut got_snapshot = false;
    let mut shown_x: HashMap<u64, f32> = HashMap::new(); // smoothed rat positions (snapshots are ~10 per second)
    let mut particles: Vec<Particle> = Vec::new();

    loop {
        let dt = get_frame_time();
        let (sw, g) = (screen_width(), ground_y());

        // --- take in what the server sent ---
        for new_snap in net.poll() {
            for ev in &new_snap.events {
                if ev.kind == "death" {
                    let y = ev.roof.and_then(|i| new_snap.roofs.get(i)).map_or(g, |r| roof_rect(r).y);
                    spawn_boom(&mut particles, vec2(to_screen_x(ev.x), y - CHAR_PX * CHAR_SCALE * 0.4));
                }
            }
            snap = new_snap;
            got_snapshot = true;
        }

        // --- the kind old lady (you) drops food ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, _) = mouse_position();
            net.send(&format!(r#"{{"type":"drop","x":{}}}"#, mx / sw * WORLD_W));
        }
        // B: break / fix the shared state (STM off / on)
        if is_key_pressed(KeyCode::B) {
            net.send(&format!(r#"{{"type":"mode","safe":{}}}"#, !snap.safe));
        }

        // --- smooth the rats between snapshots ---
        shown_x.retain(|id, _| snap.rats.iter().any(|r| r.id == *id));
        for r in &snap.rats {
            let target = to_screen_x(r.x);
            let x = shown_x.entry(r.id).or_insert(target);
            // snap when sleeping or when wrapping around the edge, otherwise glide
            if r.state == "sleep" || (*x - target).abs() > sw / 2.0 {
                *x = target;
            } else {
                *x += (target - *x) * (14.0 * dt).min(1.0);
            }
        }

        for p in particles.iter_mut() {
            p.life -= dt;
            p.vel.y += 500.0 * dt; // gravity
            p.pos += p.vel * dt;
        }
        particles.retain(|p| p.life > 0.0);

        // --- draw ---
        let daylight = snap.daylight.clamp(0.0, 1.0);
        // sky: deep night blue -> light gray morning
        let (nr, ng, nb) = (15.0, 20.0, 45.0);
        let (dr, dg, db) = (200.0, 200.0, 205.0);
        let mix = |n: f32, d: f32| (n + (d - n) * daylight) as u8;
        clear_background(Color::from_rgba(mix(nr, dr), mix(ng, dg), mix(nb, db), 255));
        draw_circle(sw - 60.0, 60.0, 25.0, Color::from_rgba(250, 240, 200, (255.0 * (1.0 - daylight)) as u8)); // moon

        // everything made of tiles is dimmed at night
        let dim = 0.5 + 0.5 * daylight;
        let tint = Color::new(dim * 0.85 + 0.15 * daylight, dim * 0.9, dim, 1.0);

        // greenhouse in the background (windows lit at night)
        let gh_x = sw * 0.5 - 2.0 * T;
        let gh = if daylight > 0.5 { GREENHOUSE_DARK } else { GREENHOUSE_LIT };
        for row in 0..3 {
            for col in 0..4 {
                draw_tile(&sheet, gh + row * 16 + col, gh_x + col as f32 * T, g - (3 - row) as f32 * T, tint);
            }
        }
        // a few decorations standing on the grass
        for (frac, idx) in [(0.05, PUMPKIN), (0.27, PUMPKIN), (0.78, PUMPKIN)] {
            draw_tile(&sheet, idx, frac * sw, g - T, tint);
        }
        for frac in [0.2, 0.72, 0.95] {
            draw_tile(&sheet, SUNFLOWER.0, frac * sw, g - 2.0 * T, tint);
            draw_tile(&sheet, SUNFLOWER.1, frac * sw, g - T, tint);
        }

        // ground: grass strip + dirt
        let cols = (sw / T).ceil() as usize + 1;
        for i in 0..cols {
            let x = i as f32 * T;
            draw_tile(&sheet, GRASS_TOP[1 + (i % 2)], x, g, tint);
            let mut y = g + T;
            let mut row = 0;
            while y < screen_height() {
                draw_tile(&sheet, DIRT[1 + (i + row) % 2] + 16 * (row % 2), x, y, tint);
                y += T;
                row += 1;
            }
        }

        // warm hay-bale roofs
        for r in &snap.roofs {
            let rc = roof_rect(r);
            for (k, idx) in HAY.iter().enumerate() {
                draw_tile(&sheet, *idx, rc.x + k as f32 * T, rc.y, tint);
            }
        }

        for f in &snap.food {
            draw_food(&food_tex, to_screen_x(f.x), g);
        }
        for r in &snap.rats {
            let x = shown_x.get(&r.id).copied().unwrap_or_else(|| to_screen_x(r.x));
            let y = match (r.state.as_str(), r.roof.and_then(|i| snap.roofs.get(i))) {
                ("sleep", Some(roof)) => roof_rect(roof).y,
                _ => g,
            };
            draw_rat(&char_tex, r, x, y);
        }
        for p in &particles {
            draw_particle(p);
        }

        // --- HUD ---
        let ink = if daylight > 0.5 { BLACK } else { WHITE };
        if !got_snapshot {
            draw_centered("Connecting to the Clojure server...", screen_height() / 2.0, 36, WHITE);
            draw_centered(&format!("(start it with `lein run` in server/, expecting {})", Net::server()), screen_height() / 2.0 + 34.0, 22, LIGHTGRAY);
        } else {
            let hud = format!(
                "Night {}{}   Rats: {}   Pups born: {}   Food eaten: {}   (click to drop food)",
                snap.night,
                if snap.morning { " (morning)" } else { "" },
                snap.diary.alive,
                snap.diary.born,
                snap.diary.eaten,
            );
            draw_text(&hud, 10.0, 24.0, 24.0, ink);

            if snap.safe {
                draw_text(&format!("STM on   Food: {}   (press B to break it)", snap.food_count), 10.0, 50.0, 22.0, ink);
            } else {
                // the ghost-food demo: this count can go below zero
                draw_text(
                    &format!("BROKEN MODE (no STM)   Food: {}   (press B to fix it)", snap.food_count),
                    10.0,
                    50.0,
                    22.0,
                    RED,
                );
            }
            if !net.connected() {
                draw_centered("Lost connection to the server, retrying...", 90.0, 28, ORANGE);
            } else if snap.rats.is_empty() {
                draw_centered("The alley is quiet now...", screen_height() / 2.0, 40, WHITE);
            }
        }

        next_frame().await
    }
}
