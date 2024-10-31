use std::fs;
use anyhow::Result;
use base64::Engine;
use tokio::try_join;
use futures::future::try_join_all;
use cdp_html_shot::{Browser, Tab};

async fn take_screenshot(tab: Tab, filename: &str) -> Result<()> {
    tab.set_content(HTML).await?;
    let element = tab.find_element("#title_and_result").await?;
    let base64 = element.screenshot().await?;
    tab.close().await?;
    let png_data = base64::prelude::BASE64_STANDARD.decode(base64)?;

    let dir = std::env::current_dir()?.join("cache");
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(filename), png_data)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let browser = Browser::new().await?;

    let (tab1, tab2, tab3, tab4, tab5, tab6) = try_join!(
        browser.new_tab(),
        browser.new_tab(),
        browser.new_tab(),
        browser.new_tab(),
        browser.new_tab(),
        browser.new_tab(),
    )?;

    let screenshot_tasks = vec![
        take_screenshot(tab1, "test1.png"),
        take_screenshot(tab2, "test2.png"),
        take_screenshot(tab3, "test3.png"),
        take_screenshot(tab4, "test4.png"),
        take_screenshot(tab5, "test5.png"),
        take_screenshot(tab6, "test6.png"),
    ];

    try_join_all(screenshot_tasks).await?;
    Ok(())
}

const HTML: &str = r#"<!DOCTYPE html>

<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width">
    <link rel="stylesheet" href="https://cn.shindanmaker.com/css/app.css?id=cbfb28ec9001aee269676b04e227a3b9">
    <style>
        :root {
            --body-bg: #ffffff;
            --text-body: #212529;
            --bg-img-line: #ffffff;
            --bg-img-fill: #ffffff;
            --main-blue: #00c5ff;
        }

        html {
            box-sizing: border-box;
            font-family: sans-serif;
            line-height: 1.15;
            -webkit-tap-highlight-color: transparent;
            max-width: 750px;
        }

        *, *::before, *::after {
            box-sizing: inherit;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            font-size: 0.9rem;
            font-weight: 400;
            line-height: 1.6;
            color: var(--text-body);
            background-color: var(--body-bg);
            background-image: repeating-linear-gradient(90deg, var(--bg-img-line) 1px, transparent 1px, transparent 15px),
            repeating-linear-gradient(0deg, var(--bg-img-line) 1px, var(--bg-img-fill) 1px, var(--bg-img-fill) 15px);
            background-size: 15px 15px;
            margin: 0;
            text-align: left;
            overflow-wrap: break-word;
            max-width: 750px;
            height: 100%;
        }

        #main-container {
            max-width: 750px;
        }

        #main {
            min-height: 500px;
        }

        #title_and_result {
            width: 100%;
            margin-bottom: 1rem;
            border: 1rem solid var(--main-blue);
            font-size: 1.9rem;
        }

        #shindanResultAbove {
            padding: 1.5rem;
            text-align: center;
            font-weight: 700;
            font-size: 1.1em;
            line-height: 1.2;
        }

        #shindanResultAbove span {
            display: inline-block;
            text-align: left;
        }

        #shindanResultAbove a {
            font-weight: 700;
            text-decoration: none;
            color: var(--text-body);
        }


        #shindanResultTitle {
            display: block;
            overflow: hidden;
            padding: 1.5rem 0.5rem;
            white-space: nowrap;
            text-align: center;
            font-weight: 700;
            background-color: var(--main-blue);
            color: #fff;
            line-height: 1.1em;
            font-size: 0.9em;
        }

        #shindanResultContainer {
            font-size: 1em;
        }

        #shindanResultHeight {
            display: flex;
            min-height: 200px;
            width: 100%;
            align-items: center;
        }

        #shindanResultCell {
            width: 100%;
        }

        #shindanResultContent {
            display: block;
            padding: 1.5rem;
            text-align: center;
            word-break: break-word;
        }

        #shindanResult {
            display: inline-block;
            text-align: left;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            line-height: 1.33;
            white-space: break-spaces;
        }

        #shindanResult span {
            font-weight: 700;
        }

        #title_and_result > .shindanTitleImageContainer {
            font-family: -apple-system, BlinkMacSystemFont, "Helvetica Neue", Arial, "Hiragino Kaku Gothic ProN", "Hiragino Sans", Meiryo, sans-serif;
            color: var(--text-body);
            font-size: 1.9rem;
            aspect-ratio: 40/21;
            width: 100%;
        }

        #title_and_result > .shindanTitleImageContainer > a {
            font-weight: 700;
            color: #fff !important;
            text-decoration: none !important;
        }

        #title_and_result > .shindanTitleImageContainer > a > img {
            width: 100%;
            height: auto;
            display: block;
            max-width: 960px;
        }
    </style>
    <!-- SCRIPTS -->
    <title>ShindanMaker</title>
