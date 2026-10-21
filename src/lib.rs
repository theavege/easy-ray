pub mod prelude {
    pub use raylib::prelude::*;
    pub use std::error::Error;

    pub mod solarized {
        use raylib::prelude::Color;
        pub const BASE03: Color = Color::new(0x00, 0x2B, 0x36, 0xFF);
        pub const BASE02: Color = Color::new(0x07, 0x36, 0x42, 0xFF);
        pub const BASE01: Color = Color::new(0x58, 0x6E, 0x75, 0xFF);
        pub const BASE00: Color = Color::new(0x65, 0x7B, 0x83, 0xFF);
        pub const BASE0: Color = Color::new(0x83, 0x94, 0x96, 0xFF);
        pub const BASE1: Color = Color::new(0x93, 0xA1, 0xA1, 0xFF);
        pub const BASE2: Color = Color::new(0xEE, 0xE8, 0xD5, 0xFF);
        pub const BASE3: Color = Color::new(0xFD, 0xF6, 0xE3, 0xFF);
        pub const YELLOW: Color = Color::new(181, 137, 0, 255);
        pub const ORANGE: Color = Color::new(0xCB, 0x4B, 0x16, 0xFF);
        pub const RED: Color = Color::new(220, 50, 47, 255);
        pub const CYAN: Color = Color::new(42, 161, 152, 255);
        pub const MAGENTA: Color = Color::new(211, 54, 130, 255);
        pub const GREEN: Color = Color::new(133, 153, 0, 255);
        pub const BLUE: Color = Color::new(0x26, 0x8B, 0xD2, 0xFF);
        pub const VIOLET: Color = Color::new(0x6C, 0x71, 0xC4, 0xFF);
        // Solarized Light semantic colors.
        pub const FOREGROUND: Color = BASE00;
        pub const FOREGROUND2: Color = BASE01;
        pub const BACKGROUND: Color = BASE3;
        pub const BACKGROUND2: Color = BASE2;
    }

    pub struct Settings {
        pub title: Option<&'static str>,
        pub width: u32,
        pub height: u32,
        pub target_fps: u32,
        pub fullscreen: bool,
    }
    impl Default for Settings {
        fn default() -> Self {
            Self {
                title: None,
                width: 960,
                height: 640,
                target_fps: 60,
                fullscreen: true,
            }
        }
    }

    pub trait Component: Default {
        type Msg: 'static;
        fn load(&mut self, _path: &str) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn exit(&mut self, _path: &str) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        fn update(&mut self, dt: f32, messages: Vec<Self::Msg>);
        fn draw(&self, d: &mut RaylibDrawHandle) -> Vec<Self::Msg>;
        fn run(settings: Settings) -> Result<(), Box<dyn Error>> {
            let mut world = Self::default();
            if settings.title.is_none() {
                return Err("title must be a non-empty file name".into());
            }
            world.load(settings.title.unwrap())?;
            if settings.width == 0 || settings.height == 0 || settings.target_fps == 0 {
                return Err("window dimensions and target FPS must be positive".into());
            }
            let (mut rl, thread) = raylib::init()
                .size(settings.width as i32, settings.height as i32)
                .title(settings.title.unwrap())
                .build();
            rl.set_target_fps(settings.target_fps);
            if settings.fullscreen {
                rl.toggle_fullscreen();
            }
            while !rl.window_should_close() {
                let messages = world.draw(&mut rl.begin_drawing(&thread));
                let dt = rl.get_frame_time();
                world.update(dt, messages);
            }
            world.exit(settings.title.unwrap())?;
            Ok(())
        }
    }
}
