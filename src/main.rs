use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use scraper::{Html, Selector};
use std::cmp::min;
use std::io::Write;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fetch_file(u: String, f: String) -> Result<()> {
    let url = &u[..];
    let file_name = &f[..];
    let response = reqwest::get(url).await?;
    let total_size = response
        .content_length()
        .ok_or(format!("Failed to get content length from "))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", file_name));

    let mut file = std::fs::File::create(file_name)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {}", file_name));

    Ok(())
}

async fn get_urls(url: String) -> Result<(String, String)> {
    let response = reqwest::get(url).await?;
    let fragment = Html::parse_fragment(&response.text().await.unwrap());
    let selector = Selector::parse("source").unwrap();

    let video_item = fragment.select(&selector).next().unwrap();
    let video_url = video_item.value().attr("src").unwrap();

    let head_selector = Selector::parse("h1").unwrap();
    let title = match fragment.select(&head_selector).next() {
        None => "some_title".to_string(),
        _ => fragment.select(&head_selector).next().unwrap().inner_html(),
    };
    //let title = fragment.select(&head_selector).next().unwrap().inner_html();

    Ok((video_url.to_string(), title))
}

#[tokio::main]
async fn main() {
    let pattern = std::env::args().nth(1).expect("no pattern given");

    if pattern.to_string().starts_with("https://vk.com/video") {
        let (url, title) = get_urls(pattern).await.unwrap();
        fetch_file(url, title.to_string() + ".mp4").await.unwrap();
    } else {
        println!("wrong url")
    }
}
