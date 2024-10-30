use crate::tab::Tab;
use serde_json::json;
use crate::general_utils;
use anyhow::{Context, Result};
use crate::general_utils::next_id;

/// An element instance.
pub struct Element<'a> {
    parent: &'a Tab,
    // node_id: u64,
    // value: String,
    // tag_name: String,
    backend_node_id: u64,
    // attributes: Vec<String>,
    // remote_object_id: String,
}

impl<'a> Element<'a> {
    pub(crate) async fn new(parent: &'a Tab, node_id: u64) -> Result<Self> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "DOM.describeNode",
            "params": {
                "nodeId": node_id,
                "depth": 100
            }
        }).to_string();

        let res = general_utils::send_and_get_msg(parent.transport.clone(), msg_id, &parent.session_id, msg).await?;

        let msg = general_utils::serde_msg(&res);
        let node = msg["result"]
            .get("node")
            .context("Failed to get node")?;

        // let attributes = node
        //     .get("attributes")
        //     .context("Failed to get attributes")?
        //     .to_string();

        // let attributes: Vec<String> = serde_json::from_str(&attributes)?;

        // let tag_name = node
        //     .get("nodeName")
        //     .context("Failed to get nodeName")?
        //     .as_str()
        //     .context("Failed to convert nodeName to string")?
        //     .to_string();

        let backend_node_id = node
            .get("backendNodeId")
            .context("Failed to get backendNodeId")?
            .as_u64()
            .context("Failed to convert backendNodeId to u64")?;

        // let msg_id = next_id();
        // let msg = json!({
        //     "id": msg_id,
        //     "method": "DOM.resolveNode",
        //     "params": {
        //         "backendNodeId": backend_node_id,
        //     }
        // }).to_string();

        // let res = general_utils::send_and_get_msg(parent.transport.clone(), msg_id, &parent.session_id, msg).await?;

        // let msg = general_utils::serde_msg(&res);
        // let object = msg["result"]
        //     .get("object")
        //     .context("Failed to get an object")?;

        // let value = object
        //     .get("value")
        //     .unwrap_or(&json!(""))
        //     .to_string();

        // let remote_object_id = object
        //     .get("objectId")
        //     .context("Failed to get objectId")?
        //     .as_str()
        //     .context("Failed to convert objectId to string")?
        //     .to_string();

        Ok(Self {
            parent,
            // value,
            // node_id,
            // tag_name,
            // attributes,
            backend_node_id,
            // remote_object_id,
        })
    }

    /**
    Capture a screenshot of the element.

    # Example
    ```no_run
    use cdp_html_shot::Browser;
    use anyhow::Result;

    #[tokio::main]
    async fn main() -> Result<()> {
        let browser = Browser::new().await?;
        let tab = browser.new_tab().await?;
        tab.set_content("<h1>Hello world!</h1>").await?;
        let element = tab.find_element("h1").await?;
        let base64 = element.screenshot().await?;
        tab.close().await?;
        Ok(())
    }
    ```
    */
    pub async fn screenshot(&self) -> Result<String> {
        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "DOM.getBoxModel",
            "params": {
                "backendNodeId": self.backend_node_id
            }
        }).to_string();

        let res = general_utils::send_and_get_msg(self.parent.transport.clone(), msg_id, &self.parent.session_id, msg).await?;

        let msg = general_utils::serde_msg(&res);
        let model = msg["result"]
            .get("model")
            .context("Failed to get model")?;

        let top_left_x = model["border"][0].as_f64().unwrap();
        let top_left_y = model["border"][1].as_f64().unwrap();
        let top_right_x = model["border"][2].as_f64().unwrap();
        let bottom_left_y = model["border"][5].as_f64().unwrap();

        let msg_id = next_id();
        let msg = json!({
            "id": msg_id,
            "method": "Page.captureScreenshot",
            "params": {
                "clip": {
                    "x": top_left_x,
                    "y": top_left_y,
                    "width": top_right_x - top_left_x,
                    "height": bottom_left_y - top_left_y,
                    "scale": 1.0
                },
                "fromSurface": true,
                "captureBeyondViewport": true,
            }
        }).to_string();

        self.parent.activate().await?;
        let res = general_utils::send_and_get_msg(self.parent.transport.clone(), msg_id, &self.parent.session_id, msg).await?;

        let msg = general_utils::serde_msg(&res);
        let base64 = msg["result"]
            .get("data")
            .context("Failed to get data")?
            .as_str()
            .context("Failed to convert data to string")?
            .to_string();

        Ok(base64)
    }
}