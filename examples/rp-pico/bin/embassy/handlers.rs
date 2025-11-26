//! Command handlers for the embassy example

use core::fmt::Write;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use heapless;
use nut_shell::{
    config::DefaultConfig, response::Response, shell::handlers::CommandHandler, CliError,
};
use rp_pico_examples::{hw_commands, system_commands};

pub enum LedCommand {
    On,
    Off,
}

pub struct PicoHandlers {
    pub led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
}

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
                    "on" => {
                        self.led_channel.try_send(LedCommand::On).ok();
                        Ok(Response::success("LED turned on").indented())
                    }
                    "off" => {
                        self.led_channel.try_send(LedCommand::Off).ok();
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
            "system_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
                write!(msg, "Chip: RP2040\r\n").ok();
                write!(msg, "Runtime: Embassy\r\n").ok();
                write!(msg, "Firmware: nut-shell v0.1.0\r\n").ok();
                write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
                Ok(Response::success(&msg).indented())
            }
            // System diagnostic commands
            "system_uptime" => system_commands::cmd_uptime::<DefaultConfig>(args),
            "system_meminfo" => system_commands::cmd_meminfo::<DefaultConfig>(args),
            "system_benchmark" => system_commands::cmd_benchmark::<DefaultConfig>(args),
            "system_flash" => system_commands::cmd_flash::<DefaultConfig>(args),
            "system_crash" => system_commands::cmd_crash::<DefaultConfig>(args),
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

    async fn execute_async(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "system_delay" => {
                // Parse delay duration
                let seconds = args[0].parse::<u64>().map_err(|_| {
                    let mut expected = heapless::String::<32>::new();
                    expected.push_str("positive integer").ok();
                    CliError::InvalidArgumentFormat {
                        arg_index: 0,
                        expected,
                    }
                })?;

                if seconds > 60 {
                    let mut msg = heapless::String::<256>::new();
                    write!(msg, "Maximum delay is 60 seconds").ok();
                    return Err(CliError::CommandFailed(msg));
                }

                // Async delay using Embassy timer
                Timer::after(Duration::from_secs(seconds)).await;

                let mut msg = heapless::String::<64>::new();
                write!(msg, "Delayed for {} second(s)", seconds).ok();
                Ok(Response::success(&msg).indented().indented())
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
