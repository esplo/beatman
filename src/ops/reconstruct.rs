use log::info;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::chart_hashes::ChartHashes;
use crate::errors::Result;
use crate::fsutil;

pub fn reconstruct(current_dir: &Path, dest_dir: &Path, dryrun: bool, shard: bool) -> Result<()> {
    let chart_hashes = ChartHashes::new(&current_dir)?;
    let parents = chart_hashes.parents()?;
    let mut hasher = DefaultHasher::new();

    for d in &parents {
        let mut move_to_dir = PathBuf::from(&dest_dir);
        let file_name = d.file_name().unwrap();
        if shard {
            file_name.hash(&mut hasher);
            let hash = hasher.finish() % 10;
            move_to_dir.push(&Path::new(&hash.to_string()));
        }
        move_to_dir.push(&Path::new(file_name));
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
