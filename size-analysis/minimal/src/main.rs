#![no_std]
#![no_main]

use core::fmt::Write;
use nut_shell::config::MinimalConfig;
use nut_shell::tree::{CommandMeta, CommandKind, Directory, Node};
use nut_shell::{CharIo, CliError, CommandHandler, Response, Shell};
use panic_halt as _;

// Minimal access level for testing
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, nut_shell::AccessLevel)]
pub enum Level {
    User = 0,
}

// Minimal CharIo implementation - measures only struct size
pub struct MinimalIo;

impl CharIo for MinimalIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(None)
    }

    fn put_char(&mut self, _c: char) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Write for MinimalIo {
    fn write_str(&mut self, _s: &str) -> core::fmt::Result {
        Ok(())
    }
}

// Command implementations
fn status_cmd<C: nut_shell::ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    Ok(Response::success("OK"))
}

#[cfg(feature = "async")]
async fn info_cmd<C: nut_shell::ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    Ok(Response::success("Info"))
}

// Command metadata
const STATUS: CommandMeta<Level> = CommandMeta {
    id: "status",
    name: "status",
    access_level: Level::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
    description: "Show status",
};

#[cfg(feature = "async")]
const INFO: CommandMeta<Level> = CommandMeta {
    id: "info",
    name: "info",
    access_level: Level::User,
    kind: CommandKind::Async,
    min_args: 0,
    max_args: 0,
    description: "Show info",
};

// Directory tree with commands
#[cfg(feature = "async")]
const ROOT: Directory<Level> = Directory {
    name: "",
    children: &[Node::Command(&STATUS), Node::Command(&INFO)],
    access_level: Level::User,
};

#[cfg(not(feature = "async"))]
const ROOT: Directory<Level> = Directory {
    name: "",
    children: &[Node::Command(&STATUS)],
    access_level: Level::User,
};

// Minimal command handler
struct MinHandlers;

impl CommandHandler<MinimalConfig> for MinHandlers {
    fn execute_sync(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<MinimalConfig>, CliError> {
        match id {
            "status" => status_cmd::<MinimalConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(
        &self,
        id: &str,
        args: &[&str],
    ) -> Result<Response<MinimalConfig>, CliError> {
        match id {
            "info" => info_cmd::<MinimalConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Minimal credential provider for authentication feature
#[cfg(feature = "authentication")]
struct MinCredentials;

#[cfg(feature = "authentication")]
impl nut_shell::auth::CredentialProvider<Level> for MinCredentials {
    type Error = ();

    fn find_user(&self, _username: &str) -> Result<Option<nut_shell::auth::User<Level>>, Self::Error> {
        Ok(None)
    }

    fn verify_password(&self, _user: &nut_shell::auth::User<Level>, _password: &str) -> bool {
        false
    }

    fn list_users(&self) -> Result<heapless::Vec<&str, 32>, Self::Error> {
        Ok(heapless::Vec::new())
    }
}

// Entry point
#[cortex_m_rt::entry]
fn main() -> ! {
    let io = MinimalIo;
    let handlers = MinHandlers;

    #[cfg(feature = "authentication")]
    let credentials = MinCredentials;

    #[cfg(feature = "authentication")]
    let mut shell = Shell::new(&ROOT, handlers, &credentials, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell = Shell::new(&ROOT, handlers, io);

    // Activate shell to ensure all code paths are included
    // Use black_box to prevent optimizer from removing the code
    let _ = core::hint::black_box(shell.activate());

    // Process one character to ensure process_char code is included
    let _ = core::hint::black_box(shell.process_char('?'));

    // Keep shell alive to prevent optimization
    loop {
        core::hint::black_box(&shell);
        cortex_m::asm::nop();
    }
}

// Required: exception handler
#[cortex_m_rt::exception]
unsafe fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
