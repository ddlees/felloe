use crate::constants::*;
use crate::progress::DownloadProgress;
use crate::release::{Release, Releases};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    input::{input, InputEvent, KeyEvent},
    queue,
    screen::{EnterAlternateScreen, LeaveAlternateScreen, RawScreen},
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
    Output,
};
use dirs;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{copy, stdout, Write},
    path::PathBuf,
    process::Command,
    sync::Arc,
};
use tar::Archive;

pub fn fetch_releases(count: usize, include_pre: bool) -> Result<Releases, failure::Error> {
    let url = format!("{}?per_page={}", &GH_RELEASES_API, &count);
    let client = Client::new();

    let releases: Releases = client.get(&url).send()?.json()?;
    let mut releases = Releases(
        releases
            .0
            .into_iter()
            .filter(|rel| !rel.prerelease || include_pre)
            .collect(),
    );

    releases.0.sort_by(|a, b| b.tag_name.cmp(&a.tag_name));
    releases.0.reverse();

    Ok(releases)
}

pub fn fetch_release(version: &str) -> Result<Release, failure::Error> {
    let url = if version == "latest" {
        format!("{}/{}", &GH_RELEASES_API, version)
    } else {
        format!("{}/tags/{}", &GH_RELEASES_API, version)
    };

    Ok(Client::new().get(&url).send()?.json()?)
}

pub fn download_release(version: &str) -> Result<(), failure::Error> {
    let name = format!("helm-{}-{}-{}", version, OS, ARCH);
    let file_name = format!("{}.tar.gz", name);
    let file_url = format!("{}/{}", HELM_DOWNLOAD_URL, file_name);
    let sha_url = format!("{}.sha256", &file_url);

    let file = download(file_url)?;
    let sha = download(sha_url)?;

    let verify_spinner = ProgressBar::new_spinner();
    verify_spinner.enable_steady_tick(150);
    verify_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(VERIFY_SPINNER_CHARS)
            .template(VERIFY_SPINNER_TEMPLATE),
    );
    verify_spinner.set_message(&format!("Verifying {}", file_name));

    let hash = hash(&file)?;
    let sha = String::from_utf8(sha)?;
    sha256sum(&hash, &sha)?;
    verify_spinner.finish_with_message(&format!("helm {} verified", version));

    let decoder = GzDecoder::new(&file[..]);
    let mut archive = Archive::new(decoder);

    let unpack_spinner = ProgressBar::new_spinner();
    unpack_spinner.enable_steady_tick(120);
    unpack_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(UNPACK_SPINNER_CHARS)
            .template(UNPACK_SPINNER_TEMPLATE),
    );
    unpack_spinner.set_message(&format!("Unpacking {}", file_name));

    let cache_dir = get_cache_path(version);

    info!("Extracting {} to {:?}", file_name, cache_dir);
    archive.unpack(cache_dir)?;

    unpack_spinner.finish_with_message(&format!("helm {} installed", version));
    Ok(())
}

fn hash(file: &[u8]) -> Result<String, failure::Error> {
    let mut hasher = Sha256::new();
    hasher.input(file);
    let hash = format!("{:x}", hasher.result());

    Ok(hash)
}

fn sha256sum(hash: &str, sum: &str) -> Result<(), failure::Error> {
    if hash == sum.trim() {
        Ok(())
    } else {
        Err(failure::err_msg(
            "The integrity of this file is unable to be verified.",
        ))
    }
}

pub fn download(url: String) -> Result<Vec<u8>, failure::Error> {
    let file_name = String::from(PathBuf::from(&url).file_name().unwrap().to_str().unwrap());

    debug!("Setting up download progress bar");
    let length = fetch_content_length(&url).unwrap();
    let pb = Arc::new(ProgressBar::new(length));

    pb.set_style(
        ProgressStyle::default_bar()
            .template(BAR_STYLE_TEMPLATE)
            .progress_chars(BAR_PROGRESS_CHARS),
    );

    pb.set_message(&file_name);

    info!("Downloading {}", file_name);
    let mut stream = DownloadProgress {
        pb: pb.clone(),
        stream: Client::new().get(&url).send()?,
    };

    let mut bytes = Vec::<u8>::new();

    copy(&mut stream, &mut bytes)?;

    pb.finish();

    Ok(bytes)
}

