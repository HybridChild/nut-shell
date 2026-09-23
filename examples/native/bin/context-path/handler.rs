//! Command handler for the context-path example

use core::fmt::Write;
use nut_shell::{
    CliError, config::DefaultConfig, response::Response, shell::cmdctx::CommandContext,
    shell::handler::CommandHandler,
};

pub struct ExampleHandler;

impl CommandHandler<DefaultConfig> for ExampleHandler {
    fn execute_sync(&self, ctx: CommandContext) -> Result<Response<DefaultConfig>, CliError> {
        match ctx.id {
            "shared_showpath" => {
                let mut msg = heapless::String::<256>::new();
                write!(msg, "Current Path Array: {:?}\r\n", ctx.path).ok();
                write!(
                    msg,
                    "Current Path With Seperator: {:?}\r\n",
                    ctx.get_path_withsep().unwrap()
                )
                .ok();
                Ok(Response::success(&msg).indented())
            }
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        ctx: CommandContext<'_>,
    ) -> Result<Response<DefaultConfig>, CliError> {
        // This example doesn't use async commands
        let mut msg = heapless::String::<128>::new();
        write!(
            msg,
            "Async command '{}' not supported in this example",
            ctx.id
        )
        .ok();
        Err(CliError::Other(msg))
    }
}
