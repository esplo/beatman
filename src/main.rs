use log::{error, info};
use std::path::{Path, PathBuf};
mod errors;
use errors::Result;
mod chart_hashes;
mod fsutil;
mod ops;
mod table_loader;
use clap::{Parser, Subcommand};
use std::env;
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    dryrun: bool,

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

        #[clap(short, long, help = "check level limit")]
        level_limit: Option<u8>,
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
        #[clap(long, help = "EASY / NORMAL / HARD")]
        target_lamp: String,
        #[clap(long)]
        task_notes: u32,
    },
}

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let start = Instant::now();

    let cli = Cli::parse();

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
        } => {
            ops::check_table_coverage::check_table_coverage(table_url, mydir, level_limit)?;
        }
        Commands::Install { from, recursive } => {
            let from = &Path::new(&from);
            if !from.is_dir() {
                error!("from is not a directory");
                return Err("from is not a directory".into());
            }
            if *recursive {
                ops::install_from_dir::install_from_dirs(&from, &mydir, dryrun)?;
            } else {
                ops::install_from_dir::install_from_dir(&from, &mydir, dryrun)?;
            }
        }
        Commands::Organize { dest } => {
            let dest = dest.as_ref().map(|d| Path::new(d)).unwrap_or(mydir);
            if !dest.is_dir() {
                error!("dest is not a directory");
                return Err("dest is not a directory".into());
            }

            info!("== rename ==");
            ops::rename::rename_dirs(&mydir, dryrun)?;
            info!("== merge ==");
            ops::merge::merge(&mydir, dryrun)?;
            info!("== reconstruct ==");
            ops::reconstruct::reconstruct(&mydir, &dest, dryrun)?;
        }
        Commands::Rename {} => {
            ops::rename::rename_dirs(&mydir, dryrun)?;
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
            let lamp_num = match target_lamp.as_str() {
                "ASSIST_EASY" => Ok(3_u8),
                "EASY" => Ok(4_u8),
                "NORMAL" => Ok(5_u8),
                "HARD" => Ok(6_u8),
                _ => Err("Invalid target-lamp"),
            }?;
            ops::create_task_folder::create_task_folder(
                table_url,
                player_score_path,
                songdata_path,
                folder_default_json,
                *lower_limit_level,
                lamp_num,
                *task_notes,
            )?;
        }
    }

    info!("Time elapsed: {:?}", start.elapsed());

    Ok(())
}
