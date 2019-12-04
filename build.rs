include!("src/cli.rs");

use std::fs;
use std::path::PathBuf;

fn main() {
    let name = std::env::var("CARGO_PKG_NAME").unwrap();
    let _target = std::env::var("TARGET").unwrap();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = std::env::var("PROFILE").unwrap();

    let root = PathBuf::from(manifest);
    let license = root.join("LICENSE");
    let readme = root.join("README.md");

    let outdir = root.join("target").join(profile);

    let completions = outdir.join("completions");

    if !completions.exists() {
        fs::create_dir_all(&completions).unwrap();
    }

    Cli::clap().gen_completions(&name, Shell::Bash, &completions);
    Cli::clap().gen_completions(&name, Shell::Zsh, &completions);
    Cli::clap().gen_completions(&name, Shell::Fish, &completions);
    Cli::clap().gen_completions(&name, Shell::Elvish, &completions);
    Cli::clap().gen_completions(&name, Shell::PowerShell, &completions);

    fs::copy(license, outdir.join("LICENSE")).unwrap();
    fs::copy(readme, outdir.join("README.md")).unwrap();
}
