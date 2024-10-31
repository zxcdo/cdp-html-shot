/*!
[![GitHub]](https://github.com/araea/cdp-html-shot)&ensp;[![crates-io]](https://crates.io/crates/cdp-html-shot)&ensp;[![docs-rs]](crate)

[GitHub]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

<br>

A Rust library for capturing HTML screenshots using CDP (Chrome DevTools Protocol) and Tokio.

- Auto cleanup
- Asynchronous API
- HTML screenshot capture

## Example

### Capture HTML screenshot

```rust
use base64::Engine;
use anyhow::Result;
use cdp_html_shot::{Browser};

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

    let png_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    std::fs::write("test0.png", png_data)?;

    Ok(())
}
```

### Fine control

```rust
use base64::Engine;
use anyhow::Result;
use cdp_html_shot::{Browser};

#[tokio::main]
async fn main() -> Result<()> {
    let browser = Browser::new().await?;
    let tab = browser.new_tab().await?;

    tab.set_content("<h1>Hello world!</h1>").await?;

    let element = tab.find_element("h1").await?;
    let base64 = element.screenshot().await?;
    tab.close().await?;

    let png_data = base64::prelude::BASE64_STANDARD.decode(base64)?;
    std::fs::write("test0.png", png_data)?;

    Ok(())
}
```
*/

mod tab;
mod browser;
mod element;
mod temp_dir;
mod transport;
mod exit_hook;
mod general_utils;
mod transport_actor;

pub use tab::Tab;
pub use element::Element;
pub use browser::Browser;
pub use exit_hook::ExitHook;