//! Command handler trait for executing commands.
//!
//! The `CommandHandler` trait maps command IDs to execution functions,
//! implementing the execution side of the metadata/execution separation pattern.
//!
//! See [DESIGN.md](../../docs/DESIGN.md) section 1 for complete pattern explanation.

use crate::config::ShellConfig;
use crate::error::CliError;
use crate::response::Response;

/// Command execution handler trait.
///
/// Generic over `C: ShellConfig` to match Response buffer sizes.
/// Implementations map command IDs to execution functions.
///
/// # Pattern
///
/// Commands use metadata/execution separation:
/// - `CommandMeta` stores const metadata (id, name, args, access level)
/// - `CommandHandler` provides execution logic (this trait)
///
/// The handler dispatches on the unique command ID, not the display name.
/// This allows multiple commands with the same name in different directories.
///
/// # Example
///
/// ```rust,ignore
/// struct MyHandlers;
///
/// impl CommandHandler<DefaultConfig> for MyHandlers {
///     fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
///         match id {
///             "system_reboot" => reboot_fn(args),
///             "system_status" => status_fn(args),
///             _ => Err(CliError::CommandNotFound),
///         }
///     }
/// }
/// ```
pub trait CommandHandler<C: ShellConfig> {
    /// Execute synchronous command by unique ID.
    ///
    /// # Arguments
    ///
    /// - `id`: Unique command identifier from `CommandMeta.id`
    /// - `args`: Command arguments (already validated by Shell)
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Command executed successfully
    /// - `Err(CliError::CommandNotFound)`: Command ID not recognized
    /// - `Err(CliError)`: Other execution error
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command by unique ID (requires `async` feature).
    ///
    /// # Arguments
    ///
    /// - `id`: Unique command identifier from `CommandMeta.id`
    /// - `args`: Command arguments (already validated by Shell)
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Command executed successfully
    /// - `Err(CliError::CommandNotFound)`: Command ID not recognized
    /// - `Err(CliError)`: Other execution error
    ///
    /// # Implementation Note
    ///
    /// This uses `async fn` in trait without Send bounds to support both:
    /// - Single-threaded embedded executors (Embassy) where Send isn't required
    /// - Multi-threaded native executors (Tokio) where implementations can be Send
    ///
    /// Users needing Send bounds for multi-threaded spawning can verify this
    /// at the call site.
    #[cfg(feature = "async")]
    #[allow(async_fn_in_trait)]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultConfig;

    // Mock handler for testing
    struct TestHandlers;

    impl CommandHandler<DefaultConfig> for TestHandlers {
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
