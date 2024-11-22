use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct WatchCommand {
    pub name: String,
    pub command: String,
}

#[derive(Deserialize)]
pub struct Layout {
    pub path: PathBuf,
    pub watch: Vec<WatchCommand>,
}

#[derive(Deserialize)]
pub struct Config {
    pub template: PathBuf,
    pub layouts: Vec<Layout>,
}
