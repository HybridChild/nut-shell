//! Command handler for the async example

use core::fmt::Write;
use nut_shell::{
    CliError, config::DefaultConfig, response::Response, shell::handler::CommandHandler,
};
use tokio::time::{Duration, sleep};

pub struct AsyncHandler;

impl CommandHandler<DefaultConfig> for AsyncHandler {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "sync_echo" => {
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
                    Ok(Response::success(&msg).indented())
                }
            }
            "sync_info" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "nut-shell Async Example\r\n").ok();
                write!(msg, "Runtime: Tokio\r\n").ok();
                write!(msg, "Features: async commands, authentication\r\n").ok();
                write!(msg, "Try the 'async' directory for async commands!").ok();
                Ok(Response::success(&msg).indented())
            }
            "sync_reboot" => Ok(Response::success("System rebooting...\r\nGoodbye!").indented()),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "async_delay" => {
                // Parse delay duration
                let seconds = args[0].parse::<u64>().map_err(|_| {
                    let mut expected = heapless::String::<32>::new();
                    expected.push_str("positive integer").ok();
                    CliError::InvalidArgumentFormat {
                        arg_index: 0,
                        expected,
                    }
                })?;

                if seconds > 30 {
                    let mut msg = heapless::String::<128>::new();
                    write!(msg, "Maximum delay is 30 seconds").ok();
                    return Err(CliError::CommandFailed(msg));
                }

                // Show starting message
                let mut start_msg = heapless::String::<128>::new();
                write!(start_msg, "Starting {}s delay...", seconds).ok();

                // Perform async delay
                sleep(Duration::from_secs(seconds)).await;

                // Return completion message
                let mut msg = heapless::String::<64>::new();
                write!(msg, "Delayed for {} second(s)", seconds).ok();
                Ok(Response::success(&msg).indented())
            }
            "async_fetch" => {
                let url = args[0];

                // Simulate async HTTP fetch
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Fetching '{}'...\r\n", url).ok();

                // Simulate network delay
                sleep(Duration::from_millis(500)).await;

                write!(msg, "Response: 200 OK\r\n").ok();
                write!(msg, "Content-Length: 1234\r\n").ok();
                write!(msg, "Fetch completed successfully!").ok();

                Ok(Response::success(&msg).indented())
            }
            "async_compute" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Starting async computation...\r\n").ok();

                // Simulate some async work with periodic delays
                for i in 1..=3 {
                    sleep(Duration::from_millis(300)).await;
                    write!(msg, "Step {}/3 completed\r\n", i).ok();
                }

                write!(msg, "Computation finished!").ok();
                Ok(Response::success(&msg).indented())
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