pub fn fetch_content_length(url: &str) -> Result<u64, failure::Error> {
    Ok(Client::new().head(url).send()?.content_length().unwrap())
}

pub fn install_latest() -> Result<(), failure::Error> {
    info!("Installing latest");
    let release = fetch_release("latest")?;
    install(&release.tag_name)
}

pub fn install(version: &str) -> Result<(), failure::Error> {
    info!("Installing {}", version);
    let release = fetch_release(version)?;

    if !is_helm_installed(version) {
        info!("Downloading helm {}", version);
        download_release(&release.tag_name)?;
    }

    info!("Setting {} as active version", release.tag_name);
    set_active(version)?;

    println!("Activated helm {}", version);
    Ok(())
}

fn get_cache_path(version: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(&INSTALLATION_DIR)
        .join("cache")
        .join(version)
}

fn is_helm_installed(version: &str) -> bool {
    let cache_dir = get_cache_path(version);

    info!("Checking {} for helm binary", cache_dir.to_str().unwrap());
    if cache_dir.exists() {
        debug!("Cache directory {} exists", cache_dir.to_str().unwrap());
        return true;
    }

    debug!(
        "Cache directory {} does not exist",
        cache_dir.to_str().unwrap()
    );
    false
}

fn get_bin_path() -> Result<PathBuf, failure::Error> {
    if cfg!(target_os = "windows") {
        #[cfg(target_arch = "x86_64")]
        let path = std::env::var("programfiles(x86)")?;

        #[cfg(any(target_arch = "x86", target_arch = "i686", target_arch = "i386"))]
        let path = std::env::var("programfiles")?;

        Ok(PathBuf::from(path).join("helm"))
    } else {
        Ok(PathBuf::from("/usr/local/bin"))
    }
}

fn set_active(version: &str) -> Result<(), failure::Error> {
    let bin = get_bin_path()?;

    info!("Installing helm and tiller into {}", bin.to_str().unwrap());
    let install_path = get_cache_path(version).join(format!("{}-{}", OS, ARCH));

    let helm_path = install_path.join(HELM_BIN_NAME);
    let tiller_path = install_path.join(TILLER_BIN_NAME);

    let helm_sym_path = PathBuf::from(&bin).join(HELM_BIN_NAME);
    let tiller_sym_path = PathBuf::from(&bin).join(TILLER_BIN_NAME);

    if helm_path.exists() {
        if helm_sym_path.exists() {
            fs::remove_file(&helm_sym_path)?;
        }

        // TODO fs::soft_link is deprecated but the equivalent function for windows is borked at this time
        #[allow(deprecated)]
        fs::soft_link(helm_path, helm_sym_path)?;
    } else {
        return Err(failure::err_msg(format!(
            "Unable to set active helm {}. The executable does not exist at {}",
            version,
            helm_path.to_str().unwrap()
        )));
    }

    if tiller_path.exists() {
        if tiller_sym_path.exists() {
            fs::remove_file(&tiller_sym_path)?;
        }
        fs::hard_link(tiller_path, tiller_sym_path)?;
    } else {
        if version.contains("v2") {
            warn!(
                "Unable to set active tiller {}. The executable does not exist at {}",
                version,
                tiller_path.to_str().unwrap()
            );
        }
    }

    Ok(())
}

pub fn get_installed_versions() -> Result<Vec<String>, failure::Error> {
    let versions = dirs::home_dir()
        .unwrap()
        .join(&INSTALLATION_DIR)
        .join("cache")
        .read_dir()?
        .map(|e| String::from(e.unwrap().file_name().to_str().unwrap()))
        .collect();

    Ok(versions)
}

