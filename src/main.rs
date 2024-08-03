use log::{error, info};
use serde::Serialize;
use std::path::{Path, PathBuf};
mod errors;
use errors::Result;
mod chart_hashes;
mod fsutil;
mod ops;
mod table_loader;
mod utils;
use clap::{Parser, Subcommand};
use std::env;
use std::io::Write;
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    dryrun: bool,

    #[clap(long)]
    jsonlog: bool,

    #[clap(short, long, help = "your bms directory")]
    mydir: PathBuf,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "check how many charts you have in the specified table")]
    Check {
        #[clap(
            short,
            long,
            help = "table or score.json url. e.g.) https://stellabms.xyz/sl/table.html"
        )]
        table_url: String,

        #[clap(long, help = "check level limit")]
        level_limit: Option<u8>,

        #[clap(long, help = "check level lower limit")]
        level_lower_limit: Option<u8>,
    },

    #[clap(about = "install from zip files into mydir")]
    Install {
        #[clap(short, long, help = "install zips from this directory")]
        from: PathBuf,

        #[clap(short, long, help = "if install recursively from nested directory")]
        recursive: bool,
    },

    #[clap(about = "organize (merge & reconstruct) your directory")]
    Organize {
        #[clap(
            short,
            long,
            help = "destination directory. if omitted, use the same dir as mydir"
        )]
        dest: Option<PathBuf>,

        #[clap(short, long, help = "if true, destination folder is divided into hash")]
        shard: bool,
    },

    #[clap(about = "rename your directories")]
    Rename {},

    #[clap(about = "create your today's task")]
    Task {
        #[clap(long)]
        table_url: String,
        #[clap(long)]
        player_score_path: PathBuf,
        #[clap(long)]
        songdata_path: PathBuf,
        #[clap(long)]
        folder_default_json: PathBuf,
        #[clap(long)]
        lower_limit_level: u8,
        #[clap(long, help = "AEASY / EASY / NORMAL / HARD / EXHARD")]
        target_lamp: String,
        #[clap(long)]
        task_notes: u32,
    },

    #[clap(about = "create an oldest played charts list")]
    Oldest {
        #[clap(long)]
        player_score_path: PathBuf,
        #[clap(long)]
        table_json_path: PathBuf,
        #[clap(long, help = "EASY / NORMAL / HARD / EXHARD")]
        target_lamp: String,
        #[clap(short, long, help = "if true, reset table json for this command")]
        reset: bool,
    },
}

#[derive(strum_macros::Display, Serialize, Debug)]
pub enum FrontendMsg {
    CheckNotFound,
    CheckSummary,
}

#[derive(Serialize, Debug)]
struct LogLine {
    pub ts: String,
    pub logtype: String,
    pub level: String,
    pub msg: String,
}

fn main() -> Result<()> {
    let start = Instant::now();

    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or(String::from("info")),
    );

    let cli = Cli::parse();
    if cli.jsonlog {
        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                let ts = buf.timestamp();
                let logtype = if record.target().starts_with("beatman") {
                    "LogMessage".to_owned()
                } else {
                    format!("frontend_{}", record.target().to_owned())
                };

                let ll = LogLine {
                    ts: ts.to_string(),
                    logtype,
                    level: record.level().to_string(),
                    msg: record.args().to_string(),
                };
                writeln!(buf, "{}", serde_json::to_string(&ll).unwrap())
            })
            .init();
    } else {
        env_logger::init();
    }

    let mydir = &Path::new(&cli.mydir);
    if !mydir.is_dir() {
        error!("mydir is not a directory");
        return Err("mydir is not a directory".into());
    }

    let dryrun = cli.dryrun;

    match &cli.command {
        Commands::Check {
            table_url,
            level_limit,
            level_lower_limit,
        } => {
            ops::check_table_coverage::check_table_coverage(
                table_url,
                mydir,
                level_limit,
                level_lower_limit,
            )?;
        }
        Commands::Install { from, recursive } => {
            let from = &Path::new(&from);
            if !from.is_dir() {
                error!("from is not a directory");
                return Err("from is not a directory".into());
            }
            if *recursive {
                ops::install_from_dir::install_from_dirs(from, mydir, dryrun)?;
            } else {
                ops::install_from_dir::install_from_dir(from, mydir, dryrun)?;
            }
        }
        Commands::Organize { dest, shard } => {
            let dest = dest.as_ref().map(Path::new).unwrap_or(mydir);
            if !dest.is_dir() {
                error!("dest is not a directory");
                return Err("dest is not a directory".into());
            }

            info!("== rename ==");
            ops::rename::rename_dirs(mydir, dryrun)?;
            info!("== merge ==");
            ops::merge::merge(mydir, dryrun)?;
            info!("== reconstruct ==");
            ops::reconstruct::reconstruct(mydir, dest, dryrun, *shard)?;
        }
        Commands::Rename {} => {
            ops::rename::rename_dirs(mydir, dryrun)?;
        }
        Commands::Task {
            table_url,
            player_score_path,
            songdata_path,
            folder_default_json,
            lower_limit_level,
            target_lamp,
            task_notes,
        } => {
            ops::create_task_folder::create_task_folder(
                table_url,
                player_score_path,
                songdata_path,
                folder_default_json,
                *lower_limit_level,
                target_lamp,
                *task_notes,
            )?;
        }
        Commands::Oldest {
            player_score_path,
            table_json_path,
            target_lamp,
            reset,
        } => {
            ops::create_oldest_played_folder::create_oldest_played_folder(
                player_score_path,
                table_json_path,
                target_lamp,
                *reset,
            )?;
        }
    }

    info!("Time elapsed: {:?}", start.elapsed());

    Ok(())
}
