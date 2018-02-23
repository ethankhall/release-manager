pub mod versions;

struct CommandError {
    pub(crate) error_code: i32,
    pub(crate) message: String
}

impl CommandError {
    fn new(error_code: i32, message: String) -> CommandError {
        return CommandError { error_code: error_code, message: message}
    }

    fn new_no_body(error_code: i32) -> CommandError {
        return CommandError::new(error_code, s!("Unknown Error"));
    }
}