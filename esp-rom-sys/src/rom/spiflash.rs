//! Definitions of ROM functions related to flash memory.

/// The operation succeeded
pub const ESP_ROM_SPIFLASH_RESULT_OK: i32 = 0;

/// The operation errored
pub const ESP_ROM_SPIFLASH_RESULT_ERR: i32 = 1;

/// The operation timed out
pub const ESP_ROM_SPIFLASH_RESULT_TIMEOUT: i32 = 2;

/// Memory space requested in an mmap call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum SpiFlashMmapMemory {
    /// Map to data memory, allowing byte-aligned access.
    Data = 0,
    /// Map to instruction memory, allowing only 4-byte-aligned access.
    Instruction = 1,
}

unsafe extern "C" {
    /// Read Data from Flash via ROM code, you should Erase it yourself if need.
    ///
    /// `src_addr` should be 4 bytes aligned.
    /// `len` should be 4 bytes aligned.
    pub fn esp_rom_spiflash_read(src_addr: u32, data: *const u32, len: u32) -> i32;

    /// Clear all SR bits except QE bit.
    pub fn esp_rom_spiflash_unlock() -> i32;

    /// Erase a sector of flash. Uses SPI flash command 20H.
    pub fn esp_rom_spiflash_erase_sector(sector_number: u32) -> i32;

    /// Erase a block of flash.
    pub fn esp_rom_spiflash_erase_block(block_number: u32) -> i32;

    /// Write Data to Flash, you should Erase it yourself if need.
    ///
    /// `dest_addr` should be 4 bytes aligned.
    /// `len` should be 4 bytes aligned.
    pub fn esp_rom_spiflash_write(dest_addr: u32, data: *const u32, len: u32) -> i32;

    /// Get the number of free pages available for memory mapping.
    ///
    /// This function returns the number of free pages available in the MMU table.
    /// This can be useful before calling `spi_flash_mmap`, which maps a flash
    /// range to DCache or ICache memory, to check whether there is sufficient
    /// space available for mapping.
    ///
    /// `memory` specifies the memory type of the MMU table free pages.
    ///
    /// Returns the number of free pages which can be mapped.
    ///
    /// # Safety
    ///
    /// The ROM mmap driver must be initialized before calling this function.
    /// ESP-IDF does this by installing allocator callbacks with
    /// `spi_flash_mmap_os_func_set` and initializing the page count with
    /// `spi_flash_mmap_page_num_init`. Bare-metal applications generally have
    /// not run that setup.
    #[cfg(not(any(feature = "esp32", feature = "esp32s2")))]
    pub fn spi_flash_mmap_get_free_pages(memory: SpiFlashMmapMemory) -> u32;
}
