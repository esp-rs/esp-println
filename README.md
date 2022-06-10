# esp-println - println for ESP32 bare-metal

- supports ESP32, ESP32-C3, ESP32-S2, ESP32-S3, and ESP8266
- dependency free (not even depending on `esp-hal`)
- supports JTAG-Serial output where available
- supports RTT (lacking working RTT hosts besides _probe-rs_ for ESP32C3)

## RTT on ESP32C3

The _cli_ utility should work for flashing and showing RTT logs on ESP32C3 by using it's `run` command.
You need to use the `direct-boot` feature of the HAL to flash via _probe-rs_.


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
