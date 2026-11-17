// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Will forcefully power-off the Raspberry Pi connected to the Waveshare UPS Hat E.
//!
//! Exit codes:
//!
//! - `0`: the UPS confirmed the power-off; the Raspberry Pi loses power in 30 seconds
//! - `1`: invalid arguments, or the confirmation prompt was declined; no command was sent
//! - `2`: the UPS accepted the power-off command, but reports that no power-off is pending
//! - `3`: the UPS accepted the power-off command, but its status could not be read back; assume
//!   the Raspberry Pi loses power in 30 seconds
use std::env;
use std::io::Write;
use std::process::exit;
use waveshare_ups_hat_e::UpsHatE;
use waveshare_ups_hat_e::error::Error;

const EXIT_POWER_OFF_PENDING: i32 = 0;
const EXIT_NOT_SENT: i32 = 1;
const EXIT_POWER_OFF_NOT_PENDING: i32 = 2;
const EXIT_POWER_OFF_UNVERIFIED: i32 = 3;

/// How the power-off is confirmed, as selected by the command line arguments.
#[derive(Debug, PartialEq, Eq)]
enum Confirmation {
    /// Ask on the terminal before powering off.
    Prompt,
    /// `-y` was given: power-off without asking.
    Skip,
}

/// Parses the command line arguments (excluding the program name).
///
/// Returns `None` for anything other than no arguments or exactly `-y`. An invocation that is
/// not understood must never reach the confirmation prompt, let alone the power-off.
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

/// Turns the status read that follows an accepted power-off command into the exit code and the
/// message for the user.
///
/// The power-off cannot be canceled once the UPS accepts the command, so a failed status read
/// must still warn about the countdown.
fn power_off_report(status: Result<bool, Error>) -> (i32, String) {
    match status {
        Ok(true) => (
            EXIT_POWER_OFF_PENDING,
            "UPS will power-off the attached Raspberry Pi in 30 seconds".to_string(),
        ),
        Ok(false) => (
            EXIT_POWER_OFF_NOT_PENDING,
            "Error: UPS failed to initiate power-off".to_string(),
        ),
        Err(err) => (
            EXIT_POWER_OFF_UNVERIFIED,
            format!(
                "Warning: UPS accepted the power-off command, but reading its status failed: {err}\n\
                 Assume the UPS will power-off the attached Raspberry Pi in 30 seconds"
            ),
        ),
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let Some(confirmation) = parse_args(&args) else {
        println!("Usage: force_power_off [-y]");
        println!("  -y: skip confirmation prompt");
        println!();
        exit(EXIT_NOT_SENT);
    };

    if confirmation == Confirmation::Prompt && !prompt_for_confirmation() {
        println!("Aborting power-off due to user input. Use -y to skip confirmation prompt.");
        exit(EXIT_NOT_SENT);
    }

    let mut ups = UpsHatE::new().expect("failed to open I2C device");

    ups.force_power_off()
        .expect("failed to issue power-off command");

    let (exit_code, message) = power_off_report(ups.is_power_off_pending());
    println!("{message}");
    exit(exit_code);
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
        let (exit_code, message) = power_off_report(Ok(true));
        assert_eq!(exit_code, EXIT_POWER_OFF_PENDING);
        assert!(message.contains("30 seconds"), "message: {message}");
    }

    #[test]
    fn power_off_not_pending_is_an_error() {
        let (exit_code, message) = power_off_report(Ok(false));
        assert_eq!(exit_code, EXIT_POWER_OFF_NOT_PENDING);
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

        let (exit_code, message) = power_off_report(Err(read_failure));
        assert_eq!(exit_code, EXIT_POWER_OFF_UNVERIFIED);
        assert!(message.contains("30 seconds"), "message: {message}");
        assert!(message.contains(&cause), "message: {message}");
    }
}
