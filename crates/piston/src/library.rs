use crate::rule::Rule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
}

// TODO: Support classifiers
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryDownload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownload {
    pub path: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}
