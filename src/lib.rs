#![no_std]


#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[cfg(not(feature = "no-op"))]
        {
            use core::fmt::Write;
            writeln!($crate::Printer, $($arg)*).ok();
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[cfg(not(feature = "no-op"))]
        {
            use core::fmt::Write;
            write!($crate::Printer, $($arg)*).ok();
        }
    };
}

#[cfg(feature = "log")]
pub mod logger;

#[cfg(feature = "esp32")]
const UART_TX_ONE_CHAR: usize = 0x40009200;
#[cfg(any(feature = "esp32c2", feature = "esp32c6", feature = "esp32h2"))]
const UART_TX_ONE_CHAR: usize = 0x40000058;
#[cfg(feature = "esp32c3")]
const UART_TX_ONE_CHAR: usize = 0x40000068;
#[cfg(feature = "esp32s3")]
const UART_TX_ONE_CHAR: usize = 0x40000648;
#[cfg(feature = "esp8266")]
const UART_TX_ONE_CHAR: usize = 0x40003b30;

pub struct Printer;

#[cfg(feature = "uart")]
impl core::fmt::Write for Printer {
    #[cfg(not(feature = "esp32s2"))]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        with(|| {
            for &b in s.as_bytes() {
                unsafe {
                    let uart_tx_one_char: unsafe extern "C" fn(u8) -> i32 =
                        core::mem::transmute(UART_TX_ONE_CHAR);
                    uart_tx_one_char(b)
                };
            }
            core::fmt::Result::Ok(())
        })
    }

    #[cfg(feature = "esp32s2")]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        with(|| {
            // On ESP32-S2 the UART_TX_ONE_CHAR ROM-function seems to have some issues.
            for chunk in s.as_bytes().chunks(64) {
                for &b in chunk {
                    unsafe {
                        // write FIFO
                        (0x3f400000 as *mut u32).write_volatile(b as u32);
                    };
                }

                // wait for TX_DONE
                while unsafe { (0x3f400004 as *const u32).read_volatile() } & (1 << 14) == 0 {}
                unsafe {
                    // reset TX_DONE
                    (0x3f400010 as *mut u32).write_volatile(1 << 14);
                }
            }
            core::fmt::Result::Ok(())
        })
    }
}

#[cfg(all(feature = "jtag_serial", any(feature = "esp32c3", feature = "esp32h2")))]
const SERIAL_JTAG_FIFO_REG: usize = 0x6004_3000;
#[cfg(all(feature = "jtag_serial", any(feature = "esp32c3", feature = "esp32h2")))]
const SERIAL_JTAG_CONF_REG: usize = 0x6004_3004;

#[cfg(all(feature = "jtag_serial", feature = "esp32c6"))]
const SERIAL_JTAG_FIFO_REG: usize = 0x6000_F000;
#[cfg(all(feature = "jtag_serial", feature = "esp32c6"))]
const SERIAL_JTAG_CONF_REG: usize = 0x6000_F004;

#[cfg(all(feature = "jtag_serial", feature = "esp32s3"))]
const SERIAL_JTAG_FIFO_REG: usize = 0x6003_8000;
#[cfg(all(feature = "jtag_serial", feature = "esp32s3"))]
const SERIAL_JTAG_CONF_REG: usize = 0x6003_8004;

#[cfg(all(
    feature = "jtag_serial",
    any(
        feature = "esp32c3",
        feature = "esp32c6",
        feature = "esp32h2",
        feature = "esp32s3"
    )
))]
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        with(|| {
            unsafe {
                let fifo = SERIAL_JTAG_FIFO_REG as *mut u32;
                let conf = SERIAL_JTAG_CONF_REG as *mut u32;

                // todo 64 byte chunks max
                for chunk in s.as_bytes().chunks(32) {
                    for &b in chunk {
                        fifo.write_volatile(b as u32);
                    }
                    conf.write_volatile(0b001);

                    while conf.read_volatile() & 0b011 == 0b000 {
                        // wait
                    }
                }
            }

            core::fmt::Result::Ok(())
        })
    }
}

#[cfg(feature = "rtt")]
mod rtt;

#[cfg(feature = "rtt")]
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        with(|| {
            let count = crate::rtt::write_str_internal(s);
            if count < s.len() {
                crate::rtt::write_str_internal(&s[count..]);
            }
            core::fmt::Result::Ok(())
        })
    }
}

#[inline]
fn with<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "critical-section")]
    return critical_section::with(|_| f());

    #[cfg(not(feature = "critical-section"))]
    f()
}
