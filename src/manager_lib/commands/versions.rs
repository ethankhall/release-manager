use std::env;

use clap::{App, ArgMatches, AppSettings, SubCommand, Arg, ArgGroup};

use super::super::repo::{DefaultRepo, Repo};

pub fn version_clap<'a,'b>() -> App<'a, 'b> {
    let list_command = SubCommand::with_name("list")
            .about("List the versions avaliable")
            .arg(Arg::with_name("output-format")
                .long("output-format")
                .help("Output the data in different formats for further processing")
                .possible_values(&["json", "plain"])
                .default_value("plain"));
    
    let create_command = SubCommand::with_name("create")
        .about("Create a new version")
        .arg(Arg::with_name("REV")
            .help("The revision to use for the version")
            .default_value("HEAD"))
        .arg(Arg::with_name("set-version")
            .long("set-major")
            .help("Specify the version to create.")
            .takes_value(true))
        .arg(Arg::with_name("bump-major")
            .long("bump-major")
            .help("Bump the major component of the version"))
        .arg(Arg::with_name("bump-minor")
            .long("bump-minor")
            .help("Bump the minor component of the version"))
        .arg(Arg::with_name("bump-patch")
            .long("bump-patch")
            .help("Bump the patch component of the version"))
        .group(ArgGroup::with_name("version-options")
            .required(true)
            .args(&["set-version", "bump-major", "bump-minor", "bump-patch"]));
    
    return App::new("version")
        .about("Operates on versions of projects.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(list_command)
        .subcommand(create_command);
}

pub fn process_version_command(args: &ArgMatches) -> i32 {
    return match args.subcommand() {
        ("list",  Some(sub_m)) => { list(sub_m) },
        ("create", Some(m)) => { 0 },
        _ => { 
            error!("No command avaliable. {:?}", args); 
            -1
        }
    };
}

fn list(args: &ArgMatches) -> i32 {
    let repo = match DefaultRepo::new(env::current_dir().unwrap().as_path()) {
        Some(v) => v,
        None => return 1
    };

    let versions = repo.find_versions();

    let format = match args.value_of("output-format") {
        Some(value) => value,
        _ => "plain"
    };

    match format {
        "json" => {
            let result = object! {
                "results" => versions
            };
            println!("{}", result.dump());
        },
        "plain" | _ => {
            versions.into_iter().for_each(|x| { 
                if x.message.is_some() {
                    let mut header = format!("{}", x.message.unwrap());
                    header.truncate(25);
                    println!("{} - {} - {}", x.name, x.id, header.trim());
                } else {
                    println!("{} - {}", x.name, x.id);
                }
            });
        }
    }
    return 0;
}