use crate::chart_hashes::ChartHashes;
use crate::errors::Result;
use crate::fsutil;
use log::info;
use md5::{Digest, Md5};
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub fn reconstruct(current_dir: &Path, dest_dir: &Path, dryrun: bool, shard: bool) -> Result<()> {
    let chart_hashes = ChartHashes::new(current_dir)?;
    let parents = chart_hashes.parents()?;

    for d in &parents {
        let mut move_to_dir = PathBuf::from(&dest_dir);
        let file_name = d.file_name().unwrap();
        if shard {
            let mut hasher = Md5::new();
            hasher.update(file_name.as_bytes());
            let hash_bytes = hasher.finalize();

            let hex_string: String = hash_bytes
                .iter()
                .map(|byte| format!("{:02X}", byte))
                .collect();
            // 先頭二文字をハッシュとする
            move_to_dir.push(Path::new(&hex_string[0..2]));
        }
        move_to_dir.push(Path::new(file_name));
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
