use crate::errors::Result;
use jwalk::WalkDir;
use jwalk::WalkDirGeneric;
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use rayon::prelude::*;

type ChartHashMap = HashMap<String, Vec<PathBuf>>;

pub const BMS_EXTENSIONS: &'static [&str] = &["bms", "bml", "bme", "pms"];

fn charts_traverse(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir).into_iter()
    .flatten()
    .filter(|entry| entry.file_type().is_file())
    .filter(|file|
        BMS_EXTENSIONS
            .iter()
            .any(|e| file.path().extension() == Some(std::ffi::OsStr::new(e)))
    )
    .map(|e| e.path())
    .collect()
}

pub struct ChartHashes {
    hash_with_dir: ChartHashMap,
}

impl ChartHashes {
    pub fn new(dir: &Path) -> Result<Self> {
        info!("looking charts up....");
        let charts = charts_traverse(&dir);
        info!("found {:?} charts", charts.len());

        // calculate hashes parallely
        let chart_with_hashes: Vec<(String, &Path)>  = charts
        .par_iter()
        .filter(|path| !path.starts_with("$RECYCLE.BIN") && !path.starts_with("."))
        .map(|path| {
            let with_hash = || -> std::result::Result<(String, &Path), Box<dyn std::error::Error + Send + Sync>> {
                let mut file = fs::File::open(&path)?;
                let mut hasher = Sha256::new();
                io::copy(&mut file, &mut hasher)?;
                let hash = hasher.finalize();
                let hash_string = format!("{:x}", hash);
                debug!("Binary hash of {:?} is {}", path, hash_string);
    
                Ok((hash_string, path))
            };
            with_hash()
        })
        .flatten()
        .collect::<Vec<(String, &Path)>>();

        info!("hashed {:?} charts", chart_with_hashes.len());

        let mut hash_with_dir = HashMap::new();
        chart_with_hashes.into_iter().for_each(|(hash, path)| {
            hash_with_dir
            .entry(hash)
            .or_insert(vec![])
            .push(path.to_path_buf());
        });

        info!("different {:?} charts", hash_with_dir.len());
    
        Ok( ChartHashes {
            hash_with_dir,
        })
    }

    pub fn hashes(&self) -> &ChartHashMap {
        &self.hash_with_dir
    }
}
