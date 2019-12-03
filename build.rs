include!("src/cli.rs");

use std::path::PathBuf;

fn main() {
    let name = std::env::var("CARGO_PKG_NAME").unwrap();
    let _target = std::env::var("TARGET").unwrap();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = std::env::var("PROFILE").unwrap();

    let outdir = PathBuf::from(manifest).join("target").join(profile);

    Cli::clap().gen_completions(&name, Shell::Bash, &outdir);
    Cli::clap().gen_completions(&name, Shell::Zsh, &outdir);
    Cli::clap().gen_completions(&name, Shell::Fish, &outdir);
    Cli::clap().gen_completions(&name, Shell::Elvish, &outdir);
    Cli::clap().gen_completions(&name, Shell::PowerShell, &outdir);
}
