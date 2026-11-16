use futures_util::StreamExt;

pub async fn download(
  client: &reqwest::Client,
  url: &str,
  max_size: usize,
) -> anyhow::Result<Vec<u8>> {
  let res = client.get(url).send().await?.error_for_status()?;

  if let Some(size) = res.content_length() {
    anyhow::ensure!(
      size <= max_size as u64,
      "File exceeds maximum allowed size."
    );
  };

  let mut stream = res.bytes_stream();
  let mut bytes = Vec::new();

  while let Some(chunk) = stream.next().await {
    let chunk = chunk?;

    anyhow::ensure!(
      bytes.len().saturating_add(chunk.len()) <= max_size,
      "File exceeds maximum allowed size."
    );

    bytes.extend_from_slice(&chunk);
  }

  anyhow::ensure!(bytes.is_empty() == false, "File is empty.");

  Ok(bytes)
}
