use std::fmt::Write;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use log::info;
use tokio::fs;

use crate::{
    util::{add_package_to_workspace, CargoFile},
    Runnable,
};

use super::{build_awto_pkg, prepare_awto_dir};

/// Compiles protobuf package from app service
#[derive(Parser)]
pub struct Protobuf {
    /// Prints more information
    #[clap(short, long)]
    pub verbose: bool,
}

#[async_trait]
impl Runnable for Protobuf {
    async fn run(&mut self) -> Result<()> {
        let cargo_file = CargoFile::load("./service/Cargo.toml")
            .await
            .context("could not load service Cargo.toml file from './service/Cargo.toml'")?;
        if cargo_file
            .package
            .as_ref()
            .map(|package| package.name != "service")
            .unwrap_or(false)
        {
            match cargo_file.package {
                Some(package) => {
                    return Err(anyhow!(
                        "service package must be named 'service' but is named '{}'",
                        package.name
                    ));
                }
                None => return Err(anyhow!("service package must be named 'service'")),
            }
        }

        prepare_awto_dir().await?;

        Self::prepare_protobuf_dir().await?;
        add_package_to_workspace("awto/protobuf").await?;
        build_awto_pkg("protobuf").await?;

        info!("compiled package 'protobuf'");

        Ok(())
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

impl Protobuf {
    const PROTOBUF_DIR: &'static str = "./awto/protobuf";
    const PROTOBUF_SRC_DIR: &'static str = "./awto/protobuf/src";
    const PROTOBUF_CARGO_PATH: &'static str = "./awto/protobuf/Cargo.toml";
    const PROTOBUF_CARGO_TOML_BYTES: &'static [u8] = include_bytes!("../templates/protobuf/Cargo.toml.template");
    const PROTOBUF_BUILD_PATH: &'static str = "./awto/protobuf/build.rs";
    const PROTOBUF_BUILD_BYTES: &'static [u8] = include_bytes!("../templates/protobuf/build.rs.template");
    const PROTOBUF_LIB_PATH: &'static str = "./awto/protobuf/src/lib.rs";

    async fn prepare_protobuf_dir() -> Result<()> {
        if Path::new(Self::PROTOBUF_DIR).is_dir() {
            fs::remove_dir_all(Self::PROTOBUF_DIR)
                .await
                .with_context(|| format!("could not delete directory '{}'", Self::PROTOBUF_DIR))?;
        }

        fs::create_dir(Self::PROTOBUF_DIR)
            .await
            .with_context(|| format!("could not create directory '{}'", Self::PROTOBUF_DIR))?;

        fs::create_dir(Self::PROTOBUF_SRC_DIR)
            .await
            .with_context(|| format!("could not create directory '{}'", Self::PROTOBUF_SRC_DIR))?;

        fs::write(Self::PROTOBUF_CARGO_PATH, Self::PROTOBUF_CARGO_TOML_BYTES)
            .await
            .with_context(|| format!("could not write file '{}'", Self::PROTOBUF_CARGO_PATH))?;

        fs::write(Self::PROTOBUF_BUILD_PATH, Self::PROTOBUF_BUILD_BYTES)
            .await
            .with_context(|| format!("could not write file '{}'", Self::PROTOBUF_BUILD_PATH))?;

        let mut lib_content = concat!(
            "// This file is automatically @generated by ",
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            "\n\n"
        )
        .to_string();

        writeln!(
            lib_content,
            r#"include!(concat!(env!("OUT_DIR"), "/app.rs"));"#
        )
        .unwrap();

        fs::write(Self::PROTOBUF_LIB_PATH, lib_content)
            .await
            .with_context(|| format!("could not write file '{}'", Self::PROTOBUF_LIB_PATH))?;

        Ok(())
    }
}