pub fn list() -> Result<(), failure::Error> {
    let versions = get_installed_versions()?;

    for version in versions {
        println!("{}", version);
    }

    Ok(())
}

pub fn exec(version: String, args: Vec<String>) -> Result<(), failure::Error> {
    let bin_path = get_cache_path(&version).join(format!("{}-{}", OS, ARCH));

    let path_envar = env::var_os("PATH").unwrap();
    let mut paths = env::split_paths(&path_envar).collect::<Vec<_>>();
    paths.insert(0, bin_path);
    let new_path = env::join_paths(paths).unwrap();

    Command::new("helm")
        .env("PATH", new_path)
        .args(args)
        .spawn()?
        .wait_with_output()?;

    Ok(())
}

pub fn prune() -> Result<(), failure::Error> {
    let active_version = get_active_version()?;
    let installed_versions = get_installed_versions()?;

    let versions: Vec<String> = installed_versions
        .into_iter()
        .filter(|v| v != &active_version)
        .collect();

    remove(versions, false)?;

    Ok(())
}

pub fn remove(versions: Vec<String>, force: bool) -> Result<(), failure::Error> {
    let active_version = get_active_version()?;
    if versions.contains(&active_version) {
        if force {
            let helm_sym_path = get_bin_path()?.join(HELM_BIN_NAME);
            let tiller_sym_path = get_bin_path()?.join(TILLER_BIN_NAME);

            if helm_sym_path.exists() {
                fs::remove_file(helm_sym_path)?;
            }

            if tiller_sym_path.exists() {
                fs::remove_file(tiller_sym_path)?;
            }
        } else {
            return Err(failure::err_msg(format!(
                "Cannot remove active version: {}",
                active_version
            )));
        }
    }

    for version in versions.iter() {
        let version_path = get_cache_path(version);

        if version_path.exists() {
            info!("Uninstalling {}", version);
            fs::remove_dir_all(version_path)?;
        } else {
            println!("Cache directory {} does not exist.", version);
        }
    }

    Ok(())
}

pub fn run_helm(version: &str, args: Vec<String>) -> Result<(), failure::Error> {
    let helm_path = get_cache_path(&version)
        .join(format!("{}-{}", OS, ARCH))
        .join(HELM_BIN_NAME);
    let command = Command::new(&helm_path)
        .args(args)
        .spawn()
        .unwrap_or_else(|_| panic!("{} failed to start", helm_path.as_path().to_str().unwrap()));

    command.wait_with_output()?;

    Ok(())
}

pub fn uninstall() -> Result<(), failure::Error> {
    let version = get_active_version()?;

    println!("Uninstalling helm {}", version);
    remove([version].to_vec(), true)?;

    println!("No active version set");

    Ok(())
}

pub fn get_active_version() -> Result<String, failure::Error> {
    let sym_path = get_bin_path()?.join(HELM_BIN_NAME);

    if !sym_path.exists() {
        return Err(failure::err_msg("An active version is not set"));
    }

    let version = match fs::read_link(sym_path) {
        Ok(path) => path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_os_string(),
        Err(_) => {
            return Err(failure::err_msg(
                "Unable to detect active version. {} is not a symbolic link.",
            ));
        }
    };

    Ok(String::from(version.as_os_str().to_str().unwrap()))
}

pub fn versions(
    filter: Option<String>,
    include_pre: bool,
    last: Option<usize>,
) -> Result<(), failure::Error> {
    let filter = filter.unwrap_or_default();
    let last = last.unwrap_or(25);

    let releases = fetch_releases(last, include_pre)?;

    releases
        .0
        .into_iter()
        .filter(|release| release.tag_name.contains(filter.as_str()))
        .for_each(|release| println!("{}", release.tag_name));

    Ok(())
}

