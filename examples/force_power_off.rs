// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Forcefully power-off the Raspberry Pi connected to the Waveshare UPS Hat E.
//!
//! See [ExitCode] for the meaning of exit code values.

use std::env;
use std::io::Write;
use waveshare_ups_hat_e::UpsHatE;
use waveshare_ups_hat_e::error::Error;

/// Exit codes reported by this program. Scripts can use them to tell whether the Raspberry Pi
/// is about to lose power.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitCode {
    /// `0`: the UPS confirmed the power-off; the Raspberry Pi will lose power in 30 seconds
    PowerOffPending = 0,
    /// `1`: invalid arguments, or confirmation prompt was declined; no command sent; no power off
    NotSent = 1,
    /// `2`: the UPS accepted the power-off command, but reports that no power-off is pending
    NoPowerOff = 2,
    /// `3`: the UPS accepted the power-off command, but its status could not be read back; assume
    /// the Raspberry Pi will lose power in 30 seconds
    Unverified = 3,
}

impl ExitCode {
    fn exit(self) -> ! {
        std::process::exit(self as i32)
    }
}

/// How the power-off is confirmed, as selected by the command line arguments.
#[derive(Debug, PartialEq, Eq)]
enum Confirmation {
    /// Ask on the terminal before powering off.
    Prompt,
    /// `-y` was given: power-off without asking.
    Skip,
}

fn parse_args(args: &[String]) -> Option<Confirmation> {
    match args {
        [] => Some(Confirmation::Prompt),
        [flag] if flag.eq_ignore_ascii_case("-y") => Some(Confirmation::Skip),
        _ => None,
    }
}

fn prompt_for_confirmation() -> bool {
    print!("Are you sure you want to power-off the Raspberry Pi? [y/N] ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");

    input.trim().eq_ignore_ascii_case("y")
}

/// Turns status into a message and exit code.
fn message_and_exit_code(status: Result<bool, Error>) -> (String, ExitCode) {
    match status {
        Ok(true) => (
            "UPS will power-off the attached Raspberry Pi in 30 seconds".to_string(),
            ExitCode::PowerOffPending,
        ),
        Ok(false) => (
            "Error: UPS failed to initiate power-off".to_string(),
            ExitCode::NoPowerOff,
        ),
        Err(err) => (
            format!(
                "Warning: UPS accepted the power-off command, but reading its status failed: {err}\n\
                 Assume the UPS will power-off the attached Raspberry Pi in 30 seconds"
            ),
            ExitCode::Unverified,
        ),
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let Some(confirmation) = parse_args(&args) else {
        println!("Usage: force_power_off [-y]");
        println!("  -y: skip confirmation prompt");
        println!();
        ExitCode::NotSent.exit();
    };

    if confirmation == Confirmation::Prompt && !prompt_for_confirmation() {
        println!("Aborting power-off due to user input. Use -y to skip confirmation prompt.");
        ExitCode::NotSent.exit();
    }

    let mut ups = UpsHatE::new().expect("failed to open UPS device");

    ups.force_power_off()
        .expect("failed to issue power-off command");

    let (message, exit_code) = message_and_exit_code(ups.is_power_off_pending());
    println!("{message}");
    exit_code.exit();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn no_arguments_prompts() {
        assert_eq!(parse_args(&args(&[])), Some(Confirmation::Prompt));
    }

    #[test]
    fn dash_y_alone_skips_the_prompt() {
        assert_eq!(parse_args(&args(&["-y"])), Some(Confirmation::Skip));
        assert_eq!(parse_args(&args(&["-Y"])), Some(Confirmation::Skip));
    }

    #[test]
    fn anything_else_is_rejected() {
        for invalid in [
            &["--help"][..],
            &["-h"],
            &["y"],
            &[""],
            &["--help", "extra"],
            &["-y", "extra"],
            &["extra", "-y"],
            &["-y", "-y"],
            &["a", "b", "c"],
        ] {
            assert_eq!(parse_args(&args(invalid)), None, "args: {invalid:?}");
        }
    }

    #[test]
    fn pending_power_off_reports_the_countdown() {
        let (message, exit_code) = message_and_exit_code(Ok(true));
        assert_eq!(exit_code, ExitCode::PowerOffPending);
        assert!(message.contains("30 seconds"), "message: {message}");
    }

    #[test]
    fn power_off_not_pending_is_an_error() {
        let (message, exit_code) = message_and_exit_code(Ok(false));
        assert_eq!(exit_code, ExitCode::NoPowerOff);
        assert!(message.starts_with("Error:"), "message: {message}");
    }

    #[test]
    fn failed_status_read_still_warns_about_the_countdown() {
        let read_failure = Error::InvalidDataLen {
            register: 0x01,
            expected: 1,
            actual: 0,
        };
        let cause = read_failure.to_string();

        let (message, exit_code) = message_and_exit_code(Err(read_failure));
        assert_eq!(exit_code, ExitCode::Unverified);
        assert!(message.contains("30 seconds"), "message: {message}");
        assert!(message.contains(&cause), "message: {message}");
    }
}
