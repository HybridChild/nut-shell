//! Command handlers for the embassy_uart_cli example

use core::fmt::Write;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use heapless;
use nut_shell::{
    config::DefaultConfig, response::Response, shell::handlers::CommandHandler, CliError,
};

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
                        Ok(Response::success("LED turned on"))
                    }
                    "off" => {
                        self.led_channel.try_send(LedCommand::Off).ok();
                        Ok(Response::success("LED turned off"))
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
                Ok(Response::success(&msg))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
