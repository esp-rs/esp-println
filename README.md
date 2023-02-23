# esp-println

Provides `print!` and `println!` implementations for various Espressif devices.

- Supports ESP32, ESP32-C2/C3/C6, ESP32-H2, ESP32-S2/S3, and ESP8266
- Dependency free (not even depending on `esp-hal`, one optional dependency is `log`, another is `critical-section`)
- Supports JTAG-Serial output where available (activates `critical-section`)
- Supports RTT (lacking working RTT hosts besides _probe-rs_ for ESP32-C3)
- `no-op` features turns printing into a no-op

## RTT on ESP32-C3

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


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
