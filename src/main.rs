use scraper::{Html, Selector};
use std::io::Cursor;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fetch_file(url: String, file_name: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn get_urls(url: String) -> Result<(String, String)> {
    let response = reqwest::get(url).await?;
    let fragment = Html::parse_fragment(&response.text().await.unwrap());
    let selector = Selector::parse("source").unwrap();

    let video_item = fragment.select(&selector).next().unwrap();
    let video_url = video_item.value().attr("src").unwrap();

    let head_selector = Selector::parse("h1").unwrap();
    let title = fragment.select(&head_selector).next().unwrap().inner_html();

    Ok((video_url.to_string(), title))
}

#[tokio::main]
async fn main() {
    let pattern = std::env::args().nth(1).expect("no pattern given");
    if pattern.to_string().starts_with("https://vk.com/video") {
        let (url, title) = get_urls(pattern).await.unwrap();
        println!("{} downloaded", title);
        fetch_file(url, title.to_string()).await.unwrap();
    } else {
        println!("wrong url")
    }
}
