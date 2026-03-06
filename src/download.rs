use std::time::Duration;

use reqwest::get;
use tokio::{io::AsyncWriteExt, time::sleep};
use url::Url;

fn filename_from_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    parsed.path_segments()?.last().map(|s| s.to_string())
}

pub async fn download_package_with_retry(
    url: &str,
    outputdir: &str,
    max_retry: u16,
) -> Result<(), String> {
    let mut retry = 0;

    let mut delay = Duration::from_millis(500);

    loop {
        match download_package(url, outputdir).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                retry += 1;

                if retry >= max_retry {
                    return Err(format!("Echec après {} tentatives: {}", max_retry, e).into());
                }

                eprintln!(
                    "Tentative {} échouée pour {}: {}. Réussai dans {:?}...",
                    retry, url, e, delay
                );
                sleep(delay).await;
                delay = Duration::from_millis(delay.as_millis() as u64 * 2);
            }
        }
    }
}
async fn download_package(url: &str, outputdir: &str) -> Result<(), String> {
    let filename = filename_from_url(url).ok_or("Impossible d'extraire le nom du fichier")?;

    let response = get(url).await.map_err(|e| e.to_string())?;

    let path = format!("{}/{}", outputdir, &filename);

    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;

    Ok(())
}
