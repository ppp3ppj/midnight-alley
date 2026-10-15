use macroquad::prelude::*;

const RAT_SPEED: f32 = 60.0;
// the rats use the mouse art: 3 frames of 64x64 (walk_a, walk_b, rest), facing left
const CHAR_SHEET: &str = "assets/rat/rat_sheet.png";
const CHAR_PX: f32 = 64.0;
const CHAR_SCALE: f32 = 0.7;

// food art (what the kind old lady drops)
const FOOD_SHEET: &str = "assets/food/snail_shell.png";
const FOOD_PX: f32 = 64.0;
const FOOD_SCALE: f32 = 0.45;
const NIGHT_LEN: f32 = 50.0; // seconds of darkness
const MORNING_LEN: f32 = 12.0; // seconds of light, rats go sleep

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

#[derive(Clone, Copy, PartialEq)]
enum State {
    Wander,
    Sleep(usize), // index of the roof the rat sleeps on
}

struct Rat {
    id: u32,
    x: f32,
    dir: f32, // -1 left, +1 right
    hunger: f32, // 0 = full, 100 = dies
    energy: f32, // 100 = rested, low = wants to sleep
    state: State,
    turn_timer: f32,
    target_roof: Option<usize>, // roof it is heading to; kept until reached or taken
}

struct Roof {
    frac_x: f32, // centre as a fraction of screen width
    tier: f32,   // how high above the ground
    owner: Option<u32>, // which rat sleeps here (only one!)
}

impl Roof {
    fn rect(&self) -> Rect {
        Rect::new(self.frac_x * screen_width() - 1.5 * T, ground_y() - self.tier * 90.0, 3.0 * T, T)
    }
    fn center_x(&self) -> f32 {
        self.frac_x * screen_width()
    }
}

/// y of the top of the grass: where rats and food stand
fn ground_y() -> f32 {
    screen_height() - 2.0 * T
}

fn random_x() -> f32 {
    rand::gen_range(0.0, screen_width())
}

fn random_dir() -> f32 {
    if rand::gen_range(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 }
}

fn nearest(xs: &[f32], from: f32) -> Option<usize> {
    xs.iter()
        .enumerate()
        .min_by(|a, b| (a.1 - from).abs().total_cmp(&(b.1 - from).abs()))
        .map(|(i, _)| i)
}

