use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
}

#[derive(Debug, Deserialize)]
pub struct Releases(pub Vec<Release>);
