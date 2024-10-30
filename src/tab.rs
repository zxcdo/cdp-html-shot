use std::sync::Arc;
use serde_json::json;
use crate::general_utils;
use crate::element::Element;
use anyhow::{Context, Result};
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
        let expression = format!(r#"
    (async () => {{
        document.open();
        document.write(String.raw`{}`);
        document.close();

        return new Promise((resolve, reject) => {{
            let checkLoadInterval = setInterval(() => {{
                if (document.readyState === 'complete') {{
                    clearInterval(checkLoadInterval);
                    resolve('Page loaded successfully');
                }}
            }}, 10);

            setTimeout(() => {{
                clearInterval(checkLoadInterval);
                reject('Timeout reached while waiting for page to load');
            }}, 30000);
        }});
    }})();"#, content);

        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Runtime.evaluate",
            "params": {
                "expression": expression,
                "awaitPromise": true
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
        let node_id = msg["result"]["nodeId"]
            .as_u64()
            .unwrap();

        if node_id == 0 {
            return Err(anyhow::anyhow!("Element not found"));
        }

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