use easy_ray::prelude::*;

const SIZE: i32 = 18;
const STEP_TIME: f32 = 0.14;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Menu,
    Playing,
    GameOver,
}

enum Msg {
    Start,
    Turn(Direction),
    Menu,
}

struct World {
    screen: Screen,
    snake: Vec<(i32, i32)>,
    direction: Direction,
    next_direction: Direction,
    food: (i32, i32),
    elapsed: f32,
    score: u32,
    seed: u32,
    camera: Camera3D,
}

impl Default for World {
    fn default() -> Self {
        let center = SIZE as f32 / 2.0 - 0.5;
        let mut world = Self {
            screen: Screen::Menu,
            snake: Vec::new(),
            direction: Direction::Right,
            next_direction: Direction::Right,
            food: (SIZE / 2 + 3, SIZE / 2),
            elapsed: 0.0,
            score: 0,
            seed: 0,
            camera: Camera3D::perspective(
                Vector3::new(center + 12.0, 17.0, center + 15.0),
                Vector3::new(center, 0.0, center),
                Vector3::new(0.0, 1.0, 0.0),
                45.0,
            ),
        };
        world.reset();
        world.screen = Screen::Menu;
        world
    }
}

impl World {
    fn reset(&mut self) {
        let mid = SIZE / 2;
        self.snake = vec![(mid, mid), (mid - 1, mid), (mid - 2, mid)];
        self.direction = Direction::Right;
        self.next_direction = Direction::Right;
        self.elapsed = 0.0;
        self.score = 0;
        self.seed = self.seed.wrapping_add(1);
        self.place_food();
    }

    fn place_food(&mut self) {
        for _ in 0..SIZE * SIZE {
            self.seed = self
                .seed
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            let cell = (self.seed >> 8) as i32 % (SIZE * SIZE);
            let candidate = (cell % SIZE, cell / SIZE);
            if !self.snake.contains(&candidate) {
                self.food = candidate;
                return;
            }
        }
    }

    fn turn(&mut self, direction: Direction) {
        let opposite = matches!(
            (self.direction, direction),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );
        if !opposite {
            self.next_direction = direction;
        }
    }

    fn step(&mut self) {
        self.direction = self.next_direction;
        let (x, z) = self.snake[0];
        let head = match self.direction {
            Direction::Up => (x, z - 1),
            Direction::Down => (x, z + 1),
            Direction::Left => (x - 1, z),
            Direction::Right => (x + 1, z),
        };
        let ate = head == self.food;
        let body_len = self.snake.len() - usize::from(!ate);
        if head.0 < 0
            || head.0 >= SIZE
            || head.1 < 0
            || head.1 >= SIZE
            || self.snake[..body_len].contains(&head)
        {
            self.screen = Screen::GameOver;
            return;
        }
        self.snake.insert(0, head);
        if ate {
            self.score += 1;
            self.place_food();
        } else {
            self.snake.pop();
        }
    }

    fn cell_position(cell: (i32, i32)) -> Vector3 {
        Vector3::new(cell.0 as f32, 0.48, cell.1 as f32)
    }
}

impl Component for World {
    type Msg = Msg;

    fn update(&mut self, dt: f32, messages: Vec<Msg>) {
        for message in messages {
            match message {
                Msg::Start => {
                    self.reset();
                    self.screen = Screen::Playing;
                }
                Msg::Turn(direction) if self.screen == Screen::Playing => self.turn(direction),
                Msg::Menu => self.screen = Screen::Menu,
                _ => {}
            }
        }

        if self.screen == Screen::Playing {
            self.elapsed += dt;
            while self.elapsed >= STEP_TIME && self.screen == Screen::Playing {
                self.elapsed -= STEP_TIME;
                self.step();
            }
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) -> Vec<Msg> {
        use solarized as s;
        d.clear_background(s::BACKGROUND);

        {
            let mut scene = d.begin_mode3D(self.camera);
            let center = (SIZE as f32 - 1.0) / 2.0;
            scene.draw_cube(
                Vector3::new(center, -0.12, center),
                SIZE as f32,
                0.2,
                SIZE as f32,
                s::BACKGROUND2,
            );
            scene.draw_grid(SIZE, 1.0);

            if matches!(self.screen, Screen::Playing | Screen::GameOver) {
                scene.draw_cube(Self::cell_position(self.food), 0.72, 0.72, 0.72, s::RED);
                for (index, &cell) in self.snake.iter().enumerate() {
                    scene.draw_cube(
                        Self::cell_position(cell),
                        0.82,
                        0.82,
                        0.82,
                        if index == 0 { s::BLUE } else { s::GREEN },
                    );
                }
            }
        }

        d.draw_text("S N A K E  3D", 32, 24, 30, s::FOREGROUND);
        let mut messages = Vec::new();
        match self.screen {
            Screen::Menu => {
                d.draw_text("Solarized Light edition", 36, 66, 18, s::FOREGROUND2);
                d.draw_text(
                    "Eat the coral fruit and avoid the walls and your tail.",
                    36,
                    96,
                    18,
                    s::FOREGROUND2,
                );
                if d.gui_button(Rectangle::new(36.0, 136.0, 190.0, 46.0), "PLAY") {
                    messages.push(Msg::Start);
                }
                d.draw_text("Move with arrow keys or WASD", 36, 198, 18, s::FOREGROUND2);
            }
            Screen::Playing => {
                d.draw_text(
                    &format!("SCORE  {:02}", self.score),
                    770,
                    28,
                    20,
                    s::FOREGROUND,
                );
                if d.is_key_pressed(KeyboardKey::KEY_UP) || d.is_key_pressed(KeyboardKey::KEY_W) {
                    messages.push(Msg::Turn(Direction::Up));
                }
                if d.is_key_pressed(KeyboardKey::KEY_DOWN) || d.is_key_pressed(KeyboardKey::KEY_S) {
                    messages.push(Msg::Turn(Direction::Down));
                }
                if d.is_key_pressed(KeyboardKey::KEY_LEFT) || d.is_key_pressed(KeyboardKey::KEY_A) {
                    messages.push(Msg::Turn(Direction::Left));
                }
                if d.is_key_pressed(KeyboardKey::KEY_RIGHT) || d.is_key_pressed(KeyboardKey::KEY_D)
                {
                    messages.push(Msg::Turn(Direction::Right));
                }
            }
            Screen::GameOver => {
                d.draw_text("GAME OVER", 36, 78, 30, s::RED);
                d.draw_text(
                    &format!("Final score: {}", self.score),
                    36,
                    118,
                    20,
                    s::FOREGROUND,
                );
                if d.gui_button(Rectangle::new(36.0, 158.0, 190.0, 44.0), "PLAY AGAIN") {
                    messages.push(Msg::Start);
                }
                if d.gui_button(Rectangle::new(36.0, 214.0, 190.0, 44.0), "MAIN MENU") {
                    messages.push(Msg::Menu);
                }
            }
        }
        messages
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    World::run(Settings {
        title: Some("Snake 3D — easy-ray"),
        width: 960,
        height: 640,
        target_fps: 60,
        fullscreen: false,
    })
}