fn nearest_free_roof(roofs: &[Roof], from: f32) -> Option<usize> {
    roofs
        .iter()
        .enumerate()
        .filter(|(_, r)| r.owner.is_none())
        .min_by(|a, b| (a.1.center_x() - from).abs().total_cmp(&(b.1.center_x() - from).abs()))
        .map(|(i, _)| i)
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

fn draw_rat(tex: &Texture2D, c: &Rat, y: f32) {
    // normal when full, tinted red when very hungry
    let t = (c.hunger / 100.0).clamp(0.0, 1.0);
    let tint = Color::new(1.0, 1.0 - 0.6 * t, 1.0 - 0.6 * t, 1.0);
    let size = CHAR_PX * CHAR_SCALE;

    // frame 0/1 = walking (offset by id so rats aren't in sync), 2 = resting
    let sleeping = matches!(c.state, State::Sleep(_));
    let frame = if sleeping { 2 } else { ((get_time() * 6.0) as usize + c.id as usize) % 2 };

    // little hop while walking, out of step between rats
    let bob = if sleeping { 0.0 } else { ((get_time() * 14.0) as f32 + c.id as f32).sin().abs() * 3.0 };

    draw_texture_ex(
        tex,
        (c.x - size / 2.0).round(),
        (y - size + 4.0 - bob).round(),
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            source: Some(Rect::new(frame as f32 * CHAR_PX, 0.0, CHAR_PX, CHAR_PX)),
            flip_x: c.dir > 0.0, // the art faces left, so flip when walking right
            ..Default::default()
        },
    );
    if sleeping {
        draw_text("z", c.x + size / 2.0 - 6.0, y - size + 8.0, 22.0, WHITE);
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

#[macroquad::main("Midnight Alley")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let sheet = load_texture(SHEET).await.expect("could not load farm_tilemap.png");
    sheet.set_filter(FilterMode::Nearest); // crisp pixels

    let char_tex = load_texture(CHAR_SHEET).await.expect("could not load rat_sheet.png");
    char_tex.set_filter(FilterMode::Linear); // the art is vector, smooth is fine
    let food_tex = load_texture(FOOD_SHEET).await.expect("could not load snail_shell.png");
    food_tex.set_filter(FilterMode::Linear);

    let mut rats: Vec<Rat> = (0..30)
        .map(|id| Rat {
            id,
            x: random_x(),
            dir: random_dir(),
            hunger: rand::gen_range(0.0, 40.0),
            energy: rand::gen_range(50.0, 100.0),
            state: State::Wander,
            turn_timer: 0.0,
            target_roof: None,
        })
        .collect();

    let mut food: Vec<f32> = Vec::new(); // x positions on the ground; only the player adds food

    let mut roofs: Vec<Roof> = [(0.12, 1.0), (0.38, 2.0), (0.62, 1.0), (0.88, 2.0)]
        .iter()
        .map(|&(frac_x, tier)| Roof { frac_x, tier, owner: None })
        .collect();

    let mut night = 1u32;
    let mut clock = 0.0f32; // seconds into the current night or morning
    let mut morning = false;

    loop {
        let dt = get_frame_time();
        let (sw, g) = (screen_width(), ground_y());

        // --- night <-> morning cycle ---
        clock += dt;
        if !morning && clock >= NIGHT_LEN {
            morning = true;
            clock = 0.0;
        } else if morning && clock >= MORNING_LEN {
            morning = false;
            clock = 0.0;
            night += 1;
        }
        // 0 = deep night, 1 = full morning (fades in and out around dawn)
        let daylight = if morning {
            (1.0 - (clock - MORNING_LEN / 2.0).abs() / (MORNING_LEN / 2.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // --- the kind old lady (you) drops food ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, _) = mouse_position();
            for _ in 0..5 {
                food.push((mx + rand::gen_range(-40.0, 40.0)).rem_euclid(sw));
            }
        }

        // --- update every rat ---
        for rat in rats.iter_mut() {
            rat.hunger += 4.0 * dt;

            if let State::Sleep(r) = rat.state {
                rat.energy = (rat.energy + 20.0 * dt).min(100.0);
                if rat.energy >= 100.0 && !morning {
                    roofs[r].owner = None; // free the roof
                    rat.state = State::Wander;
                }
                continue;
            }

            rat.energy -= 3.0 * dt;

            if rat.hunger > 40.0 {
                // 1. Hungry: go to the nearest food
                if let Some(i) = nearest(&food, rat.x) {
                    let f = food[i];
                    if (rat.x - f).abs() < 10.0 {
                        food.swap_remove(i); // first rat to arrive gets it
                        rat.hunger = (rat.hunger - 50.0).max(0.0);
                    } else {
                        rat.dir = (f - rat.x).signum();
                    }
                } else {
                    // no food anywhere: keep wandering instead of drifting forever
                    rat.turn_timer -= dt;
                    if rat.turn_timer <= 0.0 {
                        rat.dir = random_dir();
                        rat.turn_timer = rand::gen_range(1.0, 3.0);
                    }
                }
            } else if rat.energy < 30.0 || morning {
                // 2. Tired (or it's morning): go to the nearest free warm roof
                // stick to the chosen roof so the rat doesn't flip-flop between two equally close ones
                if rat.target_roof.is_some_and(|r| roofs[r].owner.is_some()) {
                    rat.target_roof = None; // someone else took it
                }
                if rat.target_roof.is_none() {
                    rat.target_roof = nearest_free_roof(&roofs, rat.x);
                }
                if let Some(r) = rat.target_roof {
                    let cx = roofs[r].center_x();
                    if (rat.x - cx).abs() < 8.0 {
                        roofs[r].owner = Some(rat.id); // claim it, hop up
                        rat.x = cx;
                        rat.state = State::Sleep(r);
                        rat.target_roof = None;
                        continue;
                    }
                    rat.dir = (cx - rat.x).signum();
                }
            } else {
                // 3. Happy: wander and sometimes turn
                rat.turn_timer -= dt;
                if rat.turn_timer <= 0.0 {
                    rat.dir = random_dir();
                    rat.turn_timer = rand::gen_range(1.0, 3.0);
                }
            }

            rat.x = (rat.x + rat.dir * RAT_SPEED * dt).rem_euclid(sw);
        }

        // --- rats that starve leave the alley (and free their roof) ---
        for rat in rats.iter().filter(|c| c.hunger >= 100.0) {
            if let State::Sleep(r) = rat.state {
                roofs[r].owner = None;
            }
        }
        rats.retain(|c| c.hunger < 100.0);

        // --- draw ---
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
        for r in &roofs {
            let rc = r.rect();
            for (k, idx) in HAY.iter().enumerate() {
                draw_tile(&sheet, *idx, rc.x + k as f32 * T, rc.y, tint);
            }
        }

        for &f in &food {
            draw_food(&food_tex, f, g);
        }
        for c in &rats {
            let y = match c.state {
                State::Sleep(r) => roofs[r].rect().y,
                State::Wander => g,
            };
            draw_rat(&char_tex, c, y);
        }

        let hud = format!(
            "Night {}{}   Rats: {}   Food: {}   (click to drop food)",
            night,
            if morning { " (morning)" } else { "" },
            rats.len(),
            food.len()
        );
        draw_text(&hud, 10.0, 24.0, 24.0, if daylight > 0.5 { BLACK } else { WHITE });

        if rats.is_empty() {
            let msg = "The alley is quiet now...";
            let size = measure_text(msg, None, 40, 1.0);
            draw_text(msg, (sw - size.width) / 2.0, screen_height() / 2.0, 40.0, WHITE);
        }

        next_frame().await
    }
}
