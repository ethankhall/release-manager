pub enum ErrorCodes {
    Unknown,
    NoRepoFound,
    GitHubError,
    NetworkCallFailed,
    UnableToGetHeadSha,
    FileDoesNotExist,
}

pub struct CommandError {
    pub error_code: ErrorCodes,
    pub message: String,
}

impl CommandError {
    pub fn new<S: Into<String>>(error_code: ErrorCodes, message: S) -> CommandError {
        return CommandError {
            error_code: error_code,
            message: message.into(),
        };
    }
}
