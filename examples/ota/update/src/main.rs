//! OTA Update Example
//!
//! This shows the basics of dealing with partitions and changing the active
//! partition. For simplicity it will flash an application image embedded into
//! the binary. In a real world application you can get the image via HTTP(S),
//! UART or from an sd-card etc.
//!
//! Adjust the target and the chip in the following commands according to the
//! chip used!
//!
//! ```ignore,bash
//! cargo xtask build examples gpio --chip=esp32
//! espflash save-image --chip=esp32 target/xtensa-esp32-none-elf/release/gpio_interrupt examples/target/ota_image
//! cargo xtask build examples update --chip=esp32
//! espflash save-image --chip=esp32 target/xtensa-esp32-none-elf/release/ota_update examples/target/ota_image
//! cargo xtask build examples update --chip=esp32
//! espflash save-image --chip=esp32 target/xtensa-esp32-none-elf/release/ota_update examples/target/ota_image
//! espflash erase-flash
//! cargo xtask run example update --chip=esp32
//! ```
//!
//! On first boot notice the firmware partition gets booted ("Loaded app from
//! partition at offset 0x10000"). Press the BOOT button, once finished press
//! the RESET button.
//!
//! Notice OTA0 gets booted ("Loaded app from partition at offset 0x110000").
//!
//! Once again press BOOT, when finished press RESET.
//! You will see the `gpio_interrupt` example gets booted from OTA1 ("Loaded app
//! from partition at offset 0x210000")
//!
//! See <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/ota.html>

#![no_std]
#![no_main]

use embedded_storage::Storage;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    main,
};
use esp_println::println;
use esp_storage::FlashStorage;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut flash = FlashStorage::new(peripherals.FLASH);

    let mut buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let pt =
        esp_bootloader_esp_idf::partitions::read_partition_table(&mut flash, &mut buffer).unwrap();

    // List all partitions - this is just FYI
    for part in pt.iter() {
        println!("{:?}", part);
    }

    let current = pt.booted_partition().unwrap().unwrap();

    println!("Currently booted partition {:?}", pt.booted_partition());

    let mut sha_backend = esp_hal::sha::ShaBackend::new(peripherals.SHA);
    let _sha_driver = sha_backend.start();
    let mut sha256 = esp_hal::sha::Sha256Context::new();

    let sha = current.get_sha256(&mut sha256).unwrap();

    let mut buffer = [0u8; 32];
    sha256.finalize(&mut buffer).wait_blocking();

    println!("Sha: {buffer:?}");


    loop {}

}
