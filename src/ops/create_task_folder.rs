use crate::errors::Result;
use crate::table_loader;
use chrono::Local;
use log::{debug, info};
use rusqlite::{named_params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct TableData {
    sha256: String,
    title: String,
    level: u32,
}

#[derive(Debug, Clone)]
struct Score {
    #[allow(dead_code)]
    sha256: String,
    clear: u8,
    playcount: u32,
    #[allow(dead_code)]
    minbp: u32,
    #[allow(dead_code)]
    scorehash: String,
}

#[derive(Debug, Clone)]
struct ChartInfo {
    #[allow(dead_code)]
    sha256: String,
    totalnotes: u32,
}

#[derive(Debug, Clone)]
struct TableDataWithScore {
    table_data: TableData,
    score: Option<Score>,
    totalnotes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultTable {
    name: String,
    folder: Option<Vec<DefaultTableFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultTableFolder {
    class: String,
    name: String,
    songs: Vec<DefaultTableSong>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultTableSong {
    class: String,
    title: String,
    sha256: String,
}

pub fn create_task_folder(
    table_url: &str,
    player_score_path: &Path,
    songdata_path: &Path,
    folder_default_json: &Path,
    lower_limit_level: u8,
    target_lamp: u8,
    task_notes: u32,
) -> Result<()> {
    info!("open {:?}", player_score_path);
    let player_scores =
        Connection::open_with_flags(player_score_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    info!("open {:?}", songdata_path);
    let songdata = Connection::open_with_flags(songdata_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    info!("load table {:?}", table_url);
    let table = table_loader::TableLoader::new(table_url)?;
    let table: Vec<TableData> = table
        .charts()
        .iter()
        .filter(|sd| {
            sd.level
                .parse()
                .map(|t: u8| t >= lower_limit_level)
                .unwrap_or(true)
        })
        .map(|sd| {
            let u_level = sd.level.parse()?;
            Ok(TableData {
                sha256: sd.sha256.to_owned(),
                title: sd.title.to_owned(),
                level: u_level,
            })
        })
        .collect::<Result<Vec<TableData>>>()?;

    info!("append clear data for {:?} charts", table.len());
    let mut query_player_score_stmt = player_scores.prepare(
        "SELECT sha256, clear, playcount, minbp, scorehash FROM score WHERE sha256 = :sha256 LIMIT 1",
    )?;
    let mut chart_info_stmt =
        songdata.prepare("SELECT sha256, notes FROM song WHERE sha256 = :sha256 LIMIT 1")?;

    let target_charts = table.iter().map(|td| {
        debug!("target hash {:?}", td.sha256);

        let player_score = query_player_score_stmt
            .query_map(named_params! { ":sha256": td.sha256 }, |row| {
                Ok(Score {
                    sha256: row.get(0)?,
                    clear: row.get(1)?,
                    playcount: row.get(2)?,
                    minbp: row.get(3)?,
                    scorehash: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let player_score = player_score.flatten().nth(0);
        if player_score.is_some() {
            debug!("player_score {:?}", player_score);
        }

        let chart_info = chart_info_stmt
            .query_map(named_params! { ":sha256": td.sha256 }, |row| {
                Ok(ChartInfo {
                    sha256: row.get(0)?,
                    // total is REAL type in SQLite. convert it as u32
                    totalnotes: ((row.get::<usize, f32>(1)?).round() as u32),
                })
            })
            .map_err(|e| e.to_string())?;
        let chart_info = chart_info.flatten().nth(0).ok_or("Chart Info Not Found")?;
        debug!("chart_info {:?}", chart_info);

        Ok::<TableDataWithScore, Box<dyn std::error::Error>>(TableDataWithScore {
            table_data: (*td).clone(),
            score: player_score,
            totalnotes: chart_info.totalnotes,
        })
    });

    let mut non_achieved_charts: Vec<TableDataWithScore> = target_charts
        .flatten()
        .filter(|td| {
            // filter not achieved charts
            match &td.score {
                None => true, // pass
                Some(s) => s.clear < target_lamp,
            }
        })
        .collect();

    // sort by (level, clear_lamp, playcount)
    let cmpfunc = |a: &TableDataWithScore| -> (u32, u8, u32) {
        (
            a.table_data.level,
            a.score.as_ref().map_or(0, |s| s.clear),
            a.score.as_ref().map_or(0, |s| s.playcount),
        )
    };
    non_achieved_charts.sort_by(|a, b| cmpfunc(a).partial_cmp(&cmpfunc(b)).unwrap());
    let mut notes = 0;
    let tasks: Vec<DefaultTableSong> = non_achieved_charts
        .iter()
        .take_while(|s| {
            notes += s.totalnotes;
            // do not omit the last chart
            notes - s.totalnotes < task_notes
        })
        .map(|t| DefaultTableSong {
            class: "bms.player.beatoraja.song.SongData".to_owned(),
            title: t.table_data.title.to_owned(),
            sha256: t.table_data.sha256.to_owned(),
        })
        .collect();

    debug!("{:?}", tasks);

    let folder_name = format!(
        "{} {} NOTES {}",
        Local::today().format("%Y.%m.%d"),
        notes,
        Local::now().timestamp() % 100
    );
    let new_folder = DefaultTableFolder {
        class: "bms.player.beatoraja.TableData$TableFolder".to_owned(),
        name: folder_name,
        songs: tasks,
    };

    debug!("{:?}", new_folder);

    let file = fs::File::open(folder_default_json)?;
    let mut default_json: DefaultTable = serde_json::from_reader(file)?;

    match default_json.folder {
        Some(ref mut f) => f.push(new_folder),
        None => default_json.folder = Some(vec![new_folder]),
    }

    debug!("{:?}", &default_json);

    fs::write(folder_default_json, &serde_json::to_string(&default_json)?)?;

    info!("wrote today's task.");

    Ok(())
}
