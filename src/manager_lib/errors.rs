#[derive(Debug)]
pub enum ErrorCodes {
    Unknown,
    NoRepoFound,
    GitHubError,
    NetworkCallFailed,
    UnableToGetHeadSha,
    UnableToBumpVersion,
    UnableToFindBranchNameForSha,
    FileDoesNotExist,
    ArtifactorySectionDoesNotExist,
    RepoNotValid,
    ArtifactoryCommunicationFailed
}

pub struct CommandError {
    pub error_code: ErrorCodes,
    pub message: String,
}

impl CommandError {
    pub fn new<S: Into<String>>(error_code: ErrorCodes, message: S) -> CommandError {
        return CommandError {
            error_code,
            message: message.into(),
        };
    }
}
