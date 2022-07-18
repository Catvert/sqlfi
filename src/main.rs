#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod config;
mod db;
mod meta;
mod ui;

use app::Sqlife;
use config::{ConnectionConfig, SqlifeConfig};
use db::sgdb::SGDBKind;

use clap::{ArgEnum, Parser};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum SGDBKindArgs {
    Mysql,
    Postgres,
    Sqlite,
}

impl Into<SGDBKind> for SGDBKindArgs {
    fn into(self) -> SGDBKind {
        match self {
            SGDBKindArgs::Mysql => SGDBKind::Mysql,
            SGDBKindArgs::Postgres => SGDBKind::Postgres,
            SGDBKindArgs::Sqlite => SGDBKind::Sqlite,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    connection_index: Option<usize>
}

fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    let options = eframe::NativeOptions::default();

    let config = SqlifeConfig::load().unwrap_or_default();

    eframe::run_native(
        "sqlife",
        options,
        Box::new(move |cc| Sqlife::new(cc, config, args.connection_index)),
    );
}
