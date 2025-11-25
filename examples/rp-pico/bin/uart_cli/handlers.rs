//! Command handlers for the uart_cli example

use core::fmt::Write;
use heapless;
use nut_shell::{
    config::DefaultConfig, response::Response, shell::handlers::CommandHandler, CliError,
};
use rp_pico_examples::hw_commands;

pub struct PicoHandlers;

impl CommandHandler<DefaultConfig> for PicoHandlers {
    fn execute_sync(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "hw_led" => hw_commands::cmd_led::<DefaultConfig>(args),
            "system_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
                write!(msg, "Chip: RP2040\r\n").ok();
                write!(msg, "Firmware: nut-shell v0.1.0\r\n").ok();
                write!(msg, "  - UART CLI example\r\n").ok();
                write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
                Ok(Response::success(&msg).indented())
            }
            "system_reboot" => {
                // In a real implementation, trigger watchdog reset
                Ok(Response::success(
                    "Rebooting...\r\n(Not implemented in example)",
                ))
            }
            // Hardware status commands
            "hw_temp" => hw_commands::cmd_temp::<DefaultConfig>(args),
            "hw_chipid" => hw_commands::cmd_chipid::<DefaultConfig>(args),
            "hw_clocks" => hw_commands::cmd_clocks::<DefaultConfig>(args),
            "hw_core" => hw_commands::cmd_core::<DefaultConfig>(args),
            "hw_bootreason" => hw_commands::cmd_bootreason::<DefaultConfig>(args),
            "hw_gpio" => hw_commands::cmd_gpio::<DefaultConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}
