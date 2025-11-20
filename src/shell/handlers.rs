//! Command handler trait for executing commands.
//!
//! The `CommandHandlers` trait maps command names to execution functions,
//! implementing the execution side of the metadata/execution separation pattern.
//!
//! See [DESIGN.md](../../docs/DESIGN.md) section 1 for complete pattern explanation.

use crate::config::ShellConfig;
use crate::error::CliError;
use crate::response::Response;

/// Command execution handler trait.
///
/// Generic over `C: ShellConfig` to match Response buffer sizes.
/// Implementations map command names to execution functions.
///
/// # Pattern
///
/// Commands use metadata/execution separation:
/// - `CommandMeta` stores const metadata (name, args, access level)
/// - `CommandHandlers` provides execution logic (this trait)
///
/// # Example
///
/// ```rust,ignore
/// struct MyHandlers;
///
/// impl CommandHandlers<DefaultConfig> for MyHandlers {
///     fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
///         match name {
///             "reboot" => reboot_fn(args),
///             "status" => status_fn(args),
///             _ => Err(CliError::CommandNotFound),
///         }
///     }
/// }
/// ```
pub trait CommandHandlers<C: ShellConfig> {
    /// Execute synchronous command by name.
    ///
    /// # Arguments
    ///
    /// - `name`: Command name (already validated by Shell)
    /// - `args`: Command arguments (already validated by Shell)
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Command executed successfully
    /// - `Err(CliError::CommandNotFound)`: Command name not recognized
    /// - `Err(CliError)`: Other execution error
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command by name (requires `async` feature).
    ///
    /// # Arguments
    ///
    /// - `name`: Command name (already validated by Shell)
    /// - `args`: Command arguments (already validated by Shell)
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Command executed successfully
    /// - `Err(CliError::CommandNotFound)`: Command name not recognized
    /// - `Err(CliError)`: Other execution error
    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultConfig;

    // Mock handler for testing
    struct TestHandlers;

    impl CommandHandlers<DefaultConfig> for TestHandlers {
        fn execute_sync(&self, name: &str, _args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
            match name {
                "test" => Ok(Response::success("OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }

        #[cfg(feature = "async")]
        async fn execute_async(&self, name: &str, _args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
            match name {
                "async-test" => Ok(Response::success("Async OK")),
                _ => Err(CliError::CommandNotFound),
            }
        }
    }

    #[test]
    fn test_sync_handler() {
        let handlers = TestHandlers;
        let result = handlers.execute_sync("test", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "OK");

        let result = handlers.execute_sync("unknown", &[]);
        assert_eq!(result, Err(CliError::CommandNotFound));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_handler() {
        let handlers = TestHandlers;
        let result = handlers.execute_async("async-test", &[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().message.as_str(), "Async OK");

        let result = handlers.execute_async("unknown", &[]).await;
        assert_eq!(result, Err(CliError::CommandNotFound));
    }
}
