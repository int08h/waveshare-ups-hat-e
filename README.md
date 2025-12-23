# waveshare-ups-hat-e

A Rust library for monitoring the [Waveshare UPS HAT (E)](https://www.waveshare.com/wiki/UPS_HAT_(E)) on Raspberry Pi via I2C.

## Example Output

Output from the [`ups_monitor`](examples/ups_monitor.rs) example demonstrating the type of information available:

![Screenshot of ups_monitor example](https://github.com/int08h/waveshare-ups-hat-e/blob/master/images/screenshot.png?raw=true)

## Usage

```rust
use waveshare_ups_hat_e::UpsHatE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ups = UpsHatE::new();

    let battery = ups.get_battery_state()?;
    println!("Battery: {}% ({} mV, {} mA)",
        battery.remaining_percent,
        battery.millivolts,
        battery.milliamps);
    println!("Remaining: {} mAh, {} min",
        battery.remaining_capacity_milliamphours,
        battery.remaining_runtime_minutes);

    let power = ups.get_power_state()?;
    println!("Charging: {:?}, Activity: {:?}",
        power.charging_state,
        power.charger_activity);
    println!("USB-C: {:?}, PD: {:?}",
        power.usbc_input_state,
        power.usbc_power_delivery);

    let vbus = ups.get_usbc_vbus()?;
    println!("VBUS: {} mV, {} mA, {} mW",
        vbus.millivolts,
        vbus.milliamps,
        vbus.milliwatts);

    let cells = ups.get_cell_voltage()?;
    println!("Cells: {:?}", cells);

    Ok(())
}
```

## API

| Method | Description |
|--------|-------------|
| [`get_battery_state`](UpsHatE::get_battery_state) | Voltage, current, capacity, remaining runtime |
| [`get_power_state`](UpsHatE::get_power_state) | Charging state, USB-C input, power delivery mode |
| [`get_cell_voltage`](UpsHatE::get_cell_voltage) | Individual cell voltages (4 cells) |
| [`get_usbc_vbus`](UpsHatE::get_usbc_vbus) | USB-C voltage, current, power |
| [`get_communication_state`](UpsHatE::get_communication_state) | Communication status with BQ4050 and IP2368 chips |
| [`get_software_revision`](UpsHatE::get_software_revision) | Firmware revision of the UPS microcontroller |
| [`is_battery_low`](UpsHatE::is_battery_low) | Check if battery voltage is below threshold |
| [`is_power_off_pending`](UpsHatE::is_power_off_pending) | Check if a power-off has been initiated |
| [`force_power_off`](UpsHatE::force_power_off) | Initiate power-off in 30 seconds (cannot be canceled) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

## Copyright

Copyright (c) 2025 Stuart Stock, all rights reserved.
