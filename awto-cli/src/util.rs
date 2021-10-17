use std::{io::SeekFrom, path::Path};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use toml_edit::{Document, Value};

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

pub async fn add_package_to_workspace(pkg: &str) -> Result<()> {
    let mut cargo_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("./Cargo.toml")
        .await
        .context("could not open root Cargo.toml file")?;
    let mut buffer = String::new();
    cargo_file.read_to_string(&mut buffer).await?;
    let mut doc: Document = buffer
        .parse()
        .context("could not parse root Cargo.toml file")?;
    let members = doc
        .as_table_mut()
        .get_mut("workspace")
        .and_then(|workspace| workspace.as_table_like_mut())
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(|members| members.as_array_mut())
        .ok_or_else(|| anyhow!("workspace does not exist in root Cargo.toml file"))?;

    if !members.iter().any(|member| {
        member
            .as_str()
            .map(|member_str| member_str == pkg)
            .unwrap_or(false)
    }) {
        let index = members
            .iter()
            .enumerate()
            .find_map(|(i, member)| {
                if member
                    .as_str()
                    .map(|member_str| member_str > pkg)
                    .unwrap_or(false)
                {
                    Some(i)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| members.len());
        let first_prefix = members
            .get(0)
            .and_then(|member| member.decor().prefix())
            .map(|prefix| prefix.to_string());
        let value: Value = pkg.into();
        if first_prefix
            .as_ref()
            .map(|prefix| prefix.contains('\n'))
            .unwrap_or(false)
        {
            members.insert_formatted(index, value.decorated(&first_prefix.unwrap(), ""));
        } else if index == 0 {
            if let Some(first_prefix) = members
                .get(0)
                .map(|member| member.decor())
                .and_then(|decor| decor.prefix())
                .map(|prefix| prefix.to_string())
            {
                members.insert_formatted(index, value.decorated(&first_prefix, ""));
            } else {
                members.insert_formatted(index, value.decorated("", ""));
            }

            if let Some(member) = members.get_mut(index + 1) {
                member.decor_mut().set_prefix(" ")
            }
        } else {
            members.insert(index, value);
        }

        cargo_file.set_len(0).await?;
        cargo_file.seek(SeekFrom::Start(0)).await?;
        cargo_file.write_all(doc.to_string().as_bytes()).await?;
    }

    Ok(())
}
