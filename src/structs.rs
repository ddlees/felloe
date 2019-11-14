use log::Level;
use serde_derive::Deserialize;
use structopt::clap::AppSettings::ColoredHelp;
use structopt::StructOpt;

// Add cool slogan for your app here, e.g.:
/// ⎈ The helm version manager
#[structopt(setting(ColoredHelp))]
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Cli {
    #[structopt(long)]
    pub log_level: Option<Level>,

    #[structopt(subcommand)]
    pub cmd: Option<Command>,

    pub version: Option<String>,

    #[structopt(short, long)]
    /// Show latest official helm version
    pub latest: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "latest")]
    /// Install the latest official helm release
    Latest,

    #[structopt(name = "run")]
    /// Execute downloaded helm <version> with [args ...]
    Run { version: String, args: Vec<String> },

    #[structopt(name = "which")]
    /// Output path for downloaded helm <version>
    Which { version: Option<String> },

    #[structopt(name = "exec")]
    /// Execute command with modified PATH, so downloaded helm <version> first
    Exec { version: String, args: Vec<String> },

    #[structopt(name = "remove")]
    /// Remove the given installed version(s)
    Remove {
        versions: Vec<String>,

        #[structopt(short = "f", long = "force")]
        force: bool,
    },

    #[structopt(name = "prune")]
    /// Remove all downloaded versions except the currently installed version
    Prune,

    #[structopt(name = "list")]
    /// Output downloaded versions
    List,

    #[structopt(name = "versions")]
    /// Output matching versions available for download
    Versions {
        #[structopt(long = "filter")]
        /// filter versions by contained string
        filter: Option<String>,

        #[structopt(long = "prerelease")]
        /// Include prerelease versions
        prerelease: bool,

        #[structopt(long = "last")]
        /// Fetch the last n number of releases on GitHub (default: 25)
        last: Option<usize>,
    },

    #[structopt(name = "uninstall")]
    /// Remove the installed helm
    Uninstall,
}

#[derive(Debug, Deserialize)]
pub struct Releases(pub Vec<Release>);

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
}
