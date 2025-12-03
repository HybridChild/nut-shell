//! Command handler trait for executing commands.
//!
//! Maps command IDs to execution functions, implementing the execution side
//! of the metadata/execution separation pattern.

use crate::config::ShellConfig;
use crate::error::CliError;
use crate::response::Response;

/// Command execution handler trait.
/// Maps command IDs to execution functions (dispatches on unique ID, not display name).
pub trait CommandHandler<C: ShellConfig> {
    /// Execute synchronous command by unique ID.
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command by unique ID (requires `async` feature).
    /// Uses `async fn` without Send bounds for both single and multi-threaded executors.
    #[cfg(feature = "async")]
    #[allow(async_fn_in_trait)]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultConfig;

    // Mock handler for testing
    struct TestHandler;

    impl CommandHandler<DefaultConfig> for TestHandler {
        fn execute_sync(
            &self,
            id: &str,
            _args: &[&str],
        ) -> Result<Response<DefaultConfig>, CliError> {
            match id {
                "test" => Ok(Response::success("OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }

        #[cfg(feature = "async")]
        async fn execute_async(
            &self,
            id: &str,
            _args: &[&str],
        ) -> Result<Response<DefaultConfig>, CliError> {
            match id {
                "async-test" => Ok(Response::success("Async OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }
    }

    #[test]
    fn test_sync_handler() {
        let handler = TestHandler;
        let result = handler.execute_sync("test", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "OK");

        let result = handler.execute_sync("unknown", &[]);
        assert_eq!(result, Err(CliError::CommandNotFound));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_handler() {
        let handler = TestHandler;
        let result = handler.execute_async("async-test", &[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "Async OK");

        let result = handler.execute_async("unknown", &[]).await;
        assert_eq!(result, Err(CliError::CommandNotFound));
    }
}
