use std::vec::Vec;

use clap::{App, ArgMatches};

use super::{is_path_repo, get_base_url};
use super::super::requests::CromApi;
use super::super::Repo;

pub fn version_clap<'a,'b>() -> App<'a, 'b> {
    return clap_app!( @subcommand version =>
        (about: "Operates on versions of projects.")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand list =>
            (about: "List the versions avaliable")
            (@arg path: +required +takes_value { |x| is_path_repo(x) } "The GitHub style path to the repo. Example: ethankhall/version-management")
            (@arg output_format: --("output-format") +takes_value {
                |x| match x.as_str() {
                    "json" => Ok(()),
                    "plain" => Ok(()),
                    _ => Err(s!("Format must be json or plain."))
                }
            })
        )
    );
}

pub fn process_version_command(args: &ArgMatches) -> i32 {
    return match args.subcommand() {
        ("list",  Some(sub_m)) => { list(sub_m) },
        _ => { 
            error!("No command avaliable. {:?}", args); 
            -1
        }
    };
}

fn list(args: &ArgMatches) -> i32 {
    let crom_api = CromApi::new(get_base_url(args), None);
    let split: Vec<&str> = args.value_of("path").unwrap().split("/").collect();
    let project = split[0];
    let repo = split[1];


    let repo = Repo { project: s!(project), repo: s!(repo) };
    let response = crom_api.do_get(&repo, vec!["versions"]);
    debug!("Got a response: {}", response.to_string());
    trace!("Got response: {:?}", response);


    return 0;
}