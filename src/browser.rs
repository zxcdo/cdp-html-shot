mod temp_dir;
mod browser_utils;
mod browser_config;
mod browser_builder;

use log::error;
use std::sync::Arc;
use std::process::Child;
use tokio::sync::OnceCell;
use anyhow::{Context, Result};
use browser_config::BrowserConfig;

use crate::tab::Tab;
use crate::CaptureOptions;
use temp_dir::CustomTempDir;
use crate::transport::Transport;
use crate::browser::browser_builder::BrowserBuilder;

/// The global browser instance.
static mut BROWSER: OnceCell<Arc<Browser>> = OnceCell::const_new();

#[derive(Debug)]
struct Process(pub Child, pub CustomTempDir);

/// A browser instance.
#[derive(Debug)]
pub struct Browser {
    transport: Arc<Transport>,
    process: Process,
    is_closed: bool,
}

impl Browser {
    /**
    Create a new browser instance with default configuration (headless).

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
        BrowserBuilder::new().build().await
    }

    /// Create a new browser instance with a visible window.
    pub async fn new_with_head() -> Result<Self> {
        BrowserBuilder::new()
            .headless(false)
            .build()
            .await
    }

    /// Create browser instance with custom configuration
    async fn create_browser(config: BrowserConfig) -> Result<Self> {
        let mut child = browser_utils::spawn_chrome_process(&config)?;
        let ws_url = browser_utils::get_websocket_url(
            child.stderr.take().context("Failed to get stderr")?
        ).await?;

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
    Basic version: Capture a screenshot of an HTML element

    # Arguments
    - `html`: The HTML content
    - `selector`: The CSS selector of the element to capture
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
    Advanced version: Capture a screenshot of an HTML element with additional options

    # Arguments
    - `html`: The HTML content
    - `selector`: The CSS selector of the element to capture
    - `options`: Configuration options for the capture

    # Example
    ```no_run
    use cdp_html_shot::{Browser, CaptureOptions};
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let options = CaptureOptions::new()
            .with_raw_png(true);

        let base64 = browser
            .capture_html_with_options(
                "<h1>Hello world!</h1>",
                "h1",
                options
            ).await?;
        Ok(())
    }
    ```
    */
    pub async fn capture_html_with_options(
        &self,
        html: &str,
        selector: &str,
        options: CaptureOptions,
    ) -> Result<String> {
        let tab = self.new_tab().await?;

        tab.set_content(html).await?;

        let element = tab.find_element(selector).await?;

        let base64 = if options.raw_png {
            element.raw_screenshot().await?
        } else {
            element.screenshot().await?
        };

        tab.close().await?;

        Ok(base64)
    }

    /**
    Close the browser.

    This will kill the browser process and clean up temporary files.

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
            .and_then(|_| self.process.0.wait())
            .context("Failed to kill the browser process")?;

        self.process.1
            .cleanup()?;

        self.is_closed = true;
        Ok(())
    }
}

impl Browser {
    /**
    Get the global Browser instance.

    Creates a new one if it doesn't exist.

    This method is thread-safe and ensures only one browser instance is created.

    The browser will be automatically closed
    when all references are dropped or when the program exits.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::instance().await;
        let tab = browser.new_tab().await?;

        Browser::close_instance();
        Ok(())
    }
    ```
    */
    pub async fn instance() -> Arc<Browser> {
        unsafe {
            let browser = BROWSER
                .get_or_init(|| async {
                    let browser = Browser::new().await.unwrap();
                    Arc::new(browser)
                })
                .await;

            browser.clone()
        }
    }

    /**
    Close the global Browser instance.

    Please ensure that this method is called before the program exits,
    and there should be no Browser instances in use at this time.
    */
    pub fn close_instance() {
        unsafe {
            BROWSER.take();
        }
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        if !self.is_closed {
            if let Err(e) = self.close() {
                error!("Error closing browser: {:?}", e);
            }
        }
    }
}
