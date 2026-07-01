use arboard::Clipboard;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 剪贴板服务（纯本地操作，不同步到服务端）
/// 封装 arboard 库，提供线程安全的剪贴板读写
pub struct ClipboardService {
    inner: Arc<Mutex<Clipboard>>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                Clipboard::new().expect("无法初始化系统剪贴板")
            )),
        }
    }

    /// 从系统剪贴板读取文本
    pub async fn get_text(&self) -> Result<String, String> {
        let mut clipboard = self.inner.lock().await;
        clipboard.get_text().map_err(|e| e.to_string())
    }

    /// 写入文本到系统剪贴板
    pub async fn set_text(&self, text: &str) -> Result<(), String> {
        let mut clipboard = self.inner.lock().await;
        clipboard.set_text(text).map_err(|e| e.to_string())
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}
