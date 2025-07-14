pub async fn get_latest_version() -> String {
    let body = reqwest::get("https://www.rust-lang.org")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    body
}
