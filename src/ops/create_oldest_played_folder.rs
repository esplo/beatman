use crate::errors::Result;
use crate::utils::{add_table_default_json, DefaultTableSong};
use chrono::TimeZone;
use chrono::{Local, Utc};
use log::{debug, info};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

#[derive(Debug, Clone)]
struct Score {
    #[allow(dead_code)]
    sha256: String,
    #[allow(dead_code)]
    clear: u8,
    #[allow(dead_code)]
    playcount: u32,
    #[allow(dead_code)]
    minbp: u32,
    #[allow(dead_code)]
    scorehash: String,
    #[allow(dead_code)]
    date: u64,
}

pub fn create_oldest_played_folder(
    player_score_path: &Path,
    // songdata_path: &Path,
    folder_default_json_path: &Path,
    _target_lamp: u8,
) -> Result<()> {
    info!("open {:?}", player_score_path);
    let player_scores =
        Connection::open_with_flags(player_score_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let mut query_player_score_stmt = player_scores.prepare(
        "
            SELECT sha256, clear, playcount, minbp, scorehash, date 
            FROM score
            ORDER BY date ASC
            LIMIT 60
            ",
    )?;

    debug!("run query");
    let player_scores = query_player_score_stmt
        .query_map([], |row| {
            Ok(Score {
                sha256: row.get(0)?,
                clear: row.get(1)?,
                playcount: row.get(2)?,
                minbp: row.get(3)?,
                scorehash: row.get(4)?,
                date: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    debug!("done query");

    let songs: Vec<DefaultTableSong> = player_scores
        .flatten()
        .into_iter()
        .map(|t| {
            let dt = Utc.timestamp(t.date as i64, 0);
            let now = Utc::now();
            let diff = now - dt;
            let days = diff.num_days();

            DefaultTableSong::new(format!("{} days ago", days).to_string(), t.sha256)
        })
        .collect();

    debug!("{:?}", songs);

    let folder_name = format!(
        "OLDEST_{}-{}",
        Local::today().format("%Y.%m.%d"),
        Local::now().timestamp() % 100
    );

    add_table_default_json(folder_default_json_path, folder_name, songs)?;

    Ok(())
}
