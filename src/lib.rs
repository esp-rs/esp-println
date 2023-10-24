#![no_std]

#[cfg(any(feature = "defmt", feature = "defmt-raw"))]
pub mod defmt;
#[cfg(feature = "log")]
pub mod logger;
#[cfg(feature = "rtt")]
mod rtt;

#[cfg(not(feature = "no-op"))]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            writeln!($crate::Printer, $($arg)*).ok();
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

impl Printer {
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        with(|| self.write_bytes_assume_cs(bytes))
    }
}

#[cfg(feature = "rtt")]
mod rtt_printer {
    impl super::Printer {
        pub(crate) fn write_bytes_assume_cs(&mut self, bytes: &[u8]) {
            let count = crate::rtt::write_bytes_internal(bytes);
            if count < bytes.len() {
                crate::rtt::write_bytes_internal(&bytes[count..]);
            }
        }
    }
}

#[cfg(all(
    feature = "jtag_serial",
    any(
        feature = "esp32c3",
        feature = "esp32c6",
        feature = "esp32h2",
        feature = "esp32s3"
    )
))]
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

    fn fifo_flush() {
        let conf = SERIAL_JTAG_CONF_REG as *mut u32;
        unsafe { conf.write_volatile(0b001) };
    }

    fn fifo_clear() -> bool {
        let conf = SERIAL_JTAG_CONF_REG as *mut u32;
        unsafe { conf.read_volatile() & 0b010 != 0b000 }
    }

    fn fifo_write(byte: u8) {
        let fifo = SERIAL_JTAG_FIFO_REG as *mut u32;
        unsafe { fifo.write_volatile(byte as u32) }
    }

    impl super::Printer {
        pub fn write_bytes_assume_cs(&mut self, bytes: &[u8]) {
            const TIMEOUT_ITERATIONS: usize = 50_000;

            if !fifo_clear() {
                // Still wasn't able to drain the FIFO - early return
                // This is important so we don't block forever if there is no host attached.
                return;
            }

            for chunk in bytes.chunks(64) {
                for &b in chunk {
                    fifo_write(b);
                }

                fifo_flush();

                // wait for fifo to clear
                let mut timeout = TIMEOUT_ITERATIONS;
                while !fifo_clear() {
                    if timeout == 0 {
                        return;
                    }
                    timeout -= 1;
                }
            }
        }
    }
}

#[cfg(all(feature = "uart", any(feature = "esp32", feature = "esp8266")))]
mod uart_printer {
    const UART_TX_ONE_CHAR: usize = 0x4000_9200;
    impl super::Printer {
        pub fn write_bytes_assume_cs(&mut self, bytes: &[u8]) {
            for &b in bytes {
                unsafe {
                    let uart_tx_one_char: unsafe extern "C" fn(u8) -> i32 =
                        core::mem::transmute(UART_TX_ONE_CHAR);
                    uart_tx_one_char(b)
                };
            }
        }
    }
}

#[cfg(all(feature = "uart", feature = "esp32s2"))]
mod uart_printer {
    const UART_TX_ONE_CHAR: usize = 0x4000_9200;
    impl super::Printer {
        pub fn write_bytes_assume_cs(&mut self, bytes: &[u8]) {
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
        }
    }
}

#[cfg(all(
    feature = "uart",
    not(any(feature = "esp32", feature = "esp32s2", feature = "esp8266"))
))]
mod uart_printer {
    trait Functions {
        const TX_ONE_CHAR: usize;
        const CHUNK_SIZE: usize = 32;

        fn tx_byte(b: u8) {
            unsafe {
                let tx_one_char: unsafe extern "C" fn(u8) -> i32 =
                    core::mem::transmute(Self::TX_ONE_CHAR);
                tx_one_char(b);
            }
        }

        fn flush();
    }

    struct Device;

    #[cfg(feature = "esp32c2")]
    impl Functions for Device {
        const TX_ONE_CHAR: usize = 0x4000_005C;

        fn flush() {
            // tx_one_char waits for empty
        }
    }

    #[cfg(feature = "esp32c3")]
    impl Functions for Device {
        const TX_ONE_CHAR: usize = 0x4000_0068;

        fn flush() {
            unsafe {
                const TX_FLUSH: usize = 0x4000_0080;
                const GET_CHANNEL: usize = 0x4000_058C;
                let tx_flush: unsafe extern "C" fn(u8) = core::mem::transmute(TX_FLUSH);
                let get_channel: unsafe extern "C" fn() -> u8 = core::mem::transmute(GET_CHANNEL);

                const G_USB_PRINT_ADDR: usize = 0x3FCD_FFD0;
                let g_usb_print = G_USB_PRINT_ADDR as *mut bool;

                let channel = if *g_usb_print {
                    // Flush USB-JTAG
                    3
                } else {
                    get_channel()
                };
                tx_flush(channel);
            }
        }
    }

    #[cfg(feature = "esp32s3")]
    impl Functions for Device {
        const TX_ONE_CHAR: usize = 0x4000_0648;

        fn flush() {
            unsafe {
                const TX_FLUSH: usize = 0x4000_0690;
                const GET_CHANNEL: usize = 0x4000_1A58;
                let tx_flush: unsafe extern "C" fn(u8) = core::mem::transmute(TX_FLUSH);
                let get_channel: unsafe extern "C" fn() -> u8 = core::mem::transmute(GET_CHANNEL);

                const G_USB_PRINT_ADDR: usize = 0x3FCE_FFB8;
                let g_usb_print = G_USB_PRINT_ADDR as *mut bool;

                let channel = if *g_usb_print {
                    // Flush USB-JTAG
                    4
                } else {
                    get_channel()
                };
                tx_flush(channel);
            }
        }
    }

    #[cfg(any(feature = "esp32c6", feature = "esp32h2"))]
    impl Functions for Device {
        const TX_ONE_CHAR: usize = 0x4000_0058;

        fn flush() {
            unsafe {
                const TX_FLUSH: usize = 0x4000_0074;
                const GET_CHANNEL: usize = 0x4000_003C;

                let tx_flush: unsafe extern "C" fn(u8) = core::mem::transmute(TX_FLUSH);
                let get_channel: unsafe extern "C" fn() -> u8 = core::mem::transmute(GET_CHANNEL);

                tx_flush(get_channel());
            }
        }
    }

    impl super::Printer {
        pub fn write_bytes_assume_cs(&mut self, bytes: &[u8]) {
            for chunk in bytes.chunks(Device::CHUNK_SIZE) {
                for &b in chunk {
                    Device::tx_byte(b);
                }

                Device::flush();
            }
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
