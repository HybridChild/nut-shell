//! Command handlers for the basic example

use core::fmt::Write;
use heapless;
use nut_shell::{
    config::DefaultConfig, response::Response, shell::handlers::CommandHandler, CliError,
};
use rp_pico_examples::{hw_commands, system_commands};

use crate::hw_state;

pub struct PicoHandlers;

impl PicoHandlers {
    fn system_info(&self) -> Result<Response<DefaultConfig>, CliError> {
        let mut msg = heapless::String::<256>::new();
        write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
        write!(msg, "Chip: RP2040\r\n").ok();
        write!(msg, "Firmware: nut-shell v0.1.0 - UART CLI Example\r\n").ok();
        write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
        Ok(Response::success(&msg).indented())
    }

    fn led_control(&self, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        let state = args[0];

        match state {
            "on" => {
                hw_state::set_led(true);
                Ok(Response::success("LED turned on").indented())
            }
            "off" => {
                hw_state::set_led(false);
                Ok(Response::success("LED turned off").indented())
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

    fn temperature(&self) -> Result<Response<DefaultConfig>, CliError> {
        let celsius = hw_state::read_temperature();
        let mut msg = heapless::String::<64>::new();
        write!(msg, "Temperature: {:.1} deg C", celsius).ok();
        Ok(Response::success(&msg).indented())
    }
}

impl CommandHandler<DefaultConfig> for PicoHandlers {
    fn execute_sync(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "system_info" => self.system_info(),
            // System diagnostic commands
            "system_uptime" => system_commands::cmd_uptime::<DefaultConfig>(args),
            "system_meminfo" => system_commands::cmd_meminfo::<DefaultConfig>(args),
            "system_benchmark" => system_commands::cmd_benchmark::<DefaultConfig>(args),
            "system_flash" => system_commands::cmd_flash::<DefaultConfig>(args),
            "system_crash" => system_commands::cmd_crash::<DefaultConfig>(args),
            // Hardware status commands
            "hw_temp" => self.temperature(),
            "hw_chipid" => hw_commands::cmd_chipid::<DefaultConfig>(args),
            "hw_clocks" => hw_commands::cmd_clocks::<DefaultConfig>(args),
            "hw_core" => hw_commands::cmd_core::<DefaultConfig>(args),
            "hw_bootreason" => hw_commands::cmd_bootreason::<DefaultConfig>(args),
            "hw_gpio" => hw_commands::cmd_gpio::<DefaultConfig>(args),
            // Hardware control commands
            "hw_led" => self.led_control(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        _id: &str,
        _args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        // basic example is synchronous-only, no async commands
        Err(CliError::CommandNotFound)
    }
}
