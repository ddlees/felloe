use dirs;
use flate2::read::GzDecoder;
use futures::future::{self, Future};
use futures::stream::Stream;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::r#async::{Chunk, Client, Decoder};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{copy, Cursor},
    mem,
    path::PathBuf,
    process::{Command},
};
use tar::Archive;

use super::*;

pub fn fetch_releases(count: usize, include_pre: bool) -> Result<Releases, failure::Error> {
    let mut core = tokio_core::reactor::Core::new()?;

    let url = format!("{}?per_page={}", &GH_RELEASES_API, &count);
    let client = Client::new();

    core.run(
        client
            .get(&url)
            .send()
            .and_then(|mut res| res.json::<Releases>())
            .from_err()
            .map(|rels| {
                Releases(
                    rels.0
                        .into_iter()
                        .filter(|rel| !rel.prerelease || include_pre)
                        .collect(),
                )
            }),
    )
}

pub fn fetch_release(version: &str) -> Result<Release, failure::Error> {
    let mut core = tokio_core::reactor::Core::new()?;

    let url = if version == "latest" {
        format!("{}/{}", &GH_RELEASES_API, version)
    } else {
        format!("{}/tags/{}", &GH_RELEASES_API, version)
    };

    let client = Client::new();

    core.run(
        client
            .get(&url)
            .send()
            .and_then(|mut res| res.json::<Release>())
            .from_err(),
    )
}

pub fn download_release(version: &str) -> Result<(), failure::Error> {
    let mut core = tokio_core::reactor::Core::new()?;

    let name = format!("helm-{}-{}-{}", version, OS, ARCH);
    let file_name = format!("{}.tar.gz", name);
    let file_url = format!("{}/{}", HELM_DOWNLOAD_URL, file_name);
    let sha_url = format!("{}.sha256", &file_url);

    let mut file = Vec::<u8>::new();
    let mut sha = Vec::<u8>::new();

    let results = core.run(future::join_all(vec![
        download(file_url),
        download(sha_url),
    ]))?;

    let mut results = results.into_iter();
    copy(&mut results.next().unwrap(), &mut file)?;
    copy(&mut results.next().unwrap(), &mut sha)?;

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

fn hash(file: &Vec<u8>) -> Result<String, failure::Error> {
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

// TODO Refactor to move references around more appropriately
pub fn download(url: String) -> impl Future<Item = Cursor<Chunk>, Error = failure::Error> {
    let file_name = String::from(PathBuf::from(&url).file_name().unwrap().to_str().unwrap());

    debug!("Setting up download progress bar");
    let length = fetch_content_length(&url).unwrap();
    let pb = Box::new(ProgressBar::new(length));

    pb.set_style(
        ProgressStyle::default_bar()
            .template(BAR_STYLE_TEMPLATE)
            .progress_chars(BAR_PROGRESS_CHARS),
    );

    info!("Downloading {}", file_name);
    Client::new()
        .get(&url)
        .send()
        .and_then(move |mut res| {
            debug!("Binding progress bar to download stream");
            let pb_clone = Box::clone(&pb);

            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.inspect(move |chunk| {
                pb.inc(*&chunk.len() as u64);
                if pb.position() == *&chunk.len() as u64 {
                    pb.set_message(&file_name);
                }
            })
            .concat2()
            .inspect(move |_| {
                pb_clone.finish();
            })
        })
        .map(|bytes| Cursor::new(bytes))
        .from_err()
}

pub fn fetch_content_length(url: &str) -> Result<u64, failure::Error> {
    let mut core = tokio_core::reactor::Core::new()?;
    let client = Client::new();

    core.run(
        client
            .head(url)
            .send()
            .map(|res| res.content_length().unwrap())
            .from_err(),
    )
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

fn set_active(version: &str) -> Result<(), failure::Error> {
    info!("Installing helm and tiller into {}", BIN_DIR);
    let install_path = get_cache_path(version).join(format!("{}-{}", OS, ARCH));

    let helm_path = install_path.join(HELM_BIN_NAME);
    let tiller_path = install_path.join(TILLER_BIN_NAME);

    let helm_sym_path = PathBuf::from(BIN_DIR).join(HELM_BIN_NAME);
    let tiller_sym_path = PathBuf::from(BIN_DIR).join(TILLER_BIN_NAME);

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
        warn!(
            "Unable to set active tiller {}. The executable does not exist at {}",
            version,
            tiller_path.to_str().unwrap()
        );
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
            let helm_sym_path = PathBuf::from(BIN_DIR).join(HELM_BIN_NAME);
            let tiller_sym_path = PathBuf::from(BIN_DIR).join(TILLER_BIN_NAME);

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
    let command = Command::new(&helm_path).args(args).spawn().expect(&format!(
        "{} failed to start",
        helm_path.as_path().to_str().unwrap()
    ));

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
    let sym_path = PathBuf::from(BIN_DIR).join(HELM_BIN_NAME);

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
    let filter = filter.unwrap_or(String::new());
    let include_pre = include_pre || false;
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