pub fn which(version: Option<String>) -> Result<(), failure::Error> {
    let version = version.unwrap_or(get_active_version()?);

    let install_path = get_cache_path(&version).join(format!("{}-{}", OS, ARCH));
    let helm_path = install_path.join(HELM_BIN_NAME);
    let tiller_path = install_path.join(TILLER_BIN_NAME);

    if helm_path.exists() {
        println!("{}", helm_path.as_path().to_str().unwrap());
    } else {
        return Err(failure::err_msg(format!(
            "A helm binary is not installed for {}",
            version
        )));
    }

    if tiller_path.exists() {
        println!("{}", tiller_path.as_path().to_str().unwrap());
    }

    Ok(())
}

pub fn select_version() -> Result<(), failure::Error> {
    let mut versions = get_installed_versions()?;
    versions.sort();

    let active_version = get_active_version()?;
    let mut active_index = versions.iter().position(|v| v == &active_version).unwrap();

    let mut stdout = stdout();
    enter_alt_screen()?;
    update_screen(&mut stdout, &versions, active_index)?;

    let _raw = RawScreen::into_raw_mode()?;
    let mut sync_stdin = input().read_sync();

    loop {
        let event = sync_stdin.next();

        if let Some(key_event) = event {
            if let InputEvent::Keyboard(k) = key_event {
                match k {
                    KeyEvent::Ctrl('c') | KeyEvent::Char('q') => {
                        leave_alt_screen()?;

                        break;
                    }
                    KeyEvent::Delete | KeyEvent::Backspace | KeyEvent::Char('d') => {
                        let version = String::from(versions.to_vec().get(active_index).unwrap());
                        leave_alt_screen()?;

                        remove([version.clone()].to_vec(), false)?;
                        queue!(stdout, Output(format!("Uninstalled {}\r\n", version)))?;
                        break;
                    }
                    KeyEvent::Enter | KeyEvent::Char('i') => {
                        let version = String::from(versions.to_vec().get(active_index).unwrap());
                        leave_alt_screen()?;

                        set_active(&version)?;
                        queue!(stdout, Output(format!("\rActivated {}\r\n", version)))?;
                        break;
                    }
                    KeyEvent::Up | KeyEvent::Char('w') | KeyEvent::Char('k') => {
                        if active_index > 0 {
                            active_index -= 1;
                        }

                        update_screen(&mut stdout, &versions, active_index)?;
                    }
                    KeyEvent::Down | KeyEvent::Char('s') | KeyEvent::Char('j') => {
                        if active_index < versions.len() - 1 {
                            active_index += 1;
                        }

                        update_screen(&mut stdout, &versions, active_index)?;
                    }
                    _ => { /* Default case: do nothing*/ }
                }
            }
        }
    }

    Ok(())
}

fn update_screen<W: Write>(
    w: &mut W,
    versions: &[String],
    active: usize,
) -> Result<(), failure::Error> {
    queue!(w, MoveTo(0, 5), Clear(ClearType::FromCursorDown))?;
    for (i, v) in versions.iter().enumerate() {
        if i == active {
            queue!(
                w,
                SetForegroundColor(Color::Blue),
                Output("  ‣ "),
                Output(v),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(w, Output("    "), Output(v))?;
        }

        queue!(w, Output("\r\n"))?;
    }
    w.flush()?;
    Ok(())
}

fn enter_alt_screen() -> Result<(), failure::Error> {
    let mut stdout = stdout();
    queue!(stdout, EnterAlternateScreen)?;
    queue!(stdout, Hide)?;
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, MoveTo(0, 0))?;
    queue!(
        stdout,
        Output(
            "Move ↑↓ to select an installed version\nReturn key (i) to activate\nDelete key (d) to uninstall\nq to quit".to_string(),
        )
    )?;

    stdout.flush()?;

    Ok(())
}

fn leave_alt_screen() -> Result<(), failure::Error> {
    let mut stdout = stdout();

    queue!(stdout, Show)?;
    queue!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;

    Ok(())
}
