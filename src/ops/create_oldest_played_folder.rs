use crate::errors::Result;
use crate::utils::{add_and_write_table_json, lamp_to_id, DefaultTableSong};
use chrono::TimeZone;
use chrono::{Local, Utc};
use log::{debug, info};
use rusqlite::{named_params, Connection, OpenFlags};
use std::fs::OpenOptions;
use std::io::Write;
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
    table_json_path: &Path,
    target_lamp: &str,
    reset: bool,
) -> Result<()> {
    info!("open {:?}", player_score_path);
    let player_scores =
        Connection::open_with_flags(player_score_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let lamp_id = lamp_to_id(target_lamp)?;

    let mut query_player_score_stmt = player_scores.prepare(
        "
            SELECT sha256, clear, playcount, minbp, scorehash, date 
            FROM score
            WHERE clear < :lamp_id
            ORDER BY date ASC
            LIMIT 30
            ",
    )?;

    debug!("run query");
    let player_scores = query_player_score_stmt
        .query_map(named_params! { ":lamp_id": lamp_id }, |row| {
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
        .map(|t| {
            let dt = Utc.timestamp_opt(t.date as i64, 0).unwrap();
            let now = Utc::now();
            let diff = now - dt;
            let days = diff.num_days();

            DefaultTableSong::new(format!("{} days ago", days).to_string(), t.sha256)
        })
        .collect();

    debug!("{:?}", songs);

    let folder_name = format!(
        "OLDEST_<{}_{}-{}",
        target_lamp,
        Local::now().format("%Y.%m.%d"),
        Local::now().timestamp() % 1000
    );

    let init_body = r#"{"name":"Oldest","folder":[]}"#;
    if !table_json_path.exists() {
        // create a new file, named oldest.json
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .truncate(true)
            .open(table_json_path)?;
        f.write_all(init_body.as_bytes())?;
    }
    if reset {
        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(table_json_path)?;
        f.write_all(init_body.as_bytes())?;
    }
    add_and_write_table_json(table_json_path, folder_name, songs)?;

    Ok(())
}
