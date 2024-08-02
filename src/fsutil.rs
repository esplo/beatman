use crate::errors::Result;
use log::debug;
use std::path::Path;
use std::{fs, io};

pub fn move_and_remove_dir(from: &Path, dest: &Path) -> Result<()> {
    debug!("from: {:?}, dest: {:?}", from, dest);

    if !dest.exists() {
        debug!("create dir: {:?}", dest);
        fs::create_dir_all(dest)?;
    }
    if !from.exists() || !dest.exists() {
        let missing = if !from.exists() { from } else { dest };
        return Err(format!("{:?} is not a directory", missing).into());
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            move_and_remove_dir(&entry.path(), &dest.join(entry.file_name()))?;
        } else {
            // ignore troublesome files
            debug!("{:?}", entry.path().file_name());
            if entry.file_name() == *"desktop.ini" || entry.file_name() == *".DS_Store" {
                continue;
            }

            let to = dest.join(entry.file_name());

            debug!("mv {:?} -> {:?}", entry.path(), to);

            // use mv as possible.
            // if it's not a CrossesDevices, use mv.
            // otherwise, use cp.
            match fs::rename(entry.path(), &to) {
                // TODO: track issue
                // currently we cannot use this feature. see issue: https://github.com/rust-lang/rust/issues/86442
                // Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {},
                Err(_) => {
                    // just try again
                    debug!(
                        "failed to mv. use cp instead: {:?} -> {:?}",
                        entry.path(),
                        to
                    );
                    fs::copy(entry.path(), to).map(|_| ())
                }
                v => v,
            }?;
        }
    }
    fs::remove_dir_all(from)?;
    Ok(())
}

// remove empty dir
pub fn remove_empty_dirs(target_dir: &Path) -> io::Result<()> {
    for d in fs::read_dir(target_dir)? {
        let entry = d?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            let is_empty = fs::read_dir(entry.path())?.next().is_none();
            if is_empty {
                debug!("remove dir {:?}", entry.path());
                fs::remove_dir(entry.path())?;
            }
        }
    }

    Ok(())
}
