fn main() {
    // Ensure that only a single chip is specified
    let chip_features = [
        cfg!(feature = "esp32"),
        cfg!(feature = "esp32c3"),
        cfg!(feature = "esp32s2"),
        cfg!(feature = "esp32s3"),
        cfg!(feature = "esp8266"),
    ];

    match chip_features.iter().filter(|&&f| f).count() {
        1 => {}
        n => panic!("Exactly 1 chip must be enabled via its Cargo feature, {n} provided"),
    };

    // Ensure that only a single communication method is specified
    let method_features = [
        cfg!(feature = "uart"),
        cfg!(feature = "jtag_serial"),
        cfg!(feature = "rtt"),
    ];

    match method_features.iter().filter(|&&f| f).count() {
        1 => {}
        n => panic!(
            "Exactly 1 communication method must be enabled via its Cargo feature, {n} provided"
        ),
    }
}
