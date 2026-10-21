use easy_ray::prelude::*;

const WIDTH: f32 = 960.0;
const HEIGHT: f32 = 640.0;
const PLAYER_RADIUS: f32 = 12.0;
const BULLET_SPEED: f32 = 390.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Menu,
    Playing,
    GameOver,
}

struct Player {
    position: Vector2,
    velocity: Vector2,
    angle: f32,
    lives: u32,
    invulnerable: f32,
    shoot_cooldown: f32,
}

struct Asteroid {
    position: Vector2,
    velocity: Vector2,
    radius: f32,
}

struct Bullet {
    position: Vector2,
    velocity: Vector2,
    lifetime: f32,
}

enum Msg {
    Start,
    ReturnToMenu,
    Controls { turn: f32, thrust: bool, fire: bool },
}

struct World {
    screen: Screen,
    player: Player,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,
    score: u32,
    wave: u32,
    random_state: u32,
}

impl Default for World {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            player: Player {
                position: Vector2::new(WIDTH / 2.0, HEIGHT / 2.0),
                velocity: Vector2::zero(),
                angle: 0.0,
                lives: 3,
                invulnerable: 0.0,
                shoot_cooldown: 0.0,
            },
            asteroids: Vec::new(),
            bullets: Vec::new(),
            score: 0,
            wave: 0,
            random_state: 0xA57E_2015,
        }
    }
}

