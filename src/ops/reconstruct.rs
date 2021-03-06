use log::info;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::chart_hashes::ChartHashes;
use crate::errors::Result;
use crate::fsutil;

pub fn reconstruct(current_dir: &Path, dest_dir: &Path, dryrun: bool) -> Result<()> {
    let chart_hashes = ChartHashes::new(&current_dir)?;
    let parents: HashSet<&Path> = HashSet::from_iter(
        chart_hashes
            .hashes()
            .iter()
            .flat_map(|(_, v)| v[0].parent())
            .collect::<Vec<&Path>>(),
    );

    for d in &parents {
        let mut move_to_dir = PathBuf::from(&dest_dir);
        move_to_dir.push(&Path::new(d.file_name().unwrap()));
        if d != &move_to_dir {
            info!("move: {:?} -> {:?}", d, move_to_dir);

            if !dryrun {
                fs::create_dir_all(&move_to_dir)?;
                fsutil::move_and_remove_dir(d, &move_to_dir)?;
            }
        }
    }

    if !dryrun {
        fsutil::remove_empty_dirs(current_dir)?;
    }

    Ok(())
}
