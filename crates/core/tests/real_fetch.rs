use webmagic_core::{DefaultDownloader, DefaultDownloaderConfig, Downloader, Request};

#[tokio::test]
async fn real_fetch_fifa_homepage_over_https() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .expect("default downloader should build");

    let page = downloader
        .download(Request::get("https://www.fifa.com/"))
        .await
        .expect("default downloader should fetch fifa homepage");

    assert_eq!(page.request.url, "https://www.fifa.com/");
    assert!((200..400).contains(&page.status_code));
    assert!(page.final_url.starts_with("https://"));
    assert!(!page.body.is_empty());

    let body = String::from_utf8_lossy(&page.body).to_lowercase();
    assert!(body.contains("fifa"));
}
