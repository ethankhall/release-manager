pub enum ErrorCodes {
    Unknown,
    NoRepoFound,
    GitHubError,
    FileDoesNotExist
}

pub struct CommandError {
    pub error_code: ErrorCodes,
    pub message: String
}

impl CommandError {
    pub fn new(error_code: ErrorCodes, message: String) -> CommandError {
        return CommandError { error_code: error_code, message: message}
    }
}