impl World {
    fn random(&mut self) -> f32 {
        self.random_state = self
            .random_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.random_state >> 8) as f32 / (u32::MAX >> 8) as f32
    }

    fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.random() * (max - min)
    }

    fn reset(&mut self) {
        self.player = Player {
            position: Vector2::new(WIDTH / 2.0, HEIGHT / 2.0),
            velocity: Vector2::zero(),
            angle: 0.0,
            lives: 3,
            invulnerable: 2.0,
            shoot_cooldown: 0.0,
        };
        self.asteroids.clear();
        self.bullets.clear();
        self.score = 0;
        self.wave = 0;
        self.spawn_wave();
    }

    fn spawn_wave(&mut self) {
        self.wave += 1;
        let count = 3 + self.wave.min(5) as usize;
        for _ in 0..count {
            // Spawn near an edge so the player gets room to move at the start.
            let (x, y) = match self.random_range(0.0, 4.0) as u32 {
                0 => (self.random_range(24.0, WIDTH - 24.0), 28.0),
                1 => (WIDTH - 28.0, self.random_range(24.0, HEIGHT - 24.0)),
                2 => (self.random_range(24.0, WIDTH - 24.0), HEIGHT - 28.0),
                _ => (28.0, self.random_range(24.0, HEIGHT - 24.0)),
            };
            let angle = self.random_range(0.0, std::f32::consts::TAU);
            let speed = self.random_range(24.0, 68.0);
            let radius = self.random_range(24.0, 38.0);
            self.asteroids.push(Asteroid {
                position: Vector2::new(x, y),
                velocity: Vector2::new(angle.cos() * speed, angle.sin() * speed),
                radius,
            });
        }
    }

    fn wrap(position: &mut Vector2) {
        if position.x < 0.0 {
            position.x += WIDTH;
        } else if position.x >= WIDTH {
            position.x -= WIDTH;
        }
        if position.y < 0.0 {
            position.y += HEIGHT;
        } else if position.y >= HEIGHT {
            position.y -= HEIGHT;
        }
    }

    fn spawn_bullet(&mut self) {
        if self.player.shoot_cooldown > 0.0 {
            return;
        }
        let direction = Vector2::new(self.player.angle.sin(), -self.player.angle.cos());
        let position = Vector2::new(
            self.player.position.x + direction.x * PLAYER_RADIUS,
            self.player.position.y + direction.y * PLAYER_RADIUS,
        );
        self.bullets.push(Bullet {
            position,
            velocity: Vector2::new(
                self.player.velocity.x + direction.x * BULLET_SPEED,
                self.player.velocity.y + direction.y * BULLET_SPEED,
            ),
            lifetime: 1.2,
        });
        self.player.shoot_cooldown = 0.22;
    }

    fn update_playing(&mut self, dt: f32, turn: f32, thrust: bool, fire: bool) {
        self.player.angle += turn * 3.6 * dt;
        if thrust {
            self.player.velocity.x += self.player.angle.sin() * 190.0 * dt;
            self.player.velocity.y -= self.player.angle.cos() * 190.0 * dt;
        }
        let speed = self.player.velocity.x.hypot(self.player.velocity.y);
        if speed > 260.0 {
            let scale = 260.0 / speed;
            self.player.velocity.x *= scale;
            self.player.velocity.y *= scale;
        }
        self.player.position.x += self.player.velocity.x * dt;
        self.player.position.y += self.player.velocity.y * dt;
        Self::wrap(&mut self.player.position);
        self.player.invulnerable = (self.player.invulnerable - dt).max(0.0);
        self.player.shoot_cooldown = (self.player.shoot_cooldown - dt).max(0.0);
        if fire {
            self.spawn_bullet();
        }

        for asteroid in &mut self.asteroids {
            asteroid.position.x += asteroid.velocity.x * dt;
            asteroid.position.y += asteroid.velocity.y * dt;
            Self::wrap(&mut asteroid.position);
        }
        for bullet in &mut self.bullets {
            bullet.position.x += bullet.velocity.x * dt;
            bullet.position.y += bullet.velocity.y * dt;
            bullet.lifetime -= dt;
            Self::wrap(&mut bullet.position);
        }
        self.bullets.retain(|bullet| bullet.lifetime > 0.0);

        let mut remaining_bullets = vec![true; self.bullets.len()];
        let mut surviving_asteroids = Vec::with_capacity(self.asteroids.len());
        let mut fragments = Vec::new();
        for asteroid in std::mem::take(&mut self.asteroids) {
            let hit = self
                .bullets
                .iter()
                .enumerate()
                .find(|(index, bullet)| {
                    remaining_bullets[*index]
                        && distance_squared(asteroid.position, bullet.position)
                            < (asteroid.radius + 3.0).powi(2)
                })
                .map(|(index, _)| index);
            if let Some(index) = hit {
                remaining_bullets[index] = false;
                self.score += (50.0 / asteroid.radius) as u32;
                if asteroid.radius > 27.0 {
                    for direction in [-1.0_f32, 1.0] {
                        let angle = self.random_range(0.0, std::f32::consts::TAU);
                        let speed = self.random_range(45.0, 95.0);
                        fragments.push(Asteroid {
                            position: asteroid.position,
                            velocity: Vector2::new(
                                angle.cos() * speed + direction * 12.0,
                                angle.sin() * speed,
                            ),
                            radius: asteroid.radius * 0.58,
                        });
                    }
                }
            } else {
                surviving_asteroids.push(asteroid);
            }
        }
        self.bullets = self
            .bullets
            .drain(..)
            .enumerate()
            .filter_map(|(index, bullet)| remaining_bullets[index].then_some(bullet))
            .collect();
        surviving_asteroids.extend(fragments);
        self.asteroids = surviving_asteroids;

        if self.player.invulnerable == 0.0
            && self.asteroids.iter().any(|asteroid| {
                distance_squared(self.player.position, asteroid.position)
                    < (PLAYER_RADIUS + asteroid.radius * 0.75).powi(2)
            })
        {
            self.player.lives -= 1;
            if self.player.lives == 0 {
                self.screen = Screen::GameOver;
            } else {
                self.player.position = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);
                self.player.velocity = Vector2::zero();
                self.player.invulnerable = 2.0;
            }
        }

        if self.asteroids.is_empty() {
            self.spawn_wave();
        }
    }

    fn draw_player(&self, d: &mut RaylibDrawHandle) {
        use solarized as s;
        if self.player.invulnerable > 0.0 && (self.player.invulnerable * 10.0) as i32 % 2 == 0 {
            return;
        }
        let forward = Vector2::new(self.player.angle.sin(), -self.player.angle.cos());
        let side = Vector2::new(self.player.angle.cos(), self.player.angle.sin());
        let nose = Vector2::new(
            self.player.position.x + forward.x * PLAYER_RADIUS * 1.5,
            self.player.position.y + forward.y * PLAYER_RADIUS * 1.5,
        );
        let left = Vector2::new(
            self.player.position.x - forward.x * PLAYER_RADIUS + side.x * PLAYER_RADIUS,
            self.player.position.y - forward.y * PLAYER_RADIUS + side.y * PLAYER_RADIUS,
        );
        let right = Vector2::new(
            self.player.position.x - forward.x * PLAYER_RADIUS - side.x * PLAYER_RADIUS,
            self.player.position.y - forward.y * PLAYER_RADIUS - side.y * PLAYER_RADIUS,
        );
        d.draw_triangle(nose, left, right, s::BLUE);
    }
}

fn distance_squared(a: Vector2, b: Vector2) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

impl Component for World {
    type Msg = Msg;

