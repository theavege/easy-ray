use easy_ray::prelude::*;
use hecs::{Entity, World as EntityWorld};

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

// Components: entities are assembled from these small, independent pieces.
#[derive(Clone, Copy)]
struct Position(Vector2);

#[derive(Clone, Copy)]
struct Velocity(Vector2);

struct Ship {
    angle: f32,
    lives: u32,
    invulnerable: f32,
    shoot_cooldown: f32,
}

struct Asteroid {
    radius: f32,
}

struct Bullet {
    lifetime: f32,
}

enum Msg {
    Start,
    ReturnToMenu,
    Controls { turn: f32, thrust: bool, fire: bool },
}

struct World {
    entities: EntityWorld,
    player: Entity,
    screen: Screen,
    score: u32,
    wave: u32,
    random_state: u32,
}

impl Default for World {
    fn default() -> Self {
        let mut entities = EntityWorld::new();
        let player = entities.spawn((
            Position(Vector2::new(WIDTH / 2.0, HEIGHT / 2.0)),
            Velocity(Vector2::zero()),
            Ship {
                angle: 0.0,
                lives: 3,
                invulnerable: 0.0,
                shoot_cooldown: 0.0,
            },
        ));
        Self {
            entities,
            player,
            screen: Screen::Menu,
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
        let mut remove = Vec::new();
        remove.extend(
            self.entities
                .query::<(Entity, &Asteroid)>()
                .iter()
                .map(|(entity, _)| entity),
        );
        remove.extend(
            self.entities
                .query::<(Entity, &Bullet)>()
                .iter()
                .map(|(entity, _)| entity),
        );
        for entity in remove {
            let _ = self.entities.despawn(entity);
        }
        for (entity, position, velocity, ship) in
            self.entities
                .query_mut::<(Entity, &mut Position, &mut Velocity, &mut Ship)>()
        {
            if entity == self.player {
                position.0 = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);
                velocity.0 = Vector2::zero();
                ship.angle = 0.0;
                ship.lives = 3;
                ship.invulnerable = 2.0;
                ship.shoot_cooldown = 0.0;
            }
        }
        self.score = 0;
        self.wave = 0;
        self.spawn_wave();
    }

