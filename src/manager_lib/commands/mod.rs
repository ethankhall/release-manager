pub mod artifactory;
pub mod github;
pub mod local;

pub use super::errors::*;

pub(crate) mod cli_shared {
    use clap::{Arg, ArgGroup, ArgMatches};
    use std::path::Path;

    use super::super::file::read_file_to_string;

    pub static GITHUB_API_TOKEN: &'static str = "github-api-token";
    pub static MESSAGE: &'static str = "message";
    pub static MESSAGE_FILE: &'static str = "message-file";

    pub(crate) fn github_token<'a, 'b>() -> Arg<'a, 'b> {
        return Arg::with_name(GITHUB_API_TOKEN)
            .long("github-token")
            .help("Auth token for Github. It's recommended to use the GITHUB_TOKEN environment variable.")
            .required(true)
            .env("GITHUB_TOKEN")
            .hide_env_values(true);
    }

    pub(crate) fn message<'a, 'b>() -> Arg<'a, 'b> {
        return Arg::with_name(MESSAGE)
            .long("message")
            .short("m")
            .help("Give a message for the tag being created")
            .takes_value(true)
            .max_values(1)
            .min_values(0);
    }

    pub(crate) fn message_file<'a, 'b>() -> Arg<'a, 'b> {
        return Arg::with_name(MESSAGE_FILE)
            .long("message-file")
            .short("F")
            .help("Read message from file for the tag being created")
            .takes_value(true)
            .number_of_values(1);
    }

    pub(crate) fn message_group<'a>() -> ArgGroup<'a> {
        return ArgGroup::with_name("messages").args(&["message", "message-file"]);
    }

    pub(crate) fn extract_message(args: &ArgMatches, default: String) -> String {
        let (message, message_file) = (args.value_of(MESSAGE), args.value_of(MESSAGE_FILE));
        let message_contents = if message_file.is_some() {
            let path = Path::new(message_file.unwrap());
            read_file_to_string(&path)
        } else if message.is_some() {
            s!(message.unwrap())
        } else {
            default
        };

        debug!("Message to be used is `{}`", message_contents);

        return message_contents;
    }
}
