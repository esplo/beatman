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

pub fn add_and_write_table_json(
    table_json_path: &Path,
    folder_name: String,
    songs: Vec<DefaultTableSong>,
) -> Result<()> {
    debug!("add_table_default_json");
    let new_folder = DefaultTableFolder::new(folder_name, songs);
    debug!("{:?}", new_folder);

    {
        let file = fs::File::open(table_json_path)?;
        let mut default_json: DefaultTable = serde_json::from_reader(file)?;

        match default_json.folder {
            Some(ref mut f) => f.push(new_folder),
            None => default_json.folder = Some(vec![new_folder]),
        }

        debug!("{:?}", &default_json);
        debug!("write to {:?}", table_json_path);

        fs::write(table_json_path, &serde_json::to_string(&default_json)?)?;
    }

    info!("add to folder json.");

    Ok(())
}

pub fn lamp_to_id(lamp: &str) -> Result<u8> {
    match lamp {
        "AEASY" => Ok(3),
        "EASY" => Ok(4),
        "NORMAL" => Ok(5),
        "HARD" => Ok(6),
        "EXHARD" => Ok(7),
        "FC" => Ok(8),
        "PERFECT" => Ok(9),
        &_ => Err(format!("Invalid Lamp: {}", lamp).into()),
    }
}
