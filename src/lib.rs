#![no_std]

#[cfg(feature = "log")]
pub mod logger;
#[cfg(feature = "rtt")]
mod rtt;

#[cfg(all(not(feature = "no-op"), not(feature = "crlf")))]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            writeln!($crate::Printer, $($arg)*).ok();
        }
    }};
}

#[cfg(all(not(feature = "no-op"), feature = "crlf"))]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            write!($crate::Printer, $($arg)*).ok();
            write!($crate::Printer, "\r\n").ok();
        }
    }};
}

#[cfg(not(feature = "no-op"))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            write!($crate::Printer, $($arg)*).ok();
        }
    }};
}

#[cfg(feature = "no-op")]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{}};
}

#[cfg(feature = "no-op")]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{}};
}

// implementation adapted from `std::dbg`
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", ::core::file!(), ::core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    ::core::file!(), ::core::line!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Printer.write_bytes(s.as_bytes());
        Ok(())
    }
}

#[cfg(feature = "rtt")]
mod rtt_printer {
    impl super::Printer {
        pub fn write_bytes(&mut self, bytes: &[u8]) {
            super::with(|| {
                let count = crate::rtt::write_bytes_internal(bytes);
                if count < bytes.len() {
                    crate::rtt::write_bytes_internal(&bytes[count..]);
                }
            })
        }
    }
}

#[cfg(feature = "jtag_serial")]
mod serial_jtag_printer {
    #[cfg(feature = "esp32c3")]
    const SERIAL_JTAG_FIFO_REG: usize = 0x6004_3000;
    #[cfg(feature = "esp32c3")]
    const SERIAL_JTAG_CONF_REG: usize = 0x6004_3004;

    #[cfg(any(feature = "esp32c6", feature = "esp32h2"))]
    const SERIAL_JTAG_FIFO_REG: usize = 0x6000_F000;
    #[cfg(any(feature = "esp32c6", feature = "esp32h2"))]
    const SERIAL_JTAG_CONF_REG: usize = 0x6000_F004;

    #[cfg(feature = "esp32s3")]
    const SERIAL_JTAG_FIFO_REG: usize = 0x6003_8000;
    #[cfg(feature = "esp32s3")]
    const SERIAL_JTAG_CONF_REG: usize = 0x6003_8004;

    #[cfg(any(
        feature = "esp32c3",
        feature = "esp32c6",
        feature = "esp32h2",
        feature = "esp32s3"
    ))]
    impl super::Printer {
        pub fn write_bytes(&mut self, bytes: &[u8]) {
            super::with(|| {
                const TIMEOUT_ITERATIONS: usize = 5_000;

                let fifo = SERIAL_JTAG_FIFO_REG as *mut u32;
                let conf = SERIAL_JTAG_CONF_REG as *mut u32;

                if unsafe { conf.read_volatile() } & 0b011 == 0b000 {
                    // still wasn't able to drain the FIFO - early return
                    return;
                }

                // todo 64 byte chunks max
                for chunk in bytes.chunks(32) {
                    unsafe {
                        for &b in chunk {
                            fifo.write_volatile(b as u32);
                        }
                        conf.write_volatile(0b001);

                        let mut timeout = TIMEOUT_ITERATIONS;
                        while conf.read_volatile() & 0b011 == 0b000 {
                            // wait
                            timeout -= 1;
                            if timeout == 0 {
                                return;
                            }
                        }
                    }
                }
            })
        }
    }
}

#[cfg(feature = "uart")]
mod uart_printer {
    #[cfg(feature = "esp32")]
    const UART_TX_ONE_CHAR: usize = 0x4000_9200;
    #[cfg(any(feature = "esp32c2", feature = "esp32c6", feature = "esp32h2"))]
    const UART_TX_ONE_CHAR: usize = 0x4000_0058;
    #[cfg(feature = "esp32c3")]
    const UART_TX_ONE_CHAR: usize = 0x4000_0068;
    #[cfg(feature = "esp32s3")]
    const UART_TX_ONE_CHAR: usize = 0x4000_0648;
    #[cfg(feature = "esp8266")]
    const UART_TX_ONE_CHAR: usize = 0x4000_3b30;

    impl super::Printer {
        #[cfg(not(feature = "esp32s2"))]
        pub fn write_bytes(&mut self, bytes: &[u8]) {
            super::with(|| {
                for &b in bytes {
                    unsafe {
                        let uart_tx_one_char: unsafe extern "C" fn(u8) -> i32 =
                            core::mem::transmute(UART_TX_ONE_CHAR);
                        uart_tx_one_char(b)
                    };
                }
            })
        }

        #[cfg(feature = "esp32s2")]
        pub fn write_bytes(&mut self, bytes: &[u8]) {
            super::with(|| {
                // On ESP32-S2 the UART_TX_ONE_CHAR ROM-function seems to have some issues.
                for chunk in bytes.chunks(64) {
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
            })
        }
    }
}

#[inline]
fn with<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "critical-section")]
    return critical_section::with(|_| f());

    #[cfg(not(feature = "critical-section"))]
    f()
}
