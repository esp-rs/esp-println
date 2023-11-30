# esp-println

A library that provides `print!`, `println!`, `dbg!` implementations and
logging capabilities for Espressif devices.
- Supports all Espressif devices.
- Supports different communication methods:
    - UART (Default)
    - JTAG-Serial (Only available in ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S3)
    - [RTT]
    - No-op: Turns printing into a no-op
- Supports [`defmt`] backend

# Usage

```toml
esp-println = { version = "0.7.0", features = ["esp32c2"] }
```
or `cargo add esp-println --features esp32c2`
It's important to specify your target device as feature.

Then in your program:

```rust
use esp_println::println;
```

You can now `println!("Hello world")` as usual.

# Features

- There is one feature for each supported target: `esp32`, `esp32c2`,
  `esp32c3`, `esp32c6`, `esp32h2`, `esp32s2`, `esp32s3` and `esp8266`.
   - One of these features must be enabled.
   - Only one of these features can be enabled at a time.
- There is one feature for each supported communication method: `uart`,
  `jtag-serial` and `rtt` `no-op`.
    - Only one of these features can be enabled at a time.
- `log`: Enables logging using [`log` crate].
- `colors` enable colored logging.
   - Only effective when using the `log` feature.
- `critical-section` enables critical sections.
- There are two ways to use this library with [`defmt`]:
   - `defmt-espflash`: This is intended to be used with [`espflash`], see `-L/--log-format` argument of `flash` or `monitor` subcommands of `espflash` and `cargo-espflash`.
     Uses [rzCOBS] encoding and adds framing.
   - `defmt-raw`: Raw defmt output without additional framing. ⚠️ Be careful when using this feature: you must only write
     output using `defmt` macros, or you may irrecoverably corrupt the output stream! This means that even the bootloader's output
     must be disabled.

`defmt` features can also be used with [`probe-rs`].

[`probe-rs`]: https://probe.rs/

## Default Features

By default, we use the `uart`, `critial-section` and `colors` features.
Which means that it will print to the UART, use critical sections and output
messages will be colored.
If we want to use a communication method that is not `uart`, the default
one, we need to [disable the default features].

## Logging

With the feature `log` activated you can initialize a simple logger like this
```rust
init_logger(log::LevelFilter::Info);
```

There is a default feature `colors` which enables colored log output.

Additionally, you can use
```rust
init_logger_from_env();
```

In this case the following environment variables are used:
- `ESP_LOGLEVEL` sets the log level, use values like `trace`, `info` etc.
- `ESP_LOGTARGETS` if set you should provide the crate names of crates (optionally with a path e.g. `esp_wifi::compat::common`) which should get logged, separated by `,` and no additional whitespace between

If this simple logger implementation isn't sufficient for your needs, you can implement your own logger on top of `esp-println`. See [Implementing a Logger section log documentaion]

## `defmt`

Using the `defmt-espflash` feature, esp-println will install a defmt global logger. The logger will
output to the same data stream as `println!()`, and adds framing bytes so it can be used even with
other, non-defmt output. Using the `defmt-espflash` feature automatically uses the Rzcobs encoding and does
not allow changing the encoding.

You can also use the `defmt-raw` feature that allows using any encoding provided by `defmt`, but
does not add extra framing. Using this feature requires some care as the `defmt` output may become
unrecoverably mangled when other data is printed.

Follow the [`defmt` book's setup instructions] on how to
set up `defmt`. Remember, the global logger is already installed for you by `esp-println`!

# `esp-backtrace`

`esp-println` is usually used alongside [`esp-backtrace`]. When using this
two crates together, make sure to use the same communication methods for
both dependencies. Table matching features:
| `esp-println` | `esp-backtrace`     |
| ------------- | ------------------- |
| `uart`        | `print-uart`        |
| `jtag_serial` | `print-jtag-serial` |
| `rtt`         | `print-rtt`         |

[`defmt`]: https://github.com/knurling-rs/defmt
[`log` crate]: https://github.com/rust-lang/log
[rzCOBS]: https://github.com/Dirbaio/rzcobs
[`espflash`]: https://github.com/esp-rs/espflash
[rtt]: https://wiki.segger.com/RTT
[disable the default features]: https://doc.rust-lang.org/cargo/reference/features.html#the-default-feature
[`esp-backtrace`]: https://github.com/esp-rs/esp-backtrace
[Implementing a Logger section log documentaion]: https://docs.rs/log/0.4.17/log/#implementing-a-logger
[`defmt` book's setup instructions]: https://defmt.ferrous-systems.com/setup

# Troubleshooting linker errors

If you experience linker errors, make sure you have *some* reference to `esp_println` in your code.
If you don't use `esp_println` directly, you'll need to add e.g. `use esp_println as _;` to your
import statements. This ensures that the global logger will not be removed by the compiler.

# License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
