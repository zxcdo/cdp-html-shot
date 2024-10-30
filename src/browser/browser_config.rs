use std::net;
use which::which;
use winreg::RegKey;
use std::path::{Path, PathBuf};
use rand::prelude::SliceRandom;
use crate::temp_dir::CustomTempDir;
use winreg::enums::HKEY_LOCAL_MACHINE;
use anyhow::{anyhow, Context, Result};

static DEFAULT_ARGS: [&str; 37] = [
    // 1. 基础系统设置
    "--no-sandbox",                     // 禁用沙盒模式，提高性能
    "--no-first-run",                   // 跳过首次运行检查
    "--no-default-browser-check",       // 跳过默认浏览器检查
    "--no-experiments",                 // 禁用实验性功能
    "--no-pings",                       // 禁用页面 ping

    // 2. 内存和缓存优化
    "--js-flags=--max-old-space-size=8192",  // 增加 JS 堆大小
    "--disk-cache-size=67108864",      // 64MB 缓存大小
    "--memory-pressure-off",            // 禁用内存压力检测
    "--aggressive-cache-discard",       // 激进的缓存丢弃策略
    "--disable-dev-shm-usage",          // 禁用 /dev/shm 使用

    // 3. 进程和线程优化
    "--process-per-site",               // 每个站点使用单独进程
    "--disable-hang-monitor",           // 禁用挂起监视器
    "--disable-renderer-backgrounding", // 禁用渲染器后台处理
    "--disable-background-timer-throttling", // 禁用后台计时器限制
    "--disable-backgrounding-occluded-windows", // 禁用遮挡窗口后台处理

    // 4. 禁用非必要功能
    "--disable-sync",                   // 禁用同步
    "--disable-breakpad",               // 禁用崩溃报告
    "--disable-infobars",               // 禁用信息栏
    "--disable-extensions",             // 禁用扩展
    "--disable-default-apps",           // 禁用默认应用
    "--disable-notifications",          // 禁用通知
    "--disable-popup-blocking",         // 禁用弹窗阻止
    "--disable-prompt-on-repost",       // 禁用重新提交提示
    "--disable-client-side-phishing-detection", // 禁用钓鱼检测

    // 5. 网络优化
    "--enable-async-dns",               // 启用异步 DNS
    "--enable-parallel-downloading",     // 启用并行下载
    "--ignore-certificate-errors",      // 忽略证书错误
    "--disable-http-cache",             // 禁用 HTTP 缓存以提高速度

    // 6. 图形和渲染优化
    "--force-color-profile=srgb",       // 强制使用 sRGB 颜色配置
    "--disable-gpu",                    // 禁用 GPU 加速
    "--disable-gpu-compositing",        // 禁用 GPU 合成
    "--use-gl=swiftshader",            // 使用软件渲染

    // 7. 功能开关
    "--disable-features=TranslateUI,BlinkGenPropertyTrees,AudioServiceOutOfProcess",
    "--enable-features=NetworkService,NetworkServiceInProcess,CalculateNativeWinOcclusion",

    // 8. 性能相关
    "--disable-ipc-flooding-protection", // 禁用 IPC 洪水保护
    "--no-zygote",                      // 禁用 zygote 进程

    // 9. 调试和日志
    "--enable-logging=stderr"           // 启用 stderr 日志
];

pub(crate) struct BrowserConfig {
    debug_port: u16,
    pub(crate) temp_dir: CustomTempDir,
    pub(crate) executable_path: PathBuf,
}

impl BrowserConfig {
    pub(crate) fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;

        Ok(Self {
            executable_path: default_executable()?,
            debug_port: get_available_port().context("Failed to get available port")?,
            temp_dir: CustomTempDir::new(current_dir, "cdp-html-shot")
                .context("Failed to create custom temporary directory")?,
        })
    }

    pub(crate) fn get_browser_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("--remote-debugging-port={}", self.debug_port),
            format!("--user-data-dir={}", self.temp_dir.path().display()),
        ];

        args.extend(DEFAULT_ARGS.iter().map(|s| s.to_string()));
        args.push("--headless".to_string());

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