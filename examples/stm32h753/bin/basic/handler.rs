//! Command handler for the NUCLEO-H753ZI example

use core::fmt::Write;
use nut_shell::{
    CliError, config::ShellConfig, response::Response, shell::handler::CommandHandler,
};
use stm32h753zi_examples::{hw_commands, system_commands};

use crate::{hw_state, systick};

pub struct H753Handler;

impl H753Handler {
    fn system_info<C: ShellConfig>(&self) -> Result<Response<C>, CliError> {
        let mut msg = heapless::String::<256>::new();
        write!(msg, "Board:    NUCLEO-H753ZI\r\n").ok();
        write!(msg, "MCU:      STM32H753ZIT6\r\n").ok();
        write!(msg, "Core:     Cortex-M7 @ 200 MHz (PLL1, VOS1)\r\n").ok();
        write!(msg, "Flash:    2 MB (dual-bank)\r\n").ok();
        write!(msg, "RAM:      1 MB (AXI/DTCM/ITCM)\r\n").ok();
        write!(msg, "I/O:      USB CDC (OTG2_HS, CN13)\r\n").ok();
        write!(msg, "Firmware: nut-shell example").ok();
        Ok(Response::success(&msg).indented())
    }

    fn uptime<C: ShellConfig>(&self) -> Result<Response<C>, CliError> {
        let ms = systick::millis() as u64;
        let secs = ms / 1000;
        let mins = secs / 60;
        let hours = mins / 60;
        let days = hours / 24;

        let mut msg = heapless::String::<128>::new();
        write!(
            msg,
            "Uptime: {}d {}h {}m {}s\r\nTotal:  {} seconds",
            days,
            hours % 24,
            mins % 60,
            secs % 60,
            secs
        )
        .ok();

        Ok(Response::success(&msg).indented())
    }

    fn led_control<C: ShellConfig>(&self, args: &[&str]) -> Result<Response<C>, CliError> {
        let n: u8 = match args[0] {
            "1" => 1,
            "2" => 2,
            "3" => 3,
            _ => {
                let mut expected = heapless::String::<32>::new();
                expected.push_str("1, 2, or 3").ok();
                return Err(CliError::InvalidArgumentFormat {
                    arg_index: 0,
                    expected,
                });
            }
        };

        match args[1] {
            "on" => {
                hw_state::set_led(n, true);
                let mut msg = heapless::String::<32>::new();
                write!(msg, "LED {} on", n).ok();
                Ok(Response::success(&msg).indented())
            }
            "off" => {
                hw_state::set_led(n, false);
                let mut msg = heapless::String::<32>::new();
                write!(msg, "LED {} off", n).ok();
                Ok(Response::success(&msg).indented())
            }
            "toggle" => {
                hw_state::toggle_led(n);
                let mut msg = heapless::String::<32>::new();
                write!(msg, "LED {} toggled", n).ok();
                Ok(Response::success(&msg).indented())
            }
            _ => {
                let mut expected = heapless::String::<32>::new();
                expected.push_str("on, off, or toggle").ok();
                Err(CliError::InvalidArgumentFormat {
                    arg_index: 1,
                    expected,
                })
            }
        }
    }
}

impl<C: ShellConfig> CommandHandler<C> for H753Handler {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match id {
            "system_info" => self.system_info(),
            "system_uptime" => self.uptime::<C>(),
            "system_meminfo" => system_commands::cmd_meminfo::<C>(args),
            "system_benchmark" => system_commands::cmd_benchmark::<C>(args),
            "system_flash" => system_commands::cmd_flash::<C>(args),
            "system_crash" => system_commands::cmd_crash::<C>(args),
            "hw_chipid" => hw_commands::cmd_chipid::<C>(args),
            "hw_clocks" => hw_commands::cmd_clocks::<C>(args),
            "hw_core" => hw_commands::cmd_core::<C>(args),
            "hw_bootreason" => hw_commands::cmd_bootreason::<C>(args),
            "hw_led" => self.led_control(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, _id: &str, _args: &[&str]) -> Result<Response<C>, CliError> {
        Err(CliError::CommandNotFound)
    }
}
