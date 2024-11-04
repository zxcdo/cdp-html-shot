use anyhow::Result;
use base64::Engine;
use cdp_html_shot::Browser;
use shindan_maker::{ShindanClient, ShindanDomain};
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    const SHINDAN_ID: &str = "384482";
    const USER_NAME: &str = "NAWYJX";

    let client = ShindanClient::new(ShindanDomain::En)?;
    let (html_str, title) = client
        .get_html_str_with_title(SHINDAN_ID, USER_NAME)
        .await?;

    println!("Result title: {}", title);

    let browser = Browser::new().await?;

    let base64 = browser.capture_html(&html_str, "#title_and_result").await?;

    let img_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    fs::write("test0.jpeg", img_data)?;

    Ok(())
}
