#![feature(lazy_cell)]

use clap::Parser;
use command::Args;
use eyre::Result;

mod command;
mod config;
mod forecast;
mod provider;
mod provider_loader;

fn main() -> Result<()> {
    let cli = Args::parse();
    cli.command.process()?;

    Ok(())
}
