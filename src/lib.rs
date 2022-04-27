#![no_std]
#![allow(dead_code)]

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            writeln!($crate::Printer, $($arg)*).ok();
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            write!($crate::Printer, $($arg)*).ok();
        }
    };
}

#[cfg(feature = "esp32")]
const UART_TX_ONE_CHAR: usize = 0x40009200;
#[cfg(feature = "esp32c3")]
const UART_TX_ONE_CHAR: usize = 0x40000068;
#[cfg(feature = "esp32s2")]
const UART_TX_ONE_CHAR: usize = 0x40012b10;
#[cfg(feature = "esp32s3")]
const UART_TX_ONE_CHAR: usize = 0x40000648;
#[cfg(feature = "esp8266")]
const UART_TX_ONE_CHAR: usize = 0x40003b30;

pub struct Printer;

#[cfg(feature = "uart")]
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &b in s.as_bytes() {
            unsafe {
                let uart_tx_one_char: unsafe extern "C" fn(u8) -> i32 =
                    core::mem::transmute(UART_TX_ONE_CHAR);
                uart_tx_one_char(b)
            };
        }
        core::fmt::Result::Ok(())
    }
}

#[cfg(all(feature = "jtag_serial", feature = "esp32c3"))]
const SERIAL_JTAG_FIFO_REG: usize = 0x6004_3000;
#[cfg(all(feature = "jtag_serial", feature = "esp32c3"))]
const SERIAL_JTAG_CONF_REG: usize = 0x6004_3004;

#[cfg(all(feature = "jtag_serial", feature = "esp32s3"))]
const SERIAL_JTAG_FIFO_REG: usize = 0x6003_8000;
#[cfg(all(feature = "jtag_serial", feature = "esp32s3"))]
const SERIAL_JTAG_CONF_REG: usize = 0x6003_8004;

#[cfg(all(feature = "jtag_serial", any(feature = "esp32c3", feature = "esp32s3")))]
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
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
    }
}

mod rtt;

#[cfg(feature = "rtt")]
impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let count = crate::rtt::write_str_internal(s);
        if count < s.len() {
            crate::rtt::write_str_internal(&s[count..]);
        }
        core::fmt::Result::Ok(())
    }
}
