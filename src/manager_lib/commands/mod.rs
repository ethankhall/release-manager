pub mod project;
pub mod github;

pub use super::errors::*;

pub(crate) mod cli_shared {
    use clap::{Arg, ArgGroup, ArgMatches};
    use std::path::Path;

    use super::super::file::read_file;

    // static GITHUB_API_USER: &'static str = "github-api-user";
    pub static GITHUB_API_TOKEN: &'static str = "github-api-token";
    pub static GITHUB_PATH: &'static str = "github-path";
    pub static MESSAGE: &'static str = "message";
    pub static MESSAGE_FILE: &'static str = "message-file";

    // pub(crate) fn github_user<'a, 'b>() -> Arg<'a, 'b> {
    //     return Arg::with_name(GITHUB_API_USER)
    //         .long("github-user")
    //         .help("GitHub user to do action.")
    //         .required(true)
    //         .env("GITHUB_USER");
    // }

    pub(crate) fn github_token<'a, 'b>() -> Arg<'a, 'b> {
        return Arg::with_name(GITHUB_API_TOKEN)
            .long("github-token")
            .help("Auth token for Github. It's recommended to use the GITHUB_TOKEN environment variable.")
            .required(true)
            .env("GITHUB_TOKEN")
            .hide_default_value(true);
    }

    pub(crate) fn github_path<'a, 'b>() -> Arg<'a, 'b> {
        return Arg::with_name(GITHUB_PATH)
            .long("path")
            .help("GitHub path to project. Like: ethankhall/release-manager")
            .takes_value(true)
            .number_of_values(2)
            .use_delimiter(true)
            .required(true)
            .value_delimiter("/");
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
            read_file(&path)
        } else if message.is_some() {
            s!(message.unwrap())
        } else {
            default
        };

        debug!("Message to be used is `{}`", message_contents);

        return message_contents;
    }
}
