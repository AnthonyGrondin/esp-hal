//! This shows some of the interrupts that can be generated by UART/Serial.
//! Use a proper serial terminal to connect to the board (espmonitor and
//! espflash won't work)

#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};

use esp32s3_hal::{
    clock::ClockControl,
    interrupt,
    pac::{self, Peripherals, UART0},
    prelude::*,
    serial::config::AtCmdConfig,
    timer::TimerGroup,
    Cpu,
    RtcCntl,
    Serial,
};
use nb::block;
use panic_halt as _;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::entry;

static mut SERIAL: SpinLockMutex<RefCell<Option<Serial<UART0>>>> =
    SpinLockMutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the TIMG watchdog timer.
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    let mut wdt0 = timer_group0.wdt;

    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    let mut serial0 = Serial::new(peripherals.UART0);
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt0.disable();
    wdt1.disable();
    rtc_cntl.set_wdt_global_enable(false);

    serial0.set_at_cmd(AtCmdConfig::new(None, None, None, b'#', None));
    serial0.set_rx_fifo_full_threshold(30);
    serial0.listen_at_cmd();
    serial0.listen_rx_fifo_full();

    unsafe {
        interrupt::enable(
            Cpu::ProCpu,
            pac::Interrupt::UART0,
            interrupt::CpuInterrupt::Interrupt20LevelPriority2,
        );
    }

    timer0.start(1u64.secs());

    unsafe {
        (&SERIAL).lock(|data| (*data).replace(Some(serial0)));
    }

    unsafe {
        xtensa_lx::interrupt::disable();
        xtensa_lx::interrupt::enable_mask(1 << 20);
    }

    loop {
        unsafe {
            (&SERIAL).lock(|data| {
                let mut serial = data.borrow_mut();
                let serial = serial.as_mut().unwrap();
                writeln!(serial, "Hello World! Send a single `#` character or send at least 30 characters and see the interrupts trigger.").ok();
            });
        }

        block!(timer0.wait()).unwrap();
    }
}

#[no_mangle]
pub fn level2_interrupt() {
    unsafe {
        (&SERIAL).lock(|data| {
            let mut serial = data.borrow_mut();
            let serial = serial.as_mut().unwrap();

            let mut cnt = 0;
            while let nb::Result::Ok(_c) = serial.read() {
                cnt += 1;
            }
            writeln!(serial, "Read {} bytes", cnt,).ok();

            writeln!(
                serial,
                "Interrupt AT-CMD: {} RX-FIFO-FULL: {}",
                serial.at_cmd_interrupt_set(),
                serial.rx_fifo_full_interrupt_set(),
            )
            .ok();

            serial.reset_at_cmd_interrupt();
            serial.reset_rx_fifo_full_interrupt();
            interrupt::clear(
                Cpu::ProCpu,
                interrupt::CpuInterrupt::Interrupt20LevelPriority2,
            );
        });
    }
}
