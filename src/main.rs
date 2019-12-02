use env_logger::Builder;
use exitfailure::ExitFailure;
use felloe::{commands as cmd, Cli, Command};
use log::Level;
use regex::Regex;
use structopt::StructOpt;

fn main() -> Result<(), ExitFailure> {
    let args = Cli::from_args();

    let log_level = args.log_level.unwrap_or(Level::Warn);
    Builder::new()
        .filter(Some("felloe"), log_level.to_level_filter())
        .try_init()?;

    if let Some(cmd) = args.cmd {
        match cmd {
            Command::Completions { shell } => {
                let mut bytes = Vec::<u8>::new();
                Cli::clap().gen_completions_to("felloe", shell, &mut bytes);
                let content = String::from_utf8(bytes.to_vec())?;
                let output = Regex::new(":_files")?.replace_all(&content, "");
                println!("{}", output);
                Ok(())
            }
            Command::Exec { version, args } => cmd::exec(version, args),
            Command::Latest => cmd::install_latest(),
            Command::List => cmd::list(),
            Command::Prune => cmd::prune(),
            Command::Remove { versions, force } => cmd::remove(versions, force),
            Command::Run { version, args } => cmd::run_helm(&version, args),
            Command::Uninstall => cmd::uninstall(),
            Command::Versions {
                filter,
                prerelease,
                last,
            } => cmd::versions(filter, prerelease, last),
            Command::Which { version } => cmd::which(version),
        }?
    } else if args.latest {
        let release = cmd::fetch_release("latest")?;
        println!("{}", release.tag_name);
    } else if let Some(version) = args.version {
        cmd::install(&version)?
    } else {
        cmd::select_version()?
    }

    Ok(())
}
