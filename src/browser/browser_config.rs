use std::net;
use which::which;
use winreg::RegKey;
use std::path::{Path, PathBuf};
use rand::prelude::SliceRandom;
use crate::temp_dir::CustomTempDir;
use winreg::enums::HKEY_LOCAL_MACHINE;
use anyhow::{anyhow, Context, Result};

static DEFAULT_ARGS: [&str; 37] = [
    // System Settings
    "--no-sandbox",
    "--no-first-run",
    "--no-default-browser-check",
    "--no-experiments",
    "--no-pings",

    // Memory Optimization
    "--js-flags=--max-old-space-size=8192",  // Set JS heap to 8GB
    "--disk-cache-size=67108864",            // 64MB cache
    "--memory-pressure-off",
    "--aggressive-cache-discard",
    "--disable-dev-shm-usage",

    // Process Management
    "--process-per-site",
    "--disable-hang-monitor",
    "--disable-renderer-backgrounding",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",

    // Disable Optional Features
    "--disable-sync",
    "--disable-breakpad",
    "--disable-infobars",
    "--disable-extensions",
    "--disable-default-apps",
    "--disable-notifications",
    "--disable-popup-blocking",
    "--disable-prompt-on-repost",
    "--disable-client-side-phishing-detection",

    // Network Settings
    "--enable-async-dns",
    "--enable-parallel-downloading",
    "--ignore-certificate-errors",
    "--disable-http-cache",

    // Graphics Settings
    "--disable-gpu",
    "--use-gl=swiftshader",            // Use software rendering
    "--disable-gpu-compositing",
    "--force-color-profile=srgb",
    "--disable-software-rasterizer",

    // Feature Flags
    "--disable-features=TranslateUI,BlinkGenPropertyTrees,AudioServiceOutOfProcess",
    "--enable-features=NetworkService,NetworkServiceInProcess,CalculateNativeWinOcclusion",

    // Performance
    "--disable-ipc-flooding-protection",
    "--no-zygote",

    // Logging
    // "--enable-logging=stderr"
];

pub(crate) struct BrowserConfig {
    debug_port: u16,
    pub(crate) headless: bool,
    pub(crate) temp_dir: CustomTempDir,
    pub(crate) executable_path: PathBuf,
}

impl BrowserConfig {
    pub(crate) fn new() -> Result<Self> {
        let temp_dir = std::env::current_dir()?.join("temp");

        Ok(Self {
            headless: true,
            executable_path: default_executable()?,
            debug_port: get_available_port().context("Failed to get available port")?,
            temp_dir: CustomTempDir::new(temp_dir, "cdp-html-shot")
                .context("Failed to create custom temporary directory")?,
        })
    }

    pub(crate) fn get_browser_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("--remote-debugging-port={}", self.debug_port),
            format!("--user-data-dir={}", self.temp_dir.path().display()),
        ];

        args.extend(DEFAULT_ARGS.iter().map(|s| s.to_string()));
        if !self.headless {
            args.push("--auto-open-devtools-for-tabs".to_string());
        } else {
            args.push("--headless".to_string());
        }

        args
    }
}

fn default_executable() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CHROME") {
        if Path::new(&path).exists() {
            return Ok(path.into());
        }
    }

    let apps = [
        "google-chrome-stable",
        "google-chrome-beta",
        "google-chrome-dev",
        "google-chrome-unstable",
        "chromium",
        "chromium-browser",
        "microsoft-edge-stable",
        "microsoft-edge-beta",
        "microsoft-edge-dev",
        "chrome",
        "chrome-browser",
        "msedge",
        "microsoft-edge",
    ];
    for app in apps {
        if let Ok(path) = which(app) {
            return Ok(path);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let macos_apps = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Google Chrome Beta.app/Contents/MacOS/Google Chrome Beta",
            "/Applications/Google Chrome Dev.app/Contents/MacOS/Google Chrome Dev",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "/Applications/Microsoft Edge Beta.app/Contents/MacOS/Microsoft Edge Beta",
            "/Applications/Microsoft Edge Dev.app/Contents/MacOS/Microsoft Edge Dev",
            "/Applications/Microsoft Edge Canary.app/Contents/MacOS/Microsoft Edge Canary",
        ];
        for path in macos_apps.iter() {
            let path = Path::new(path);
            if path.exists() {
                return Ok(path.into());
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(path) = get_chrome_path_from_registry().filter(|p| p.exists()) {
            return Ok(path);
        }

        let windows_apps = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        ];
        for path in windows_apps.iter() {
            let path = Path::new(path);
            if path.exists() {
                return Ok(path.into());
            }
        }
    }

    Err(anyhow!("Could not auto detect a chrome executable"))
}

#[cfg(windows)]
fn get_chrome_path_from_registry() -> Option<PathBuf> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe")
        .and_then(|key| key.get_value::<String, _>(""))
        .map(PathBuf::from)
        .ok()
}

fn get_available_port() -> Option<u16> {
    let mut ports: Vec<u16> = (8000..9000).collect();
    ports.shuffle(&mut rand::thread_rng());
    ports.iter().find(|port| port_is_available(**port)).copied()
}

fn port_is_available(port: u16) -> bool {
    net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}