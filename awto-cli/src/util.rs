use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize, Clone, Debug)]
pub struct CargoFile {
    pub package: Option<CargoPackage>,
    pub workspace: Option<CargoWorkspace>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CargoPackage {
    pub name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CargoWorkspace {
    pub members: Vec<String>,
}

impl CargoFile {
    pub async fn load(path: impl AsRef<Path>) -> Result<CargoFile> {
        let bytes = fs::read(path).await.context("file not found")?;
        Ok(toml::from_slice(&bytes).context("Cargo.toml file corrupt")?)
    }
}
