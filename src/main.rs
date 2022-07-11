#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod config;
mod db;
mod meta;
mod ui;

use app::Sqlife;
use config::ConnectionConfig;
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
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    uri: Option<String>,
    #[clap(short, long, value_parser)]
    schema: Option<String>,
    #[clap(short, long, arg_enum, value_parser)]
    kind: Option<SGDBKindArgs>,
}

fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    let options = eframe::NativeOptions::default();

    let connection = ConnectionConfig::new(
        "default",
        args.kind.unwrap().into(),
        args.uri.unwrap(),
        args.schema.unwrap(),
    );

    eframe::run_native(
        "sqlfi",
        options,
        Box::new(|cc| Sqlife::new(cc, connection)),
    );
}
