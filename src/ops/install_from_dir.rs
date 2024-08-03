use crate::errors::Result;
use crate::ops::rename::rename_dirs;
use chrono::{DateTime, Utc};
use core::str;
use log::{debug, info, warn};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use unrar::Archive;

pub fn install_from_dirs(target_dir: &Path, dest_dir: &Path, dryrun: bool) -> Result<()> {
    let dirs: Vec<fs::DirEntry> = target_dir
        .read_dir()?
        .flatten()
        .filter(|e| e.file_type().unwrap().is_dir())
        .collect();

    for d in &dirs {
        info!("target_dir {:?}", d.file_name());
        install_from_dir(&d.path(), dest_dir, dryrun)?;
        // delete
        if !dryrun {
            fs::remove_dir(d.path()).unwrap_or_else(|e| warn!("failed to remove dir: {:?}", e));
        }
    }

    Ok(())
}

pub fn install_from_dir(target_dir: &Path, dest_dir: &Path, dryrun: bool) -> Result<()> {
    // サブフォルダを対象ディレクトリに追加
    let utc: DateTime<Utc> = Utc::now();
    let format = "%s%6f";
    let ts = utc.format(format).to_string();

    let dest_dir = dest_dir.join(ts);

    // lookup zip files
    let zips: Vec<fs::DirEntry> = target_dir
        .read_dir()?
        .flatten()
        .filter(|e| {
            let path = e.path();
            let ext = path.extension();
            ext == Some(OsStr::new("zip")) || ext == Some(OsStr::new("rar"))
        })
        .collect();

    for zip_file in zips {
        info!(
            "extracting: source:{:?} , dest:{:?}",
            zip_file.file_name(),
            dest_dir
        );

        if dryrun {
            continue;
        }

        match zip_file.path().extension() {
            None => continue,
            Some(os_str) => match os_str.to_str() {
                Some("zip") => {
                    let file = fs::File::open(zip_file.path()).unwrap();
                    let mut archive = zip::ZipArchive::new(file).unwrap();

                    for i in 0..archive.len() {
                        let mut file = archive.by_index(i)?;

                        // UTF-8かSJISかを判定する
                        let sjis_name = encoding_rs::SHIFT_JIS.decode(file.name_raw()).0;
                        let archived_file_name =
                            str::from_utf8(file.name_raw()).unwrap_or(&sjis_name);

                        let file_name = Path::new(archived_file_name).file_name().unwrap();
                        let t = &dest_dir.join(Path::new(file_name));

                        t.parent().map(fs::create_dir_all);
                        let mut output = fs::File::create(t)
                            .unwrap_or_else(|_| panic!("Failed to create file: {:?}", t));
                        std::io::copy(&mut file, &mut output)?;
                    }
                }
                Some("rar") => {
                    let mut archive = Archive::new(&zip_file.path())
                        .open_for_processing()
                        .unwrap();
                    while let Some(header) = archive.read_header()? {
                        debug!(
                            "{} bytes: {}",
                            header.entry().unpacked_size,
                            header.entry().filename.to_string_lossy(),
                        );
                        archive = if header.entry().is_file() {
                            let archived_path = header.entry().filename.file_name();
                            match archived_path {
                                Some(n) => {
                                    let t = dest_dir.join(Path::new(n));
                                    header.extract_to(t)?
                                }
                                None => {
                                    warn!(
                                        "Invalid file {:?} in {:?}",
                                        &header.entry().filename,
                                        zip_file.path()
                                    );
                                    header.skip()?
                                }
                            }
                        } else {
                            header.skip()?
                        };
                    }
                }
                _ => panic!("unknown extension"),
            },
        };

        // delete
        fs::remove_file(zip_file.path())?;
    }

    // rename
    rename_dirs(&dest_dir, dryrun)?;

    Ok(())
}
