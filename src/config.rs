use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Watch {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub broadcast: bool,
}

#[derive(Deserialize)]
pub struct Layout {
    pub path: PathBuf,
    pub watch: Vec<Watch>,
}

#[derive(Deserialize)]
pub struct Config {
    pub template: PathBuf,
    pub layout: Vec<Layout>,
}
