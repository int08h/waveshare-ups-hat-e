// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

use waveshare_ups_hat_e::UpsHatE;

fn main() {
    println!("Hello, world!");

    let mut ups_hat = UpsHatE::default();
    let v = ups_hat.get_cell_voltage().unwrap();
    println!("{:?}", v);

    let b = ups_hat.get_battery_state().unwrap();
    println!("{:?}", b);

    let vbus = ups_hat.get_usbc_vbus().unwrap();
    println!("{:?}", vbus);

    let power = ups_hat.get_power_state().unwrap();
    println!("{:?}", power);
}
