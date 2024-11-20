use std::sync::Arc;
use serde_json::json;
use anyhow::{Context, Result};

use crate::general_utils;
use crate::element::Element;
use crate::transport::Transport;
use crate::general_utils::next_id;
use crate::transport_actor::TransportResponse;

/// A tab instance.
pub struct Tab {
    pub(crate) transport: Arc<Transport>,
    pub(crate) session_id: String,
    pub(crate) target_id: String,
}

impl Tab {
    /**
    Create a new tab instance.

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
    pub(crate) async fn new(transport: Arc<Transport>) -> Result<Self> {
        let TransportResponse::Response(res) = transport.send(json!({
            "id": next_id(),
            "method": "Target.createTarget",
            "params": {
                "url": "about:blank"
            }
        })).await? else { panic!() };

        let target_id = res
            .result
            .get("targetId")
            .context("Failed to get targetId")?
            .as_str()
            .unwrap();

        let TransportResponse::Response(res) = transport.send(json!({
            "id": next_id(),
            "method": "Target.attachToTarget",
            "params": {
                "targetId": target_id
            }
        })).await? else { panic!() };

        let session_id = res
            .result["sessionId"]
            .as_str()
            .unwrap();

        Ok(Self {
            transport,
            session_id: String::from(session_id),
            target_id: String::from(target_id),
        })
    }

    /**
    Set the content of the tab.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        tab.set_content("<h1>Hello world!</h1>").await?;
        Ok(())
    }
    ```
    */
    pub async fn set_content(&self, content: &str) -> Result<&Self> {
        let content = match (content.contains('`'), content.contains("${")) {
            (true, true) => &content.replace('`', "${BACKTICK}").replace("${", "$ {"),
            (true, false) => &content.replace('`', "${BACKTICK}"),
            (false, true) => &content.replace("${", "$ {"),
            (false, false) => content,
        };

        let expression = format!(
            r#"
    (async () => {{
        try {{
            const BACKTICK = '`';
            document.open();
            document.write(String.raw`{content}`);
            document.close();

            await Promise.race([
                new Promise((resolve) => {{
                    const checkResources = async () => {{
                        if (document.readyState !== 'complete') {{
                            return false;
                        }}

                        const images = Array.from(document.images);
                        const imagePromises = images.map(img => {{
                            if (img.complete) return Promise.resolve();
                            return new Promise(resolve => {{
                                img.onload = resolve;
                                img.onerror = resolve;
                            }});
                        }});

                        const styleSheets = Array.from(document.styleSheets);
                        const stylePromises = styleSheets.map(sheet => {{
                            if (!sheet.href) return Promise.resolve();
                            return new Promise(resolve => {{
                                const link = document.querySelector(`link[href="${{sheet.href}}"]`);
                                if (link.sheet) resolve();
                                else {{
                                    link.onload = resolve;
                                    link.onerror = resolve;
                                }}
                            }});
                        }});

                        await Promise.all([...imagePromises, ...stylePromises]);

                        return new Promise(resolve => {{
                            requestAnimationFrame(() => {{
                                requestAnimationFrame(resolve);
                            }});
                        }});
                    }};

                    checkResources().then(resolved => {{
                        if (!resolved) {{
                            window.addEventListener('load', () => {{
                                checkResources().then(resolve);
                            }});
                        }} else {{
                            resolve(true);
                        }}
                    }});
                }}),

                new Promise((_, reject) => {{
                    setTimeout(() => reject(new Error('Timeout')), 30000);
                }})
            ]);

            return 'Page loaded successfully';
        }} catch (error) {{
            throw new Error(`Failed to set content: ${{error.message}}`);
        }}
    }})();
    "#
        );

        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Runtime.evaluate",
            "params": {
                "expression": expression,
                "awaitPromise": true,
            }
        }).to_string();

        general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        Ok(self)
    }

    /**
    Find an element by CSS selector.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        let element = tab.find_element("h1").await?;
        Ok(())
    }
    ```
    */
    pub async fn find_element(&self, selector: &str) -> Result<Element> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "DOM.getDocument",
            "params": {}
        }).to_string();

        let res = general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        let msg = general_utils::serde_msg(&res);
        let node_id = msg["result"]["root"]["nodeId"]
            .as_u64()
            .unwrap();

        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "DOM.querySelector",
            "params": {
                "nodeId": node_id,
                "selector": selector
            }
        }).to_string();

        let res = general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        let msg = general_utils::serde_msg(&res);

        let node_id = match msg["result"]["nodeId"].as_u64() {
            Some(node_id) => node_id,
            None => return Err(anyhow::anyhow!("Element not found")),
        };

        Element::new(self, node_id).await
    }

    /**
    Close the tab.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        tab.close().await?;
        Ok(())
    }
    ```
    */
    pub async fn activate(&self) -> Result<&Self> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Target.activateTarget",
            "params": {
                "targetId": self.target_id
            }
        }).to_string();

        general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        Ok(self)
    }

    /**
    Navigate to a URL.

    # Warning

    This API does not wait for the page to load, it is only used to navigate to local HTML files,
    which is convenient for getting font and other resources.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;
    use tokio::time;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        tab.goto("https://www.rust-lang.org/").await?;
        time::sleep(time::Duration::from_secs(5)).await;
        Ok(())
    }
    ```
    */
    pub async fn goto(&self, url: &str) -> Result<&Self> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Page.navigate",
            "params": {
                "url": url
            }
        }).to_string();

        general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        Ok(self)
    }

    /**
    Close the tab.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        tab.close().await?;
        Ok(())
    }
    ```
    */
    pub async fn close(&self) -> Result<()> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Target.closeTarget",
            "params": {
                "targetId": self.target_id
            }
        }).to_string();

        general_utils::send_and_get_msg(self.transport.clone(), msg_id, &self.session_id, msg).await?;

        Ok(())
    }
}