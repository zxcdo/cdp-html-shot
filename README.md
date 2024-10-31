# cdp-html-shot

[<img alt="github" src="https://img.shields.io/badge/github-araea/cdp_html_shot-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/araea/cdp-html-shot)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cdp_html_shot.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cdp-html-shot)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-cdp_html_shot-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/cdp-html-shot)

A Rust library for capturing HTML screenshots using CDP.

- Auto cleanup
- HTML screenshot captures
- Asynchronous API (Tokio)

## Usage

```toml
[dependencies]
cdp-html-shot = "0.1"
```

## Example

### Capture HTML screenshot

```rust
use base64::Engine;
use anyhow::Result;
use cdp_html_shot::Browser;

#[tokio::main]
async fn main() -> Result<()> {
    const HTML: &str = r#"
        <html lang="en-US">
        <body>
        <h1>My test page</h1>
        <p>Hello, Rust!</p>
        </body>
        </html>
    "#;
    
    let browser = Browser::new().await?;
    let base64 = browser.capture_html(HTML, "html").await?;

    let img_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    std::fs::write("test0.jpeg", img_data)?;

    Ok(())
}
```

### Fine control

```rust
use base64::Engine;
use anyhow::Result;
use cdp_html_shot::Browser;

#[tokio::main]
async fn main() -> Result<()> {
    let browser = Browser::new().await?;
    let tab = browser.new_tab().await?;

    tab.set_content("<h1>Hello world!</h1>").await?;

    let element = tab.find_element("h1").await?;
    let base64 = element.screenshot().await?;
    tab.close().await?;

    let img_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    std::fs::write("test0.jpeg", img_data)?;

    Ok(())
}
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

