//! Command handlers for the uart_cli example

use core::fmt::Write;
use heapless;
use nut_shell::{
    config::DefaultConfig, response::Response, shell::handlers::CommandHandler, CliError,
};

pub struct PicoHandlers;

impl CommandHandler<DefaultConfig> for PicoHandlers {
    fn execute_sync(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "led" => {
                let state = args[0];
                match state {
                    "on" | "off" => {
                        // In a real implementation, you would control the LED here
                        // For now, just acknowledge the command
                        let mut msg = heapless::String::<128>::new();
                        write!(msg, "LED turned {}", state).ok();
                        Ok(Response::success(&msg))
                    }
                    _ => {
                        let mut expected = heapless::String::<32>::new();
                        expected.push_str("on or off").ok();
                        Err(CliError::InvalidArgumentFormat {
                            arg_index: 0,
                            expected,
                        })
                    }
                }
            }
            "system_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
                write!(msg, "Chip: RP2040\r\n").ok();
                write!(msg, "Firmware: nut-shell v0.1.0\r\n").ok();
                write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
                Ok(Response::success(&msg))
            }
            "system_reboot" => {
                // In a real implementation, trigger watchdog reset
                Ok(Response::success(
                    "Rebooting...\r\n(Not implemented in example)",
                ))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
