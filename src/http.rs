use anyhow::Result;
use http::Uri;
use hyper::body::Buf;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;

pub async fn fetch_json<T>(url: &Uri) -> Result<T>
where
    T: DeserializeOwned,
{
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    // Fetch the url...
    let res = client.get(url.clone()).await?;

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(res).await?;

    // try to parse as json with serde_json
    let reader = body.reader();
    let result = serde_json::from_reader(reader)?;

    Ok(result)
}

pub async fn fetch_bytes(url: &Uri) -> Result<Vec<u8>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    // Fetch the url...
    let res = client.get(url.clone()).await?;

    // return the body as bytes
    let bytes = hyper::body::to_bytes(res.into_body()).await?;
    Ok(bytes.to_vec())
}
