//! Command handler trait for executing commands.
//!
//! Maps command IDs to execution functions, implementing the execution side
//! of the metadata/execution separation pattern.

use crate::config::ShellConfig;
use crate::error::CliError;
use crate::response::Response;
use crate::shell::cmdctx::CommandContext;

/// Command execution handler trait.
/// Maps command IDs to execution functions (dispatches on unique ID, not display name).
pub trait CommandHandler<C: ShellConfig> {
    /// Execute synchronous command by unique ID.
    fn execute_sync(&self, ctx: CommandContext) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command by unique ID (requires `async` feature).
    /// Uses `async fn` without Send bounds for both single and multi-threaded executors.
    #[cfg(feature = "async")]
    #[allow(async_fn_in_trait)]
    async fn execute_async(&self, ctx: CommandContext) -> Result<Response<C>, CliError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultConfig;

    // Mock handler for testing
    struct TestHandler;

    impl CommandHandler<DefaultConfig> for TestHandler {
        fn execute_sync(&self, ctx: CommandContext) -> Result<Response<DefaultConfig>, CliError> {
            match ctx.id {
                "test" => Ok(Response::success("OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }

        #[cfg(feature = "async")]
        async fn execute_async(
            &self,
            ctx: CommandContext<'_>,
        ) -> Result<Response<DefaultConfig>, CliError> {
            match ctx.id {
                "async-test" => Ok(Response::success("Async OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }
    }

    #[test]
    fn test_sync_handler() {
        let handler = TestHandler;
        let ctx = CommandContext::new(
            "test",
            &[],
            #[cfg(feature = "context-path")]
            &[],
        );
        let result = handler.execute_sync(ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "OK");

        let ctx = CommandContext::new(
            "unknown",
            &[],
            #[cfg(feature = "context-path")]
            &[],
        );
        let result = handler.execute_sync(ctx);
        assert_eq!(result, Err(CliError::CommandNotFound));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_handler() {
        let handler = TestHandler;
        let ctx = CommandContext::new(
            "async-test",
            &[],
            #[cfg(feature = "context-path")]
            &[],
        );
        let result = handler.execute_async(ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "Async OK");

        let ctx = CommandContext::new(
            "unknown",
            &[],
            #[cfg(feature = "context-path")]
            &[],
        );
        let result = handler.execute_async(ctx).await;
        assert_eq!(result, Err(CliError::CommandNotFound));
    }
}
