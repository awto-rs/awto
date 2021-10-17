use std::io::Write;

use anyhow::Result;
use async_trait::async_trait;
use clap::{AppSettings, Clap};
use colored::Colorize;
use compile::Compile;
use log::{error, Level, LevelFilter};

mod compile;
mod macros;
mod util;

/// Awto cli
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Compile(Compile),
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let mut cmd = match opts.subcmd {
        SubCommand::Compile(compile) => match compile.subcmd {
            Some(compile::SubCommand::Database(database)) => runnable_cmd!(database),
            Some(compile::SubCommand::Protobuf(protobuf)) => runnable_cmd!(protobuf),
            None => runnable_cmd!(compile),
        },
    };

    let log_level = if cmd.is_verbose() {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format(|buf, record| {
            let prefix = match record.level() {
                Level::Error => "error".red(),
                Level::Warn => "warn".yellow(),
                Level::Info => "info".blue(),
                Level::Debug => "debug".purple(),
                Level::Trace => "trace".cyan(),
            }
            .bold();
            writeln!(buf, "{} {}", prefix, record.args())
        })
        .init();

    if let Err(err) = cmd.run().await {
        error!("{}", err);
        if cmd.is_verbose() {
            let err_chain = err.chain().skip(1);
            if err_chain.clone().next().is_some() {
                eprintln!("{}", "\nCaused by:".italic().truecolor(190, 190, 190));
            }
            err_chain
                .for_each(|cause| eprintln!(" - {}", cause.to_string().truecolor(190, 190, 190)));
        }
        #[cfg(not(debug_assertions))]
        eprintln!(
            "\nIf the problem persists, please submit an issue on the Github repository.\n{}",
            "https://github.com/Acidic9/awto/issues/new".underline()
        );
        std::process::exit(1);
    }
}

#[async_trait]
pub trait Runnable {
    async fn run(&mut self) -> Result<()>;

    fn is_verbose(&self) -> bool {
        false
    }
}
