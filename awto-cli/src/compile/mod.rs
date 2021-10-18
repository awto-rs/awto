use std::path::Path;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::{Parser, IntoApp};
use tokio::fs;

use crate::Runnable;

use self::database::Database;
use self::protobuf::Protobuf;

mod database;
mod protobuf;

/// Compiles app to generate packages
#[derive(Parser)]
pub struct Compile {
    /// Compiles all packages
    #[clap(long)]
    pub all: bool,
    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
    /// Prints more information
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Parser)]
pub enum SubCommand {
    Database(Database),
    Protobuf(Protobuf),
}

#[async_trait]
impl Runnable for Compile {
    async fn run(&mut self) -> Result<()> {
        if !self.all {
            return Ok(Compile::into_app().print_help()?);
        }

        let mut database = Database {
            verbose: self.verbose,
        };
        database.run().await?;

        let mut protobuf = Protobuf {
            verbose: self.verbose,
        };
        protobuf.run().await?;

        Ok(())
    }
}

async fn prepare_awto_dir() -> Result<()> {
    let awto_path = Path::new("./awto");
    if !awto_path.is_dir() {
        fs::create_dir(awto_path)
            .await
            .context("could not create directory './awto'")?;
    }

    fs::write("./awto/README.md", include_bytes!("../templates/README.md"))
        .await
        .context("could not write file './awto/README.md'")?;

    Ok(())
}

async fn build_awto_pkg(name: &str) -> Result<()> {
    let status = tokio::process::Command::new("cargo")
        .current_dir("./awto")
        .arg("build")
        .arg("-p")
        .arg(name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow!("cargo build failed"));
    }

    Ok(())
}
