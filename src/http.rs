use anyhow::Result;
use http::{Uri};
use serde::de::DeserializeOwned;

pub async fn fetch_json<T>(url: &Uri) -> Result<T>
where
    T: DeserializeOwned,
{
    let reqwest_url = url.to_string();
    let result = reqwest::get(reqwest_url)
        .await?
        .json::<T>()
        .await?;

    Ok(result)
}

pub async fn fetch_bytes(url: &Uri) -> Result<Vec<u8>> {
    let reqwest_url = url.to_string();
    let bytes = reqwest::get(reqwest_url)
        .await?
        .bytes()
        .await?
        .to_vec();
    Ok(bytes.to_vec())
}
