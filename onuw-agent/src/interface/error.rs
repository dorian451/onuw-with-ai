use onuw_game::playerinterface::error::PlayerInterfaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Failed to parse {role} as a role.")]
    RoleParseError { role: String },

    #[error("Failed to initalize agent: {error}")]
    InitializationError { error: String },

    #[error("Can't run functions on an uninitialized agent!")]
    UninitializedError,

    #[error("Error communicating with agent: {error}")]
    CommunicationError { error: String },
}

pub type AgentResult<T> = Result<T, AgentError>;

impl From<AgentError> for PlayerInterfaceError {
    fn from(value: AgentError) -> Self {
        PlayerInterfaceError::CommunicationError(value.to_string())
    }
}
