//! Play songs through a piezo-electric buzzer plugged on GPIO6.
//!
//! This assumes that a piezo-electric buzzer is connected to the pin assigned
//! to `buzzer`. (GPIO6)

#![no_std]
#![no_main]

use esp32h2_hal::{
    clock::ClockControl,
    gpio::IO,
    ledc::{channel, timer, LSGlobalClkSource, LEDC},
    peripherals::Peripherals,
    prelude::*,
};
use esp_backtrace as _;
use esp_hal_buzzer::{songs, Buzzer, VolumeType};
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut ledc = LEDC::new(peripherals.LEDC, &clocks);

    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut buzzer = Buzzer::new(
        &ledc,
        timer::Number::Timer0,
        channel::Number::Channel1,
        io.pins.gpio6,
        &clocks,
    );

    buzzer.with_volume(io.pins.gpio0, VolumeType::OnOff);
    buzzer.set_volume(1).unwrap();

    buzzer.play_tones(songs::DOOM).unwrap();
    buzzer.play_tones(songs::FURELISE).unwrap();
    buzzer.play_tones(songs::MERRY_CHRISTMAS).unwrap();
    buzzer.play_tones(songs::MII_CHANNEL).unwrap();
    buzzer.play_tones(songs::NEVER_GONNA_GIVE_YOU_UP).unwrap();
    buzzer.play_tones(songs::ODE_TO_JOY).unwrap();
    buzzer.play_tones(songs::PACMAN).unwrap();
    buzzer.play_tones(songs::STAR_WARS).unwrap();
    buzzer.play_tones(songs::SUPER_MARIO_BROS).unwrap();
    buzzer.play_tones(songs::TAKE_ON_ME).unwrap();
    buzzer.play_tones(songs::TETRIS).unwrap();
    buzzer.play_tones(songs::THE_LION_SLEEPS_TONIGHT).unwrap();
    buzzer.play_tones(songs::ZELDA_LULLABY).unwrap();
    buzzer.play_tones(songs::ZELDA_THEME).unwrap();

    println!("Done.");

    loop {}
}
