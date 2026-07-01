use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use base64::Engine;

const MAX_IMAGE_DIMENSION: usize = 4096;

pub struct ClipboardMonitor {
    running: Arc<AtomicBool>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self { running: Arc::new(AtomicBool::new(false)) }
    }

    pub fn start(&self, db: sqlx::SqlitePool) {
        if self.running.load(Ordering::SeqCst) { return; }
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        std::thread::spawn(move || {
            let rt = match Runtime::new() {
                Ok(r) => r,
                Err(e) => { eprintln!("[cb_mon] Runtime: {}", e); return; }
            };

            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => { eprintln!("[cb_mon] 剪贴板: {}", e); return; }
            };

            let mut last_hash: Option<u64> = None;
            println!("[cb_mon] 已启动");

            while running.load(Ordering::SeqCst) {
                let item = try_read_clipboard(&mut clipboard);

                if let Some((ct, c)) = item {
                    let hash = simple_hash(&c);
                    let is_new = last_hash.map(|h| h != hash).unwrap_or(true);

                    if is_new {
                        last_hash = Some(hash);
                        let ct_owned = ct.to_string();
                        let c_clone = c;

                        rt.block_on(async {
                            // 去重检测
                            let existing: Option<(String, bool)> = sqlx::query_as(
                                "SELECT id, is_favorite FROM clipboard_history \
                                 WHERE content = ?1 AND content_type = ?2 LIMIT 1"
                            )
                            .bind(&c_clone).bind(&ct_owned)
                            .fetch_optional(&db).await.ok().flatten();

                            match existing {
                                Some((id, _)) => {
                                    let _ = sqlx::query(
                                        "UPDATE clipboard_history SET last_used_at = \
                                         datetime('now','localtime') WHERE id = ?1"
                                    )
                                    .bind(&id).execute(&db).await;
                                    println!("[cb_mon] ↻ 更新 id={}", &id[..8]);
                                }
                                None => {
                                    let id = Uuid::new_v4().to_string();
                                    let r = sqlx::query(
                                        "INSERT INTO clipboard_history \
                                         (id, content_type, content, source_app, is_favorite, \
                                          created_at, last_used_at) \
                                         VALUES (?1, ?2, ?3, NULL, 0, \
                                          datetime('now','localtime'), datetime('now','localtime'))"
                                    )
                                    .bind(&id).bind(&ct_owned).bind(&c_clone)
                                    .execute(&db).await;

                                    if r.is_ok() {
                                        println!("[cb_mon] + {} len={}", ct_owned, c_clone.len());
                                    }

                                    // 清理（保留收藏的）
                                    let _ = sqlx::query(
                                        "DELETE FROM clipboard_history WHERE is_favorite = 0 \
                                         AND id NOT IN (SELECT id FROM clipboard_history \
                                         ORDER BY is_favorite DESC, last_used_at DESC LIMIT 200)"
                                    )
                                    .execute(&db).await;
                                }
                            }
                        });
                    }
                }

                std::thread::sleep(Duration::from_millis(500));
            }
            println!("[cb_mon] 已停止");
        });
    }

    pub fn stop(&self) { self.running.store(false, Ordering::SeqCst); }
}

/// 读取剪贴板内容，优先图片后文本
fn try_read_clipboard(clipboard: &mut arboard::Clipboard) -> Option<(&'static str, String)> {
    // 尝试图片
    if let Ok(img) = clipboard.get_image() {
        let w = img.width;
        let h = img.height;
        if w > 0 && h > 0 && w <= MAX_IMAGE_DIMENSION && h <= MAX_IMAGE_DIMENSION {
            if let Some(png) = encode_png(w, h, &img.bytes) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
                return Some(("image/png", b64));
            }
        }
    }

    // 尝试文本
    if let Ok(t) = clipboard.get_text() {
        if !t.is_empty() {
            return Some(("text", t));
        }
    }

    None
}

fn encode_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    use image::{ImageBuffer, Rgba};
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, rgba.to_vec())?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).ok()?;
    Some(buf.into_inner())
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
