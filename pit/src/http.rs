use bytes::Bytes;
use reqwest::{Client, Response};

#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("{url} did not match expected hash of {expected}")]
    UnexpectedHash { url: String, expected: String },
}

pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    // TODO: Proper client
    pub fn new() -> Self {
        Self {
            inner: Client::new(),
        }
    }

    pub async fn send(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.inner
            .get(url)
            .send()
            .await
            .and_then(Response::error_for_status)
    }

    pub async fn download(&self, url: &str, sha1: Option<&str>) -> Result<Bytes, DownloadError> {
        let mut response = self.send(url).await?;

        let mut data = Vec::new();
        let mut digest = sha1.map(|_| sha1_smol::Sha1::new());

        while let Some(ref item) = response.chunk().await? {
            data.extend(item);
            if let Some(digest) = digest.as_mut() {
                digest.update(item);
            }
        }

        if let Some(expected) = sha1
            && digest.as_ref().unwrap().hexdigest() != expected
        {
            Err(DownloadError::UnexpectedHash {
                url: url.to_string(),
                expected: expected.to_string(),
            })
        } else {
            Ok(Bytes::from(data))
        }
    }
}
