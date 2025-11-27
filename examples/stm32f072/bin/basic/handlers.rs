//! Command handlers for the NUCLEO-F072RB example

use core::fmt::Write;
use nut_shell::{
    config::ShellConfig, response::Response, shell::handlers::CommandHandler, CliError,
};
use stm32_examples::hw_commands;

use crate::hw_state;

pub struct Stm32Handlers;

impl Stm32Handlers {
    fn system_info<C: ShellConfig>(&self) -> Result<Response<C>, CliError> {
        // Use a buffer size that fits within MinimalConfig's MAX_RESPONSE (128 bytes)
        let mut msg = heapless::String::<128>::new();
        write!(msg, "Device: NUCLEO-F072RB\r\n").ok();
        write!(msg, "Chip: STM32F072RBT6\r\n").ok();
        write!(msg, "Core: Cortex-M0\r\n").ok();
        write!(msg, "Firmware: nut-shell\r\n").ok();
        write!(msg, "UART: 115200 baud").ok();
        Ok(Response::success(&msg).indented())
    }

    fn temperature<C: ShellConfig>(&self) -> Result<Response<C>, CliError> {
        let celsius = hw_state::read_temperature();
        let mut msg = heapless::String::<64>::new();
        write!(msg, "Temperature: {:.1} deg C", celsius).ok();
        Ok(Response::success(&msg).indented())
    }

    fn led_control<C: ShellConfig>(&self, args: &[&str]) -> Result<Response<C>, CliError> {
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
}

impl<C: ShellConfig> CommandHandler<C> for Stm32Handlers {
    fn execute_sync(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<C>, CliError> {
        match id {
            "system_info" => self.system_info(),
            // Hardware status commands
            "hw_temp" => self.temperature(),
            "hw_chipid" => hw_commands::cmd_chipid::<C>(args),
            "hw_clocks" => hw_commands::cmd_clocks::<C>(args),
            "hw_core" => hw_commands::cmd_core::<C>(args),
            "hw_bootreason" => hw_commands::cmd_bootreason::<C>(args),
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
    ) -> Result<Response<C>, CliError> {
        // Basic example is synchronous-only, no async commands
        Err(CliError::CommandNotFound)
    }
}
