use core::convert::Infallible;

use crate::{
    hmac::{Hmac, HmacPurpose, KeyId},
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{DS, HMAC},
    system::{Peripheral as PeripheralEnable, PeripheralClockControl},
};

// TODO: Adapt per chip
const DS_IV_LEN: usize = 16;
const DS_C_LEN: usize = 12672 / 8;

pub struct DsData {
    /// RSA LENGTH register parameters
    /// (number of words in RSA key & operands, minus one).
    ///
    /// Max value 127 (for RSA 4096).
    ///
    /// This value must match the length field encrypted and stored in 'c',
    /// or invalid results will be returned. (The DS peripheral will
    /// always use the value in 'c', not this value, so an attacker can't
    /// alter the DS peripheral results this way, it will just truncate or
    /// extend the message and the resulting signature in software.)
    rsa_length: usize,

    /// IV value used to encrypt 'c'
    iv: [u8; DS_IV_LEN],

    /// Encrypted Digital Signature parameters. Result of AES-CBC encryption
    /// of plaintext values. Includes an encrypted message digest.
    c: [u8; DS_C_LEN],
}

/// DS interface error
#[derive(Debug)]
pub enum Error {
    /// Supplied parameters are invalid
    InvalidParam,

    /// Errors from the HMAC peripheral
    Hmac(crate::hmac::Error),

    /// 'c' decrypted with invalid padding
    InvalidPadding,

    /// 'c' decrypted with invalid digest
    InvalidDigest,
}

impl From<crate::hmac::Error> for Error {
    fn from(value: crate::hmac::Error) -> Self {
        Self::Hmac(value)
    }
}

pub struct Ds<'d> {
    hmac: Hmac<'d>,
    ds: PeripheralRef<'d, DS>,
}

impl<'d> Ds<'d> {
    pub fn new(
        ds: impl Peripheral<P = DS> + 'd,
        hmac: impl Peripheral<P = HMAC> + 'd,
        peripheral_clock_control: &mut PeripheralClockControl,
    ) -> Self {
        crate::into_ref!(ds);

        // We also enable SHA and HMAC here. SHA is used by HMAC, HMAC is used by DS.
        peripheral_clock_control.enable(PeripheralEnable::Sha);
        peripheral_clock_control.enable(PeripheralEnable::Hmac);
        peripheral_clock_control.enable(PeripheralEnable::Ds);

        let hmac = Hmac::new(hmac, peripheral_clock_control);

        Self { hmac, ds }
    }

    pub fn free(self) -> PeripheralRef<'d, DS> {
        self.ds
    }

    /// Step 1. Enable the DS module.
    pub fn enable(&mut self) {
        // Enable HMAC peripheral
        let bit = self.ds.query_key_wrong.read().query_key_wrong().bits();
        log::info!("{}", bit);
    }

    pub fn configure(&mut self, key_id: KeyId) -> Result<(), Error> {
        // Configure HMAC purpose and key
        log::debug!("Configuring HMAC");
        self.hmac.init();
        // TODO: Disable peripheral if we fail with error
        nb::block!(self.hmac.configure(HmacPurpose::ToDs, key_id))?;
        log::debug!("Configuring HMAC Done! Using key: {:?}", key_id);

        // Start DS peripheral
        self.ds.set_start.write(|w| w.set_start().set_bit());
        // TODO: If pending for too long, this means HMAC failed
        // Question: Since we handle it above, does this mean this never fails.
        while self.is_busy() {
            log::error!("Busy");
        }

        let bit = self.ds.query_busy.read().query_busy().bit();
        log::debug!("BUSy: {}", bit);
        let bit = self.ds.query_key_wrong.read().query_key_wrong().bits();
        log::debug!("query_key_wrong: {}", bit);

        // write private key params
        //

        // Start DS operation
        self.ds.set_me.write(|w| w.set_me().set_bit());
        // Poll until not busy
        loop {
            let busy = self.ds.query_busy.read().query_busy().bit() as u8;
            if busy == 0 {
                break;
            }
            log::error!("Busy: {}", busy);
        }

        Ok(())
    }

    fn is_busy(&mut self) -> bool {
        self.ds.query_busy.read().query_busy().bit_is_set()
    }

    pub fn finalize(&mut self) -> nb::Result<(), Infallible> {
        self.ds.set_finish.write(|w| w.set_finish().set_bit());
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        Ok(())
    }

    // fn write_private_key_params(&mut self, c: [u8; DS_C_LEN]) {
    //     self.
    // }
}
