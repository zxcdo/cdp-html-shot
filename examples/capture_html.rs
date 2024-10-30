use base64::Engine;
use anyhow::Result;
use cdp_html_shot::{Browser};

#[tokio::main]
async fn main() -> Result<()> {
    let browser = Browser::new().await?;

    const HTML: &str = r#"
        <html lang="en-US">
        <body>
        <h1>My test page</h1>
        <p>Hello, Rust!</p>
        </body>
        </html>
    "#;

    let base64 = browser.capture_html(HTML, "html").await?;

    let png_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    std::fs::write("test0.png", png_data)?;

    Ok(())
}