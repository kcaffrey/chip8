extern crate chip8;
#[macro_use]
extern crate clap;
extern crate ggez;
extern crate rodio;

use std::fs;
use std::path::Path;
use std::process;
use std::time::Duration;

use chip8::{AudioHandler, Emulator};
use clap::{App, Arg};
use ggez::conf;
use ggez::error::GameError;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics::{self, Color, DrawMode, MeshBuilder, Point2};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use rodio::source::SineWave;
use rodio::{Device, Sink};

#[derive(Default)]
struct MainState {
    emulator: Emulator,
    clock_speed: u32,
}

fn main() {
    let matches = App::new("CHIP-8")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A CHIP-8 emulator written in Rust.")
        .arg(
            Arg::with_name("clock_speed")
                .short("c")
                .long("clock-speed")
                .value_name("hz")
                .default_value("1200")
                .help("Sets the clock speed (in hz) of the CPU.")
                .validator(validate_clock_speed)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("program")
                .required(true)
                .validator(validate_file_exists)
                .help("The CHIP-8 ROM to load."),
        )
        .get_matches();

    let filename = matches.value_of("program").unwrap();
    let rom = match fs::read(&filename) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let mut main_state = MainState {
        clock_speed: value_t!(matches, "clock_speed", u32).unwrap(),
        ..Default::default()
    };
    match main_state.emulator.load_program(rom.as_slice()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(2);
        }
    }

    if let Some(device) = rodio::default_output_device() {
        main_state
            .emulator
            .set_audio_handler(Box::new(SimpleAudio::with_device(device)));
    } else {
        eprintln!("Could not open audio output device.");
    }

    let cb = ContextBuilder::new("chip8", "kevin")
        .window_setup(conf::WindowSetup::default().title("CHIP-8"))
        .window_mode(conf::WindowMode::default().dimensions(640, 320));
    let ctx = &mut cb.build().unwrap();

    event::run(ctx, &mut main_state).unwrap();
}

fn validate_clock_speed(v: String) -> Result<(), String> {
    match v.parse::<u32>() {
        Ok(n) if n < 60 => Err(format!("clock speed must be at least 60 Hz, got {} Hz", n)),
        Ok(n) if n > 1024 * 120 => Err(format!(
            "maximum clock speed is 120 KHz, got {}",
            format_hz(n)
        )),
        Ok(_) => Ok(()),
        _ => Err(format!("expecting a number, got '{}'", v)),
    }
}

fn format_hz(hz: u32) -> String {
    match hz {
        n if n >= 1024 * 1024 * 1024 => format!("{:.1} GHz", n as f32 / 1024.0 / 1024.0 / 1024.0),
        n if n >= 1024 * 1024 => format!("{:.1} MHz", n as f32 / 1024.0 / 1024.0),
        n if n >= 1024 => format!("{:.1} KHz", n as f32 / 1024.0),
        n => format!("{} Hz", n),
    }
}

fn validate_file_exists(f: String) -> Result<(), String> {
    if Path::new(&f).exists() {
        Ok(())
    } else {
        Err(format!("`{}` doesn't exist", f))
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut delta = timer::get_delta(ctx);
        while timer::check_update_time(ctx, self.clock_speed) {
            match self.emulator.execute_cycle(delta) {
                Ok(_) => delta = Duration::from_secs(0),
                Err(e) => return Err(GameError::from(e.0)),
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::set_color(ctx, Color::from_rgb(246, 234, 190))?;

        let mut mesh = MeshBuilder::new();
        for (y, row) in self.emulator.get_display().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if *cell {
                    let (x0, y0) = (10.0 * x as f32, 10.0 * y as f32);
                    let (x1, y1) = (x0 + 10.0, y0 + 10.0);
                    mesh.polygon(
                        DrawMode::Fill,
                        &[
                            Point2::new(x0, y0),
                            Point2::new(x0, y1),
                            Point2::new(x1, y1),
                            Point2::new(x1, y0),
                            Point2::new(x0, y0),
                        ],
                    );
                }
            }
        }
        let mesh = mesh.build(ctx)?;
        graphics::draw(ctx, &mesh, Point2::new(0.0, 0.0), 0.0)?;

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: Keycode, _keymod: Mod, _repeat: bool) {
        match key {
            Keycode::Escape => ctx.quit().unwrap(),
            k => {
                if let Some(k) = keypad_key_from_keycode(k) {
                    self.emulator.on_key_down(k);
                }
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, key: Keycode, _keymod: Mod, _repeat: bool) {
        if let Some(key) = keypad_key_from_keycode(key) {
            self.emulator.on_key_up(key);
        }
    }
}

fn keypad_key_from_keycode(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Num1 => Some(1),
        Keycode::Num2 => Some(2),
        Keycode::Num3 => Some(3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(4),
        Keycode::W => Some(5),
        Keycode::E => Some(6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(7),
        Keycode::S => Some(8),
        Keycode::D => Some(9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

struct SimpleAudio {
    device: Device,
    sink: Option<Sink>,
}

impl SimpleAudio {
    fn with_device(device: Device) -> SimpleAudio {
        SimpleAudio { device, sink: None }
    }
}

impl AudioHandler for SimpleAudio {
    fn start_sound(&mut self) {
        if self.sink.is_none() {
            let sink = Sink::new(&self.device);
            sink.append(SineWave::new(440));
            self.sink = Some(sink);
        }
    }

    fn stop_sound(&mut self) {
        self.sink.take();
    }
}
