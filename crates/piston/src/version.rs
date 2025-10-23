use std::collections::HashMap;

use crate::{argument::Arguments, library::Library};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<Version>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    #[serde(serialize_with = "serialize_date")]
    pub time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_date")]
    pub release_time: DateTime<Utc>,
    pub sha1: String,
    pub compliance_level: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedVersion {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<u32>,
    pub downloads: Downloads,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    pub main_class: String,
    pub minimum_launcher_version: u32,
    #[serde(serialize_with = "serialize_date")]
    pub release_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_date")]
    pub time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub version_type: VersionType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
    pub client: Download,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_mappings: Option<Download>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Download>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_mappings: Option<Download>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_server: Option<Download>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub total_size: u32,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchedAssetIndex {
    pub objects: HashMap<String, Object>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub logging_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

// default serializer doesn't output the same as piston meta
fn serialize_date<S: Serializer>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use crate::VERSION_MANIFEST_URL;

    use super::*;
    use anyhow::{Result, anyhow};

    #[tokio::test]
    async fn download_version_manifest() -> Result<()> {
        let client = http::Client::new();
        serde_json::from_slice::<VersionManifest>(
            &client.download(VERSION_MANIFEST_URL, None).await?,
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn download_versions() -> Result<()> {
        let client = http::Client::new();

        let version_manifest = serde_json::from_slice::<VersionManifest>(
            &client.download(VERSION_MANIFEST_URL, None).await?,
        )?;

        for version in version_manifest.versions {
            serde_json::from_slice::<FetchedVersion>(
                &client.download(&version.url, Some(&version.sha1)).await?,
            )?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn download_latest_asset_index() -> Result<()> {
        let client = http::Client::new();

        let version_manifest = serde_json::from_slice::<VersionManifest>(
            &client.download(VERSION_MANIFEST_URL, None).await?,
        )?;

        for version in version_manifest.versions {
            if version.id == version_manifest.latest.release {
                let fetched_version = serde_json::from_slice::<FetchedVersion>(
                    &client.download(&version.url, Some(&version.sha1)).await?,
                )?;
                serde_json::from_slice::<FetchedAssetIndex>(
                    &client
                        .download(
                            &fetched_version.asset_index.url,
                            Some(&fetched_version.asset_index.sha1),
                        )
                        .await?,
                )?;
                return Ok(());
            }
        }

        Err(anyhow!("Failed to find latest version"))
    }
}