    fn update(&mut self, dt: f32, messages: Vec<Msg>) {
        let mut controls = (0.0, false, false);
        for message in messages {
            match message {
                Msg::Start => {
                    self.reset();
                    self.screen = Screen::Playing;
                }
                Msg::ReturnToMenu => self.screen = Screen::Menu,
                Msg::Controls { turn, thrust, fire } => controls = (turn, thrust, fire),
            }
        }
        if self.screen == Screen::Playing {
            self.update_playing(dt, controls.0, controls.1, controls.2);
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) -> Vec<Msg> {
        use solarized as s;
        d.clear_background(s::BACKGROUND);
        let mut messages = Vec::new();

        match self.screen {
            Screen::Menu => {
                d.draw_text("ASTEROIDS", 48, 40, 48, s::FOREGROUND);
                d.draw_text(
                    "A tiny arcade game built with easy-ray",
                    52,
                    104,
                    20,
                    s::FOREGROUND2,
                );
                if d.gui_button(Rectangle::new(52.0, 160.0, 220.0, 48.0), "PLAY") {
                    messages.push(Msg::Start);
                }
                d.draw_text("A / D or arrows: turn", 52, 240, 18, s::FOREGROUND2);
                d.draw_text(
                    "W / Up: thrust     Space: fire",
                    52,
                    270,
                    18,
                    s::FOREGROUND2,
                );
                d.draw_text(
                    "Break large asteroids into smaller pieces.",
                    52,
                    310,
                    18,
                    s::FOREGROUND2,
                );
            }
            Screen::Playing | Screen::GameOver => {
                for asteroid in &self.asteroids {
                    d.draw_circle_v(asteroid.position, asteroid.radius, s::BASE2);
                    d.draw_circle_lines(
                        asteroid.position.x as i32,
                        asteroid.position.y as i32,
                        asteroid.radius,
                        s::BASE00,
                    );
                }
                for bullet in &self.bullets {
                    d.draw_circle_v(bullet.position, 3.0, s::ORANGE);
                }
                if self.screen == Screen::Playing {
                    self.draw_player(d);
                    if d.is_key_down(KeyboardKey::KEY_W) || d.is_key_down(KeyboardKey::KEY_UP) {
                        let tail = Vector2::new(
                            self.player.position.x - self.player.angle.sin() * 20.0,
                            self.player.position.y + self.player.angle.cos() * 20.0,
                        );
                        d.draw_circle_v(tail, 4.0, s::ORANGE);
                    }
                    let turn = (d.is_key_down(KeyboardKey::KEY_D)
                        || d.is_key_down(KeyboardKey::KEY_RIGHT))
                        as i32 as f32
                        - (d.is_key_down(KeyboardKey::KEY_A)
                            || d.is_key_down(KeyboardKey::KEY_LEFT))
                            as i32 as f32;
                    messages.push(Msg::Controls {
                        turn,
                        thrust: d.is_key_down(KeyboardKey::KEY_W)
                            || d.is_key_down(KeyboardKey::KEY_UP),
                        fire: d.is_key_down(KeyboardKey::KEY_SPACE),
                    });
                }
                d.draw_text(
                    &format!("SCORE  {:05}", self.score),
                    24,
                    22,
                    22,
                    s::FOREGROUND,
                );
                d.draw_text(&format!("WAVE  {}", self.wave), 24, 52, 18, s::FOREGROUND2);
                d.draw_text(
                    &format!("LIVES  {}", self.player.lives),
                    800,
                    22,
                    20,
                    s::FOREGROUND,
                );

                if self.screen == Screen::GameOver {
                    d.draw_rectangle(
                        0,
                        0,
                        WIDTH as i32,
                        HEIGHT as i32,
                        Color::new(253, 246, 227, 220),
                    );
                    d.draw_text("GAME OVER", 350, 230, 42, s::RED);
                    d.draw_text(
                        &format!("Final score: {}", self.score),
                        390,
                        290,
                        22,
                        s::FOREGROUND,
                    );
                    if d.gui_button(Rectangle::new(370.0, 340.0, 220.0, 46.0), "PLAY AGAIN") {
                        messages.push(Msg::Start);
                    }
                    if d.gui_button(Rectangle::new(370.0, 398.0, 220.0, 46.0), "MAIN MENU") {
                        messages.push(Msg::ReturnToMenu);
                    }
                }
            }
        }
        messages
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    World::run(Settings {
        title: Some("Asteroids — easy-ray"),
        width: WIDTH as u32,
        height: HEIGHT as u32,
        target_fps: 60,
        fullscreen: false,
    })
}
