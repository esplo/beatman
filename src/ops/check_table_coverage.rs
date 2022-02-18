use log::{debug, info};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::chart_hashes::ChartHashes;
use crate::errors::Result;
use crate::fsutil;
use crate::table_loader;
use std::io;
use std::time::SystemTime;

pub fn check_table_coverage(
    score_url: &str,
    current_dir: &Path,
    level_limit: &Option<u8>,
) -> Result<()> {
    let table = table_loader::TableLoader::new(score_url)?;
    let chart_hashes = ChartHashes::new(&current_dir)?;

    let filtered_table = table.charts().iter().filter(|sd| {
        level_limit.map_or(true, |l| {
            sd.level.parse().map(|t: u8| t <= l).unwrap_or(true)
        })
    });

    let mut total = 0;
    let mut counter = 0;
    filtered_table.for_each(|sd| {
        total += 1;
        let hashes = chart_hashes.hashes();
        if hashes.contains_key(&sd.sha256) {
            debug!("found! {:?}", hashes.get(&sd.sha256).unwrap()[0]);
            counter += 1;
        } else {
            info!("not found! [{}] {}", sd.level, sd.title);
            info!("--url--> {}", sd.url);
            if !sd.url_diff.is_empty() {
                info!("--diff--> {}", sd.url_diff);
            }
        }
    });

    info!("{} / {} charts found", counter, total);

    Ok(())
}
