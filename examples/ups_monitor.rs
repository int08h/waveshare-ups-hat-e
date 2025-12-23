// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A "top" like monitor for the Waveshare UPS Hat E.
//!
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use waveshare_ups_hat_e::UpsHatE;

const CLEAR_SCREEN: &str = "\x1b[2J";
const CURSOR_HOME: &str = "\x1b[H";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ups = UpsHatE::new();
    let mut stdout = io::stdout();

    let software_revision = ups.get_software_revision()?;

    loop {
        let battery = ups.get_battery_state()?;
        let power = ups.get_power_state()?;
        let comm = ups.get_communication_state()?;
        let vbus = ups.get_usbc_vbus()?;
        let cells = ups.get_cell_voltage()?;
        let battery_low = ups.is_battery_low()?;
        let power_off_pending = ups.is_power_off_pending()?;

        print!("{CLEAR_SCREEN}{CURSOR_HOME}");

        let epoch_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        println!("{BOLD}UPS HAT (E) Monitor{RESET}");
        println!("═══════════════════════════════════════════");
        println!("Unix time: {epoch_secs}");
        println!();

        println!("{BOLD}UPS Info{RESET}");
        println!("  Software Rev:  {:?}", software_revision);
        println!();

        // Power state
        println!("{BOLD}Power{RESET}");
        println!("  State:         {:?}", power.charging_state);
        println!("  Activity:      {:?}", power.charger_activity);
        println!("  USB-C In:      {:?}", power.usbc_input_state);
        println!("  USB-C PD:      {:?}", power.usbc_power_delivery);
        println!("  Off Pending?   {}", if power_off_pending { "Yes" } else { "No" });
        println!();

        // Communication state
        println!("{BOLD}Communication{RESET}");
        println!("  BQ4050:        {:?}", comm.bq4050);
        println!("  IP2368:        {:?}", comm.ip2368);
        println!();

        // Battery
        println!("{BOLD}Battery{RESET}");
        println!("  Charge:        {}%", battery.remaining_percent);
        println!("  Voltage:       {} mV", battery.millivolts);
        println!("  Current:       {} mA", battery.milliamps);
        println!("  Est. Capacity: {} mAh", battery.remaining_capacity_milliamphours);
        if battery.milliamps < 0 {
            println!("  Est. Runtime:  {} min", battery.remaining_runtime_minutes);
        } else if battery.time_to_full_minutes > 0 {
            println!("  Time To Full:  {} min", battery.time_to_full_minutes);
        }
        println!("  Low Battery?   {}", if battery_low { "Yes" } else { "No" });
        println!();

        // USB-C VBUS
        println!("{BOLD}USB-C VBUS{RESET}");
        println!("  Voltage:      {} mV", vbus.millivolts);
        println!("  Current:      {} mA", vbus.milliamps);
        println!("  Power:        {} mW", vbus.milliwatts);
        println!();

        // Cell voltages
        println!("{BOLD}Cell Voltages{RESET}");
        println!("  Cell 1:       {} mV", cells.cell_1_millivolts);
        println!("  Cell 2:       {} mV", cells.cell_2_millivolts);
        println!("  Cell 3:       {} mV", cells.cell_3_millivolts);
        println!("  Cell 4:       {} mV", cells.cell_4_millivolts);
        println!();

        println!("Press Ctrl+C to exit");

        stdout.flush()?;
        thread::sleep(Duration::from_secs(2));
    }
}
