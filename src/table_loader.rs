use crate::errors::Result;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ScoreData {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub url_diff: String,
    pub sha256: String,
    pub level: String,
    // #[serde(default)]
    // pub comment: String,
}

pub struct TableLoader {
    charts: Vec<ScoreData>,
}

impl TableLoader {
    pub fn new(url: &str) -> Result<Self> {
        let url = match url {
            _ if url.ends_with("table.html") => url.replace("table.html", "score.json"),
            _ => url.to_owned(),
        };

        let table_charts = reqwest::blocking::get(url)?.json::<Vec<ScoreData>>()?;

        Ok(TableLoader {
            charts: table_charts,
        })
    }

    pub fn charts(&self) -> &Vec<ScoreData> {
        &self.charts
    }
}
