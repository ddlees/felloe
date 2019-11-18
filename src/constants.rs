pub static GH_RELEASES_API: &str = "https://api.github.com/repos/helm/helm/releases";
pub static HELM_DOWNLOAD_URL: &str = "https://get.helm.sh";
pub static INSTALLATION_DIR: &str = ".felloe";

#[cfg(target_os = "macos")]
pub static OS: &str = "darwin";

#[cfg(target_os = "linux")]
pub static OS: &str = "linux";

#[cfg(target_os = "windows")]
pub static OS: &str = "windows";

#[cfg(any(target_arch = "x86", target_arch = "i686", target_arch = "i386"))]
pub static ARCH: &str = "386";

#[cfg(target_arch = "x86_64")]
pub static ARCH: &str = "amd64";

#[cfg(target_arch = "powerpc64le")]
pub static ARCH: &str = "ppc64le";

#[cfg(target_arch = "s390x")]
pub static ARCH: &str = "s390x";

#[cfg(any(target_arch = "arm", target_arch = "armv7"))]
pub static ARCH: &str = "arm";

#[cfg(target_arch = "aarch64")]
pub static ARCH: &str = "arm64";

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub static HELM_BIN_NAME: &str = "helm";

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub static TILLER_BIN_NAME: &str = "tiller";

#[cfg(target_os = "windows")]
pub static HELM_BIN_NAME: &str = "helm.exe";

#[cfg(target_os = "windows")]
pub static TILLER_BIN_NAME: &str = "tiller.exe";

pub static BAR_STYLE_TEMPLATE: &str =
    "{spinner:.green} {msg} {percent}% {bar:40.cyan/blue} {bytes}/{total_bytes} eta: {eta}";
pub static BAR_PROGRESS_CHARS: &str = "#>-";

pub static UNPACK_SPINNER_CHARS: &[&str] =
    &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▊", "▋", "▌", "▍", "▎"];

pub static UNPACK_SPINNER_TEMPLATE: &str = "{spinner:.blue} {msg}";

pub static VERIFY_SPINNER_CHARS: &[&str] = &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"];

pub static VERIFY_SPINNER_TEMPLATE: &str = "{spinner:.green} {msg}";
