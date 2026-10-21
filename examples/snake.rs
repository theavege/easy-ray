use easy_ray::prelude::*;

const CELL: i32 = 24;
const COLS: i32 = 36;
const ROWS: i32 = 22;
const BOARD_X: i32 = 48;
const BOARD_Y: i32 = 72;
const STEP_TIME: f32 = 0.12;

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
    Direction(Direction),
    BackToMenu,
}

struct World {
    screen: Screen,
    snake: Vec<(i32, i32)>,
    direction: Direction,
    queued_direction: Direction,
    food: (i32, i32),
    elapsed: f32,
    score: u32,
    food_seed: u32,
}

impl Default for World {
    fn default() -> Self {
        let mut world = Self {
            screen: Screen::Menu,
            snake: Vec::new(),
            direction: Direction::Right,
            queued_direction: Direction::Right,
            food: (COLS / 2 + 5, ROWS / 2),
            elapsed: 0.0,
            score: 0,
            food_seed: 0,
        };
        world.reset();
        world.screen = Screen::Menu;
        world
    }
}

impl World {
    fn reset(&mut self) {
        self.snake = vec![
            (COLS / 2, ROWS / 2),
            (COLS / 2 - 1, ROWS / 2),
            (COLS / 2 - 2, ROWS / 2),
        ];
        self.direction = Direction::Right;
        self.queued_direction = Direction::Right;
        self.score = 0;
        self.elapsed = 0.0;
        self.food_seed = self.food_seed.wrapping_add(1);
        self.place_food();
    }

    fn place_food(&mut self) {
        // A tiny deterministic sequence keeps the example dependency-free.
        for _ in 0..(COLS * ROWS) {
            self.food_seed = self
                .food_seed
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            let cell = ((self.food_seed >> 8) as i32 % (COLS * ROWS)).abs();
            let candidate = (cell % COLS, cell / COLS);
            if !self.snake.contains(&candidate) {
                self.food = candidate;
                return;
            }
        }
    }

    fn step(&mut self) {
        self.direction = self.queued_direction;
        let (x, y) = self.snake[0];
        let head = match self.direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };
        let ate = head == self.food;
        let body_len = self.snake.len() - usize::from(!ate);
        if head.0 < 0
            || head.0 >= COLS
            || head.1 < 0
            || head.1 >= ROWS
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

    fn set_direction(&mut self, next: Direction) {
        let opposite = matches!(
            (self.direction, next),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );
        if !opposite {
            self.queued_direction = next;
        }
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
                Msg::Direction(direction) if self.screen == Screen::Playing => {
                    self.set_direction(direction)
                }
                Msg::BackToMenu => self.screen = Screen::Menu,
                _ => {}
            }
        }
        if self.screen == Screen::Playing {
            self.elapsed += dt;
            if self.elapsed >= STEP_TIME {
                self.elapsed %= STEP_TIME;
                self.step();
            }
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) -> Vec<Msg> {
        use solarized as s;
        d.clear_background(s::BACKGROUND);
        d.draw_text("S N A K E", 48, 22, 32, s::FOREGROUND);

        let mut messages = Vec::new();
        match self.screen {
            Screen::Menu => {
                d.draw_text(
                    "A small game built with easy-ray",
                    50,
                    84,
                    20,
                    s::FOREGROUND2,
                );
                if d.gui_button(Rectangle::new(50.0, 140.0, 220.0, 48.0), "PLAY") {
                    messages.push(Msg::Start);
                }
                d.draw_text("Move with arrow keys or WASD", 50, 212, 18, s::FOREGROUND2);
                d.draw_text(
                    "Eat the coral fruit. Avoid walls and yourself.",
                    50,
                    240,
                    18,
                    s::FOREGROUND2,
                );
            }
            Screen::Playing | Screen::GameOver => {
                d.draw_text(
                    &format!("SCORE  {:02}", self.score),
                    650,
                    30,
                    20,
                    s::FOREGROUND,
                );
                let board = Rectangle::new(
                    BOARD_X as f32,
                    BOARD_Y as f32,
                    (COLS * CELL) as f32,
                    (ROWS * CELL) as f32,
                );
                d.draw_rectangle_rec(board, s::BACKGROUND2);
                d.draw_rectangle_lines_ex(board, 2.0, s::BASE1);
                let food = Rectangle::new(
                    (BOARD_X + self.food.0 * CELL + 4) as f32,
                    (BOARD_Y + self.food.1 * CELL + 4) as f32,
                    16.0,
                    16.0,
                );
                d.draw_rectangle_rounded(food, 0.35, 6, s::RED);
                for (index, &(x, y)) in self.snake.iter().enumerate() {
                    let cell = Rectangle::new(
                        (BOARD_X + x * CELL + 2) as f32,
                        (BOARD_Y + y * CELL + 2) as f32,
                        20.0,
                        20.0,
                    );
                    d.draw_rectangle_rounded(
                        cell,
                        0.28,
                        6,
                        if index == 0 { s::BLUE } else { s::GREEN },
                    );
                }
                if d.is_key_pressed(KeyboardKey::KEY_UP) || d.is_key_pressed(KeyboardKey::KEY_W) {
                    messages.push(Msg::Direction(Direction::Up));
                }
                if d.is_key_pressed(KeyboardKey::KEY_DOWN) || d.is_key_pressed(KeyboardKey::KEY_S) {
                    messages.push(Msg::Direction(Direction::Down));
                }
                if d.is_key_pressed(KeyboardKey::KEY_LEFT) || d.is_key_pressed(KeyboardKey::KEY_A) {
                    messages.push(Msg::Direction(Direction::Left));
                }
                if d.is_key_pressed(KeyboardKey::KEY_RIGHT) || d.is_key_pressed(KeyboardKey::KEY_D)
                {
                    messages.push(Msg::Direction(Direction::Right));
                }
                if self.screen == Screen::GameOver {
                    d.draw_rectangle(
                        BOARD_X,
                        250,
                        COLS * CELL,
                        150,
                        Color::new(253, 246, 227, 230),
                    );
                    d.draw_text("GAME OVER", 335, 276, 32, s::RED);
                    d.draw_text(
                        &format!("Final score: {}", self.score),
                        370,
                        322,
                        20,
                        s::FOREGROUND,
                    );
                    if d.gui_button(Rectangle::new(368.0, 360.0, 180.0, 40.0), "PLAY AGAIN") {
                        messages.push(Msg::Start);
                    }
                    if d.gui_button(Rectangle::new(558.0, 360.0, 150.0, 40.0), "MAIN MENU") {
                        messages.push(Msg::BackToMenu);
                    }
                }
            }
        }
        messages
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    World::run(Settings {
        title: Some("Snake — easy-ray"),
        width: 960,
        height: 640,
        target_fps: 60,
        fullscreen: false,
    })
}
