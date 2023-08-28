//! Demonstrates the use of the Digital Signature (DS) peripheral and compares
//! the speed of hardware-accelerated and pure software hashing.

#![no_std]
#![no_main]

use esp32s3_hal::{
    clock::ClockControl,
    ds::{Ds, DsData},
    peripherals::Peripherals,
    prelude::*,
    systimer::SystemTimer,
    timer::TimerGroup,
    Rng,
    Rtc,
};
use esp_backtrace as _;
use esp_println::println;
use nb::block;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt = timer_group0.wdt;

    let mut rng = Rng::new(peripherals.RNG);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    rtc.rwdt.disable();
    wdt.disable();

    esp_println::logger::init_logger_from_env();

    let mut ds = Ds::new(
        peripherals.DS,
        peripherals.HMAC,
        &mut system.peripheral_clock_control,
    );

    ds.configure(esp32s3_hal::hmac::KeyId::Key1).unwrap();

    loop {}
}
