use crate::errors::Result;
use crate::fsutil;
use log::{debug, info};
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

pub fn install_from_dirs(target_dir: &Path, dest_dir: &Path, dryrun: bool) -> Result<()> {
    let dirs: Vec<fs::DirEntry> = target_dir
        .read_dir()?
        .flatten()
        .filter(|e| e.file_type().unwrap().is_dir())
        .collect();

    for d in &dirs {
        info!("target_dir {:?}", d.file_name());
        install_from_dir(&d.path(), &dest_dir, dryrun)?;
    }

    Ok(())
}

pub fn install_from_dir(target_dir: &Path, dest_dir: &Path, dryrun: bool) -> Result<()> {
    // lookup zip files
    let zips: Vec<fs::DirEntry> = target_dir
        .read_dir()?
        .flatten()
        .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("zip"))
        .collect();

    // define destination folder name
    // this will be detected automatically from zips.
    // - use the zip file with the largest number of files
    let ts = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_micros(),
        Err(_) => 0,
    };
    let mut dest_name = (0, ts.to_string().into());

    for zip_file in zips {
        info!("extracting {:?}", zip_file.file_name());

        let file = fs::File::open(&zip_file.path()).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let n = archive.len();

        // update name as zip's name. this will be overwritten as the name of the inside dir
        if dest_name.0 < n {
            // set n-1 in order to update later
            dest_name = (n - 1, zip_file.file_name().to_os_string());
        }

        for i in 0..n {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            // update dest_name with its the top dir
            if dest_name.0 < n && outpath.parent().is_some() {
                let top_dir = outpath
                    .ancestors()
                    .into_iter()
                    .filter(|e| e != &Path::new("") && e != &Path::new("/"))
                    .last();

                if let Some(name) = top_dir {
                    // update once, by setting n
                    dest_name = (n, std::ffi::OsString::from(name));
                }
            }

            // flatten file (remove directory name)
            let outpath = zip_file
                .path()
                .parent()
                .unwrap()
                .join(&outpath.file_name().unwrap());

            if (*file.name()).ends_with('/') {
                debug!("ignore {}", file.name());
            } else {
                debug!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );

                if !dryrun {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p).unwrap();
                        }
                    }
                    let mut outfile = fs::File::create(&outpath).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }
            }
        }

        if !dryrun {
            fs::remove_file(zip_file.path())?;
        }
    }

    if !dryrun {
        let dest = dest_dir.join(dest_name.1);
        fs::create_dir_all(&dest)?;
        fsutil::move_and_remove_dir(target_dir, &dest)?;
    }

    Ok(())
}
