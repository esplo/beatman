use crate::chart_hashes::{filter_bms_files, ChartHashes};
use crate::errors::Result;
use crate::fsutil;
use log::warn;
use log::{debug, info};
use rayon::prelude::*;
use regex::Regex;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;

fn read_artist_and_title(lines: Vec<String>) -> Result<(Option<String>, Option<String>)> {
    let mut artist = None;
    let mut title = None;

    for l in lines {
        // filter title, artist
        if let Some(t) = l.strip_prefix("#ARTIST ") {
            artist = Some(String::from(t))
        }
        if let Some(t) = l.strip_prefix("#TITLE ") {
            title = Some(String::from(t));
        }
        if artist.is_some() && title.is_some() {
            break;
        }
    }

    Ok((artist, title))
}

fn sanitize_str(s: &str, n: usize) -> String {
    // replace (possibly) invalid path string
    let s = ["/", "\"", "?", "<", ">", "*", ":", "|", "."]
        .iter()
        .fold(String::from(s), |s, inv| s.replace(inv, ""));
    let mut m = std::cmp::min(s.len(), n);
    while !s.is_char_boundary(m) {
        m += 1;
    }
    s[..m].to_owned()
}

fn lookup_names<T>(charts_paths: T) -> impl Iterator<Item = Result<(String, String)>>
where
    T: Iterator<Item = PathBuf>,
{
    charts_paths.map(|path| {
        let file = fs::File::open(&path)?;
        let lines = io::BufReader::new(file)
            .lines()
            .map_while(|x| x.ok())
            .collect::<Vec<String>>();
        let (artist1, title1) = read_artist_and_title(lines)?;

        // if one of info is None, read again in SHIFT-JIS
        let (artist2, title2) = if artist1.is_none() || title1.is_none() {
            let bytes = fs::read(&path)?;
            let (res, enc, b) = encoding_rs::SHIFT_JIS.decode(&bytes);
            debug!("{:?} {:?}", enc, b);
            let lines = res.into_owned().lines().map(String::from).collect();
            read_artist_and_title(lines)?
        } else {
            (None, None)
        };

        let artist = artist1.or(artist2);
        let title = title1.or(title2);
        debug!("{:?}, {:?}, {:?}", &path, artist, title);

        if let (Some(a), Some(t)) = (&artist, &title) {
            if a.is_empty() || t.is_empty() {
                let msg: String = format!("artist or title is blank: {:?} {:?}", &artist, &title);
                Err(msg.into())
            } else {
                Ok((a.clone(), t.clone()))
            }
        } else {
            let msg: String = format!("cannot read artist or title: {:?} {:?}", &artist, &title);
            Err(msg.into())
        }
    })
}

// if the name has '[.*]' and other strings, remove it
fn remove_difficulty(name: &str) -> String {
    let re = Regex::new(r"(?P<name>.+?)(?P<diff>\s*\[.*?\])\s*$").unwrap();
    let caps = re.captures(name);
    debug!("{:?}", caps);
    caps.and_then(|cap| cap.name("name").map(|e| e.as_str()))
        .unwrap_or(name)
        .to_owned()
}

fn read_files_and_name(dir: &Path) -> Option<OsString> {
    let charts_paths = dir
        .read_dir()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .filter(|file| filter_bms_files(&file.path()))
        .map(|e| e.path());

    let names = lookup_names(charts_paths);

    // find shortest name (which is the most likely "the answer")
    let info: Vec<(String, String)> = names.flatten().collect();
    let mut artists: Vec<String> = info.iter().map(|e| e.0.clone()).collect();
    let mut titles: Vec<String> = info.iter().map(|e| remove_difficulty(&e.1)).collect();

    artists.sort_by_key(|a| a.len());
    titles.sort_by_key(|a| a.len());

    let artist = artists.first();
    let title = titles.first();

    if let (Some(a), Some(t)) = (artist, title) {
        Some(OsString::from(format!(
            "[{}] {}",
            sanitize_str(a, 50),
            sanitize_str(t, 100)
        )))
    } else {
        None
    }
}

pub fn rename_dirs(current_dir: &Path, dryrun: bool) -> Result<()> {
    let chart_hashes = ChartHashes::new(current_dir)?;
    let parents = chart_hashes.parents()?;

    // process in parallel, but rename sequentially
    let mut rename_targets = vec![];
    let vec_parents = Vec::from_iter(parents);
    vec_parents
        .par_iter()
        .map(|path| {
            read_files_and_name(path).and_then(|name| {
                path.parent()
                    .map(|par| par.join(PathBuf::from(name)))
                    .map(|n| (path, n))
            })
        })
        .collect_into_vec(&mut rename_targets);

    rename_targets
        .into_iter()
        .flatten()
        .for_each(|(from, dest)| {
            if from.as_os_str() != dest.as_os_str() {
                info!("rename {:?} -> {:?}", from, dest);
                if dest.exists() {
                    warn!("rename cancelled. destination already exists. {:?}", dest);
                } else if !dryrun {
                    fsutil::move_and_remove_dir(from, &dest)
                        .unwrap_or_else(|_| warn!("rename failed. {:?} -> {:?}", from, dest))
                }
            }
        });

    Ok(())
}
