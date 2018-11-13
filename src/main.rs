extern crate chip8;

use chip8::{Emulator, Result};
use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    let mut emulator: Emulator = Default::default();
    let mut timer = FixedRateTimer::default();
    let program = [0u8; 8];

    game_loop(&mut emulator, &program, &mut timer)
        .unwrap_or_else(|err| eprintln!("encountered error: {:?}", err));
}

fn game_loop(emulator: &mut Emulator, program: &[u8], timer: &mut FixedRateTimer) -> Result {
    emulator.load_program(&program)?;
    loop {
        while timer.should_tick() {
            emulator.execute_cycle()?;
        }
        thread::sleep(timer.sleep_time());
    }
}

#[derive(Debug)]
struct FixedRateTimer {
    last_check_time: SystemTime,
    accumulator: Duration,
    time_between_cycles: Duration,
}

impl Default for FixedRateTimer {
    fn default() -> FixedRateTimer {
        FixedRateTimer::new(60)
    }
}

impl FixedRateTimer {
    fn new(cycles_per_second: u32) -> FixedRateTimer {
        assert!(cycles_per_second > 0);
        assert!(cycles_per_second < 5000);
        FixedRateTimer {
            last_check_time: SystemTime::now(),
            accumulator: Duration::from_millis(0),
            time_between_cycles: Duration::from_micros(
                (1_000_000.0 / f64::from(cycles_per_second)).round() as u64,
            ),
        }
    }

    fn should_tick(&mut self) -> bool {
        let now = SystemTime::now();
        let elapsed = now
            .duration_since(self.last_check_time)
            .unwrap_or(Duration::from_millis(0));
        self.last_check_time = now;
        self.accumulator += elapsed;
        if self.accumulator >= self.time_between_cycles {
            self.accumulator -= self.time_between_cycles;
            true
        } else {
            false
        }
    }

    fn sleep_time(&self) -> Duration {
        if self.accumulator >= self.time_between_cycles {
            Duration::from_millis(0)
        } else {
            (self.time_between_cycles - self.accumulator) * 2 / 3
        }
    }
}
