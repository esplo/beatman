use log::warn;
use log::{debug, error, info};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use crate::chart_hashes::ChartHashes;
use crate::errors::Result;
use crate::fsutil;

const MERGE_THRESHOLD: u8 = 80;

fn is_duplicated(path1: &Path, path2: &Path, threshold: Option<u8>) -> Result<bool> {
    let threshold = threshold.unwrap_or(MERGE_THRESHOLD);

    if !path1.is_dir() || !path2.is_dir() {
        return Err(format!("{:?} or {:?} is not a directory", path1, path2).into());
    }

    // compare contants in two different pathes
    let lookup = |dir: &Path| -> Result<HashSet<OsString>> {
        let files = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .flat_map(|e| e.path().file_name().map(|e| e.to_os_string()))
            .collect();
        Ok(files)
    };

    let contents1 = lookup(path1)?;
    let contents2 = lookup(path2)?;
    let diff_size = contents1.symmetric_difference(&contents2).count();
    let size_sum = contents1.len() + contents2.len();
    let dupe_confidence = 100 * (size_sum - diff_size) / size_sum;
    Ok(dupe_confidence >= threshold.into())
}

pub fn merge(current_dir: &Path, dryrun: bool) -> Result<()> {
    let chart_hashes = ChartHashes::new(current_dir)?;

    let mut merge_targets: HashMap<&Path, &Path> = HashMap::new();
    for v in chart_hashes.hashes().values() {
        if v.len() > 1 {
            // compare 1st and 2nd, and move to 2nd, iterate this...
            // zip (0,1), (1,2), (2,3), ...
            let original = v;
            let slided = &v[1..];
            for (p1, p2) in original.iter().zip(slided) {
                if let (Some(p1p), Some(p2p)) = (p1.parent(), p2.parent()) {
                    match is_duplicated(p1p, p2p, None) {
                        Ok(dup) => {
                            if dup {
                                debug!("duplicated: {}, {:?} {:?}", dup, p1, p2);
                                merge_targets.insert(p2p, p1p);
                            }
                        }
                        Err(e) => error!("{} for {:?} or {:?}", e, p1, p2),
                    }
                }
            }
        }
    }

    // check duplication
    let flattened_merge_targets: Vec<(&Path, &Path)> = merge_targets
        .iter()
        .map(|(from, dest)| {
            let mut target = dest;
            // only 10th depth to avoid infinite loop
            for _ in 0..10 {
                if merge_targets.contains_key(target) {
                    target = merge_targets.get(target).as_mut().unwrap();
                } else {
                    break;
                }
            }
            (*from, *target)
        })
        .collect();

    for (from, dest) in &flattened_merge_targets {
        info!("merge: {:?} -> {:?}", from, dest);

        //TODO: I don't know the reason, but this occurs
        if from != dest {
            if !dryrun {
                fsutil::move_and_remove_dir(from, dest)?;
            }
        } else {
            warn!("{:?} contains the same chart file?", from);
        }
    }

    Ok(())
}
