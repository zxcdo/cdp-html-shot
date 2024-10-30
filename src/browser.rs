mod browser_utils;
mod browser_config;

use std::sync::Arc;
use crate::tab::Tab;
use std::process::Child;
use anyhow::{Context, Result};
use crate::transport::Transport;
use browser_config::BrowserConfig;
use crate::temp_dir::CustomTempDir;

#[derive(Debug)]
struct Process(pub Child, pub CustomTempDir);

/// A browser instance.
pub struct Browser {
    transport: Arc<Transport>,
    process: Process,
    is_closed: bool,
}

impl Browser {
    /**
    Create a new browser instance.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        Ok(())
    }
    ```
    */
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::new()?;
        let mut child = browser_utils::spawn_chrome_process(&config)?;
        let ws_url = browser_utils::get_websocket_url(child.stderr.take().context("Failed to get stderr")?).await?;

        Ok(Self {
            transport: Arc::new(Transport::new(&ws_url).await?),
            process: Process(child, config.temp_dir),
            is_closed: false,
        })
    }

    /**
    Create a new tab.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        Ok(())
    }
    ```
    */
    pub async fn new_tab(&self) -> Result<Tab> {
        Tab::new(self.transport.clone()).await
    }

    /**
    Capture a screenshot of an HTML element.

    # Arguments
    - `html`: The HTML content.
    - `selector`: The CSS selector of the element to capture.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let base64 = browser.capture_html("<h1>Hello world!</h1>", "h1").await?;
        Ok(())
    }
    ```
    */
    pub async fn capture_html(&self, html: &str, selector: &str) -> Result<String> {
        let tab = self.new_tab().await?;
        tab.set_content(html).await?;
        let element = tab.find_element(selector).await?;
        let base64 = element.screenshot().await?;
        tab.close().await?;
        Ok(base64)
    }

    /**
    Close the browser.

    This will kill the browser process and cleanup temporary files.

    Normally, this method does not need to be called manually.

    Because it will be called automatically when the `Browser` instance is destroyed.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let mut browser = Browser::new().await?;
        browser.close()?;
        Ok(())
    }
    ```
    */
    pub fn close(&mut self) -> Result<()> {
        if self.is_closed {
            return Ok(());
        }

        Arc::get_mut(&mut self.transport)
            .unwrap()
            .shutdown();

        self.process.0
            .kill()
            .context("Failed to kill the browser process")?;

        self.process.1
            .cleanup()?;

        self.is_closed = true;
        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        if !self.is_closed {
            if let Err(e) = self.close() {
                eprintln!("Error closing browser: {:?}", e);
            }
        }
    }
}
