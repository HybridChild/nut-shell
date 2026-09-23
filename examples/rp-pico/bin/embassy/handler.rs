//! Command handler for the embassy example

use core::fmt::Write;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use heapless;
use nut_shell::{
    CliError, config::DefaultConfig, response::Response, shell::cmdctx::CommandContext,
    shell::handler::CommandHandler,
};
use rp_pico_examples::{hw_commands, system_commands};

use crate::hw_state;

pub enum LedCommand {
    On,
    Off,
}

pub struct PicoHandler {
    pub led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
}

impl PicoHandler {
    fn system_info(&self) -> Result<Response<DefaultConfig>, CliError> {
        let mut msg = heapless::String::<256>::new();
        write!(msg, "Device: Raspberry Pi Pico\r\n").ok();
        write!(msg, "Chip: RP2040\r\n").ok();
        write!(msg, "Runtime: Embassy\r\n").ok();
        write!(msg, "Firmware: nut-shell v0.1.0 - UART CLI Example\r\n").ok();
        write!(msg, "UART: GP0(TX)/GP1(RX) @ 115200").ok();
        Ok(Response::success(&msg).indented())
    }

    fn signal_led(&self, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
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

    fn temperature(&self) -> Result<Response<DefaultConfig>, CliError> {
        let celsius = hw_state::read_temperature();
        let mut msg = heapless::String::<64>::new();
        write!(msg, "Temperature: {:.1}°C", celsius).ok();
        Ok(Response::success(&msg).indented())
    }
}

impl CommandHandler<DefaultConfig> for PicoHandler {
    fn execute_sync(&self, ctx: CommandContext) -> Result<Response<DefaultConfig>, CliError> {
        match ctx.id {
            "system_info" => self.system_info(),
            // System diagnostic commands
            "system_uptime" => system_commands::cmd_uptime::<DefaultConfig>(ctx.args),
            "system_meminfo" => system_commands::cmd_meminfo::<DefaultConfig>(ctx.args),
            "system_benchmark" => system_commands::cmd_benchmark::<DefaultConfig>(ctx.args),
            "system_flash" => system_commands::cmd_flash::<DefaultConfig>(ctx.args),
            "system_crash" => system_commands::cmd_crash::<DefaultConfig>(ctx.args),
            // Hardware status commands
            "hw_temp" => self.temperature(),
            "hw_chipid" => hw_commands::cmd_chipid::<DefaultConfig>(ctx.args),
            "hw_clocks" => hw_commands::cmd_clocks::<DefaultConfig>(ctx.args),
            "hw_core" => hw_commands::cmd_core::<DefaultConfig>(ctx.args),
            "hw_bootreason" => hw_commands::cmd_bootreason::<DefaultConfig>(ctx.args),
            "hw_gpio" => hw_commands::cmd_gpio::<DefaultConfig>(ctx.args),
            // Hardware control commands
            "led" => self.signal_led(ctx.args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    async fn execute_async(
        &self,
        ctx: CommandContext<'_>,
    ) -> Result<Response<DefaultConfig>, CliError> {
        match ctx.id {
            "system_delay" => {
                // Parse delay duration
                let seconds = ctx.args[0].parse::<u64>().map_err(|_| {
                    let mut expected = heapless::String::<32>::new();
                    expected.push_str("positive integer").ok();
                    CliError::InvalidArgumentFormat {
                        arg_index: 0,
                        expected,
                    }
                })?;

                if seconds > 60 {
                    let mut msg = heapless::String::<128>::new();
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
