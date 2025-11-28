//! Command handlers for the basic example

use core::fmt::Write;
use nut_shell::{
    CliError, config::DefaultConfig, response::Response, shell::handlers::CommandHandler,
};

pub struct ExampleHandlers;

impl CommandHandler<DefaultConfig> for ExampleHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "coffee_make" => Ok(Response::success("Brewing coffee...")),
            "system_reboot" => Ok(Response::success("System rebooting...\r\nGoodbye!")),
            "system_status" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "System Status:\r\n").ok();
                write!(msg, "  CPU Usage: 23%\r\n").ok();
                write!(msg, "  Memory: 45% used\r\n").ok();
                write!(msg, "  Uptime: 42 hours").ok();
                Ok(Response::success(&msg))
            }
            "system_version" => Ok(Response::success(
                "nut-shell v0.1.0\r\nRust embedded CLI framework",
            )),
            "config_get" => {
                let key = args[0];
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Config[{}] = <simulated value>", key).ok();
                Ok(Response::success(&msg))
            }
            "config_set" => {
                let key = args[0];
                let value = args[1];
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Config[{}] set to '{}'", key, value).ok();
                Ok(Response::success(&msg))
            }
            "echo" => {
                if args.is_empty() {
                    Ok(Response::success(""))
                } else {
                    let mut msg = heapless::String::<256>::new();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            msg.push(' ').ok();
                        }
                        msg.push_str(arg).ok();
                    }
                    Ok(Response::success(&msg))
                }
            }
            "uptime" => Ok(Response::success("System uptime: 42 hours, 13 minutes")),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        id: &str,
        _args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        // This example doesn't use async commands
        let mut msg = heapless::String::<256>::new();
        write!(msg, "Async command '{}' not supported in this example", id).ok();
        Err(CliError::Other(msg))
    }
}
