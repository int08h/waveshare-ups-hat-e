// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Will forcefully power-off the Raspberry Pi connected to the Waveshare UPS Hat E.
//! 
use std::env;
use std::io::Write;
use std::process::exit;
use waveshare_ups_hat_e::UpsHatE;

fn confirm_power_off(args: &Vec<String>) -> bool {
    if args.len() == 2 && args[1].to_ascii_lowercase() == "-y" {
        return true;
    }

    print!("Are you sure you want to power-off the Raspberry Pi? [y/N] ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("failed to read input");
    input.trim().to_ascii_lowercase() == "y"
}

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() == 2 && args[1].to_ascii_lowercase() != "-y" {
        println!("Usage: force_power_off [-y]");
        println!("  -y: skip confirmation prompt");
        println!();
        exit(1);
    };

    if !confirm_power_off(&args) {
        println!("Aborting power-off due to user input. Use -y to skip confirmation prompt.");
        exit(1);
    }

    let mut ups = UpsHatE::new();

    ups.force_power_off().expect("failed to issue power-off command");

    let pending = ups.is_power_off_pending().expect("failed reading power-off status");

    if pending {
        println!("UPS will power-off the attached Raspberry Pi in 30 seconds");
        exit(0);
    } else {
        println!("Error: UPS failed to initiate power-off");
        exit(2);
    }
}