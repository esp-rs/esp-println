# esp-println

Provides `print!`, `println!` and `dbg!` implementations for various Espressif devices.

- Supports ESP32, ESP32-C2/C3/C6, ESP32-H2, ESP32-S2/S3, and ESP8266
- Dependency free (not even depending on `esp-hal`, one optional dependency is `log`, another is `critical-section`)
- Supports JTAG-Serial output where available
- Supports RTT (lacking working RTT hosts besides _probe-rs_ for ESP32-C3)
- `no-op` features turns printing into a no-op

## RTT on ESP32-C3 / ESP32-C6

The _cli_ utility should work for flashing and showing RTT logs on ESP32-C3 by using it's `run` command.
You need to use the `direct-boot` feature of the HAL to flash via _probe-rs_.

## Usage

In your `Cargo.toml`, under `[dependencies]`, add:

```
esp-println = { version = "0.3.1", features = ["esp32"] }
```

Choose a recent version and your chipset.

Then in your program:

```
use esp_println::println;
```

You can now `println!("Hello world")` as usual.

## Logging

With the feature `log` activated you can initialize a simple logger like this
```rust
init_logger(log::LevelFilter::Info);
```

There is a default feature `colors` which enables colored log output.

Additionally you can use
```rust
init_logger_from_env();
```

In this case the following environment variables are used:
- `ESP_LOGLEVEL` sets the log level, use values like `trace`, `info` etc.
- `ESP_LOGTARGETS` if set you should provide the crate names of crates (optionally with a path e.g. `esp_wifi::compat::common`) which should get logged, separated by `,` and no additional whitespace between

If this simple logger implementation isn't sufficient for your needs you can implement your own logger on top of `esp-println` - see https://docs.rs/log/0.4.17/log/#implementing-a-logger

## defmt

Using the `defmt` feature, esp-println will install a defmt global logger. The logger will output
to the same data stream as `println!()`, and adds framing bytes so it can be used even with other,
non-defmt output. Using the `defmt` feature automatically uses the Rzcobs encoding and does not
allow changing the encoding.

You can also use the `defmt-raw` feature that allows using any encoding provided by defmt, but
does not add extra framing. Using this feature requires some care as the defmt output may become
unrecoverably mangled when other data are printed.

Follow the [defmt book's setup instructions](https://defmt.ferrous-systems.com/setup) on how to
set up defmt. Remember, the global logger is already installed for you by esp-println!

### Troubleshooting linker errors

If you experience linker errors, make sure you have *some* reference to `esp_println` in your code.
If you don't use `esp_println` directly, you'll need to add e.g. `use esp_println as _;` to your
import statements. This ensures that the global logger will not be removed by the compiler.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
