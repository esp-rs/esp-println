//! defmt global logger implementation.
// Implementation taken from defmt-rtt, with a custom framing prefix

use core::sync::atomic::{AtomicBool, Ordering};

use critical_section::RestoreState;

use super::Printer;

/// Global logger lock.
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: RestoreState = RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[defmt::global_logger]
pub struct Logger;
unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        TAKEN.store(true, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        unsafe { CS_RESTORE = restore };

        // Write a non-UTF8 sequence to indicate the start of a defmt frame. We need
        // this to distinguish defmt frames from other data that might be
        // written to the printer.
        do_write(&[0xFF, 0x00]);

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        ENCODER.end_frame(do_write);

        // We don't need to write a custom end-of-frame sequence because the rzcobs
        // encoding already includes a terminating zero.

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        TAKEN.store(false, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        let restore = CS_RESTORE;

        // safety: Must be paired with corresponding call to acquire(), see above
        critical_section::release(restore);
    }

    unsafe fn flush() {}

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    Printer.write_bytes(bytes)
}
