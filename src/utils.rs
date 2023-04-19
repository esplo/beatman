use crate::errors::Result;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultTable {
    name: String,
    folder: Option<Vec<DefaultTableFolder>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultTableFolder {
    class: String,
    name: String,
    songs: Vec<DefaultTableSong>,
}
impl DefaultTableFolder {
    fn new(name: String, songs: Vec<DefaultTableSong>) -> Self {
        DefaultTableFolder {
            class: "bms.player.beatoraja.TableData$TableFolder".to_owned(),
            name,
            songs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultTableSong {
    class: String,
    title: String,
    sha256: String,
}
impl DefaultTableSong {
    pub fn new(title: String, sha256: String) -> Self {
        DefaultTableSong {
            class: "bms.player.beatoraja.song.SongData".to_owned(),
            title,
            sha256,
        }
    }
}

pub fn add_table_default_json(
    folder_default_json_path: &Path,
    folder_name: String,
    songs: Vec<DefaultTableSong>,
) -> Result<()> {
    let new_folder = DefaultTableFolder::new(folder_name, songs);
    debug!("{:?}", new_folder);

    {
        let file = fs::File::open(folder_default_json_path)?;
        let mut default_json: DefaultTable = serde_json::from_reader(file)?;

        match default_json.folder {
            Some(ref mut f) => f.push(new_folder),
            None => default_json.folder = Some(vec![new_folder]),
        }

        debug!("{:?}", &default_json);

        fs::write(
            folder_default_json_path,
            &serde_json::to_string(&default_json)?,
        )?;
    }

    info!("add to default.json.");

    Ok(())
}