</head>

<body>
<div id="main-container">
    <div id="main">
        <div id="title_and_result" class="mb-3"> <div class="shindanTitleImageContainer"> <a class="text-white text-decoration-none" href="https://en.shindanmaker.com/1222992"><script style="display:none" type="text/javascript">
//<![CDATA[
window.__mirage2 = {petok:"wBMAbLzqtFkjHaXCkXwyzzaFyN4uX30DDjryDW4zBFI-1800-0.0.1.1"};
//]]>
</script>
<script type="text/javascript" src="https://ajax.cloudflare.com/cdn-cgi/scripts/04b3eb47/cloudflare-static/mirage2.min.js"></script>
<img style="display:none;visibility:hidden;" height="504" width="960" data-cfsrc="https://pic.shindanmaker.com/shindantitle/1222992/img/4794e1bce8123c8d2ff471a83dd9f864faeecf73_head.jpg?v=29b73e9b4f83a70299f4ef6a27bea9894310566a" class="img-fluid" alt="Reincarnation."><noscript>&lt;img src="https://pic.shindanmaker.com/shindantitle/1222992/img/4794e1bce8123c8d2ff471a83dd9f864faeecf73_head.jpg?v=29b73e9b4f83a70299f4ef6a27bea9894310566a" alt="Reincarnation." class="img-fluid" width="960" height="504"&gt;</noscript></a> </div> <div id="shindanResultBlock" data-shindan_title_image="https://pic.shindanmaker.com/shindantitle/1222992/img/4794e1bce8123c8d2ff471a83dd9f864faeecf73_head.jpg?v=29b73e9b4f83a70299f4ef6a27bea9894310566a" class="mx-0" name="shindanResultBlock"> <span class="d-block text-center text-nowrap overflow-hidden font-weight-bold px-2 mb-0" id="shindanResultTitle"> <span id="shindanResultTitleText" class="d-block py-3 py-sm-4"> Diagnosis results </span> </span> <span class="d-block" id="shindanResultContainer"> <span id="shindanResultHeight"> <span id="shindanResultCell"> <span class="d-block py-4 px-3 px-sm-4 text-break text-center " id="shindanResultContent"> <span id="shindanResult" class="text-left d-inline-block"><span style="font-family:'Righteous';font-size:1em;line-height:1.4em;color:#202020 !important;"><span class="shindanResult_name">test_user</span>'s Character Stats</span><br><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#4d4d4d !important;">Class Type:</span>&nbsp;<span style="color:#e5989b !important;font-weight:bold;">Guardian</span><br><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Strength:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">B</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Agility:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">SS</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Speed:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">Heroic</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Stamina:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">E</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Defence:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">B</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Magic Power:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">C</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Aspect Control:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">SS</span><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#202020 !important;">Luck:</span>&nbsp;<span style="color:#ffd100 !important;font-weight:bold;">SSS</span><br><br><span style="font-family:'Bungee';font-size:0.8em;line-height:1.4em;color:#4d4d4d !important;">Aspect:</span>&nbsp;<span style="color:#e5989b !important;font-weight:bold;">Nature/Infinity</span></span> </span> </span> </span> </span> </div></div>
    </div>
</div>
</body>
</html>"#;