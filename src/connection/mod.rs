pub mod transport;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct RefInfo {
    pub name: String,
    pub hash: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PackfileResponse {
    pub data: Vec<u8>,
}


pub struct VoxTransport {
    client: Client,
    base_url: Url,
}


impl VoxTransport {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: Url::parse(base_url)?,
        })
    }


    pub async fn list_refs(&self) -> Result<Vec<RefInfo>> {
        let url = self.base_url.join("/api/v1/refs")?;
        
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch refs: {}", response.status()));            
        }
        Ok(response.json().await?)
    }

    
    pub async fn fetch_packfile(&self, want: &[String]) -> Result<Vec<u8>> {
        let url = self.base_url.join("/api/v1/packfile")?;

        let response = self.client
            .post(url)
            .json(want)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch packfile: {}", response.status()));
        }

        let pack: PackfileResponse = response.json().await?;
        
        Ok(pack.data)
    }
}
