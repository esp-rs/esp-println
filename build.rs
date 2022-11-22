use feature_utils::mandatory_and_unique;

mandatory_and_unique!(
    "esp32", "esp32c2", "esp32c3", "esp32c6", "esp32h2", "esp32s2", "esp32s3", "esp8266",
);

mandatory_and_unique!("uart", "jtag_serial", "rtt");

fn main() {
    // Ensure that, if the `jtag_serial` communication method feature is enabled,
    // either the `esp32c3`, `esp32c6` or `esp32s3` chip feature is enabled.
    if cfg!(feature = "jtag_serial")
        && !(cfg!(feature = "esp32c3")
            || cfg!(feature = "esp32c6")
            || cfg!(feature = "esp32h2")
            || cfg!(feature = "esp32s3"))
    {
        panic!(
            "The `jtag_serial` feature is only supported by the ESP32-C3, ESP32-C6, and ESP32-S3"
        );
    }
}