    fn spawn_wave(&mut self) {
        self.wave += 1;
        let count = 3 + self.wave.min(5) as usize;
        for _ in 0..count {
            let (x, y) = match self.random_range(0.0, 4.0) as u32 {
                0 => (self.random_range(24.0, WIDTH - 24.0), 28.0),
                1 => (WIDTH - 28.0, self.random_range(24.0, HEIGHT - 24.0)),
                2 => (self.random_range(24.0, WIDTH - 24.0), HEIGHT - 28.0),
                _ => (28.0, self.random_range(24.0, HEIGHT - 24.0)),
            };
            let angle = self.random_range(0.0, std::f32::consts::TAU);
            let speed = self.random_range(24.0, 68.0);
            let radius = self.random_range(24.0, 38.0);
            self.entities.spawn((
                Position(Vector2::new(x, y)),
                Velocity(Vector2::new(angle.cos() * speed, angle.sin() * speed)),
                Asteroid { radius },
            ));
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
        let mut shot = None;
        for (entity, position, velocity, ship) in
            self.entities
                .query_mut::<(Entity, &mut Position, &mut Velocity, &mut Ship)>()
        {
            if entity == self.player && ship.shoot_cooldown <= 0.0 {
                let direction = Vector2::new(ship.angle.sin(), -ship.angle.cos());
                shot = Some((
                    Vector2::new(
                        position.0.x + direction.x * PLAYER_RADIUS,
                        position.0.y + direction.y * PLAYER_RADIUS,
                    ),
                    Vector2::new(
                        velocity.0.x + direction.x * BULLET_SPEED,
                        velocity.0.y + direction.y * BULLET_SPEED,
                    ),
                ));
                ship.shoot_cooldown = 0.22;
            }
        }
        if let Some((position, velocity)) = shot {
            self.entities.spawn((
                Position(position),
                Velocity(velocity),
                Bullet { lifetime: 1.2 },
            ));
        }
    }

    fn update_playing(&mut self, dt: f32, turn: f32, thrust: bool, fire: bool) {
        for (entity, velocity, ship) in self
            .entities
            .query_mut::<(Entity, &mut Velocity, &mut Ship)>()
        {
            if entity == self.player {
                ship.angle += turn * 3.6 * dt;
                if thrust {
                    velocity.0.x += ship.angle.sin() * 190.0 * dt;
                    velocity.0.y -= ship.angle.cos() * 190.0 * dt;
                }
                let speed = velocity.0.x.hypot(velocity.0.y);
                if speed > 260.0 {
                    let scale = 260.0 / speed;
                    velocity.0.x *= scale;
                    velocity.0.y *= scale;
                }
                ship.invulnerable = (ship.invulnerable - dt).max(0.0);
                ship.shoot_cooldown = (ship.shoot_cooldown - dt).max(0.0);
            }
        }
        for (position, velocity) in self.entities.query_mut::<(&mut Position, &Velocity)>() {
            position.0.x += velocity.0.x * dt;
            position.0.y += velocity.0.y * dt;
            Self::wrap(&mut position.0);
        }

        if fire {
            self.spawn_bullet();
        }

        let mut expired_bullets = Vec::new();
        for (entity, bullet) in self.entities.query_mut::<(Entity, &mut Bullet)>() {
            bullet.lifetime -= dt;
            if bullet.lifetime <= 0.0 {
                expired_bullets.push(entity);
            }
        }
        for entity in expired_bullets {
            let _ = self.entities.despawn(entity);
        }

        // Snapshot the two groups so collision handling can safely despawn,
        // score, and create split asteroids after the queries finish.
        let asteroids: Vec<(Entity, Vector2, f32)> = self
            .entities
            .query::<(Entity, &Position, &Asteroid)>()
            .iter()
            .map(|(entity, position, asteroid)| (entity, position.0, asteroid.radius))
            .collect();
        let bullets: Vec<(Entity, Vector2)> = self
            .entities
            .query::<(Entity, &Position, &Bullet)>()
            .iter()
            .map(|(entity, position, _)| (entity, position.0))
            .collect();
        let mut consumed_bullets = vec![false; bullets.len()];
        let mut removed_asteroids = Vec::new();
        let mut removed_bullet_entities = Vec::new();
        let mut fragments = Vec::new();
        for &(asteroid_entity, asteroid_position, radius) in &asteroids {
            let hit = bullets
                .iter()
                .enumerate()
                .find(|(index, (_, bullet_position))| {
                    !consumed_bullets[*index]
                        && distance_squared(asteroid_position, *bullet_position)
                            < (radius + 3.0).powi(2)
                });
            if let Some((index, (bullet_entity, _))) = hit {
                consumed_bullets[index] = true;
                removed_asteroids.push(asteroid_entity);
                removed_bullet_entities.push(*bullet_entity);
                self.score += (50.0 / radius) as u32;
                if radius > 27.0 {
                    for direction in [-1.0_f32, 1.0] {
                        let angle = self.random_range(0.0, std::f32::consts::TAU);
                        let speed = self.random_range(45.0, 95.0);
                        fragments.push((
                            asteroid_position,
                            Vector2::new(
                                angle.cos() * speed + direction * 12.0,
                                angle.sin() * speed,
                            ),
                            radius * 0.58,
                        ));
                    }
                }
            }
        }
        for entity in removed_asteroids.into_iter().chain(removed_bullet_entities) {
            let _ = self.entities.despawn(entity);
        }
        for (position, velocity, radius) in fragments {
            self.entities
                .spawn((Position(position), Velocity(velocity), Asteroid { radius }));
        }

        let mut player_data = None;
        for (entity, position, ship) in self.entities.query::<(Entity, &Position, &Ship)>().iter() {
            if entity == self.player {
                player_data = Some((position.0, ship.invulnerable));
            }
        }
        if let Some((player_position, invulnerable)) = player_data {
            if invulnerable <= 0.0 {
                let collision = self
                    .entities
                    .query::<(Entity, &Position, &Asteroid)>()
                    .iter()
                    .any(|(_, position, asteroid)| {
                        distance_squared(player_position, position.0)
                            < (PLAYER_RADIUS + asteroid.radius * 0.75).powi(2)
                    });
                if collision {
                    for (entity, ship) in self.entities.query_mut::<(Entity, &mut Ship)>() {
                        if entity == self.player {
                            ship.lives -= 1;
                            ship.invulnerable = 2.0;
                            if ship.lives == 0 {
                                self.screen = Screen::GameOver;
                            }
                        }
                    }
                    for (entity, position, velocity, _ship) in
                        self.entities
                            .query_mut::<(Entity, &mut Position, &mut Velocity, &Ship)>()
                    {
                        if entity == self.player {
                            position.0 = Vector2::new(WIDTH / 2.0, HEIGHT / 2.0);
                            velocity.0 = Vector2::zero();
                        }
                    }
                }
            }
        }

        if self
            .entities
            .query::<(Entity, &Asteroid)>()
            .iter()
            .next()
            .is_none()
        {
            self.spawn_wave();
        }
    }

    fn draw_player(&self, d: &mut RaylibDrawHandle) {
        use solarized as s;
        for (entity, position, ship) in self.entities.query::<(Entity, &Position, &Ship)>().iter() {
            if entity != self.player
                || (ship.invulnerable > 0.0 && (ship.invulnerable * 10.0) as i32 % 2 == 0)
            {
                continue;
            }
            let forward = Vector2::new(ship.angle.sin(), -ship.angle.cos());
            let side = Vector2::new(ship.angle.cos(), ship.angle.sin());
            let nose = Vector2::new(
                position.0.x + forward.x * PLAYER_RADIUS * 1.5,
                position.0.y + forward.y * PLAYER_RADIUS * 1.5,
            );
            let left = Vector2::new(
                position.0.x - forward.x * PLAYER_RADIUS + side.x * PLAYER_RADIUS,
                position.0.y - forward.y * PLAYER_RADIUS + side.y * PLAYER_RADIUS,
            );
            let right = Vector2::new(
                position.0.x - forward.x * PLAYER_RADIUS - side.x * PLAYER_RADIUS,
                position.0.y - forward.y * PLAYER_RADIUS - side.y * PLAYER_RADIUS,
            );
            d.draw_triangle(nose, left, right, s::BLUE);
        }
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
                d.draw_text("ASTEROIDS ECS", 48, 40, 46, s::FOREGROUND);
                d.draw_text(
                    "hecs edition: entities are built from components",
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
                    "Split large rocks, survive each wave.",
                    52,
                    310,
                    18,
                    s::FOREGROUND2,
                );
            }
            Screen::Playing | Screen::GameOver => {
                for (_, position, asteroid) in self
                    .entities
                    .query::<(Entity, &Position, &Asteroid)>()
                    .iter()
                {
                    d.draw_circle_v(position.0, asteroid.radius, s::BASE2);
                    d.draw_circle_lines(
                        position.0.x as i32,
                        position.0.y as i32,
                        asteroid.radius,
                        s::BASE00,
                    );
                }
                for (_, position, _bullet) in
                    self.entities.query::<(Entity, &Position, &Bullet)>().iter()
                {
                    d.draw_circle_v(position.0, 3.0, s::ORANGE);
                }
                if self.screen == Screen::Playing {
                    self.draw_player(d);
                    let mut controls = (0.0, false, false);
                    controls.0 += (d.is_key_down(KeyboardKey::KEY_D)
                        || d.is_key_down(KeyboardKey::KEY_RIGHT))
                        as i32 as f32;
                    controls.0 -= (d.is_key_down(KeyboardKey::KEY_A)
                        || d.is_key_down(KeyboardKey::KEY_LEFT))
                        as i32 as f32;
                    controls.1 =
                        d.is_key_down(KeyboardKey::KEY_W) || d.is_key_down(KeyboardKey::KEY_UP);
                    controls.2 = d.is_key_down(KeyboardKey::KEY_SPACE);
                    messages.push(Msg::Controls {
                        turn: controls.0,
                        thrust: controls.1,
                        fire: controls.2,
                    });
                }
                let mut lives = 0;
                for (entity, ship) in self.entities.query::<(Entity, &Ship)>().iter() {
                    if entity == self.player {
                        lives = ship.lives;
                    }
                }
                d.draw_text(
                    &format!("SCORE  {:05}", self.score),
                    24,
                    22,
                    22,
                    s::FOREGROUND,
                );
                d.draw_text(&format!("WAVE  {}", self.wave), 24, 52, 18, s::FOREGROUND2);
                d.draw_text(&format!("LIVES  {lives}"), 800, 22, 20, s::FOREGROUND);
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
        title: Some("Asteroids ECS — easy-ray"),
        width: WIDTH as u32,
        height: HEIGHT as u32,
        target_fps: 60,
        fullscreen: false,
    })
}
