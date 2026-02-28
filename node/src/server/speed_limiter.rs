use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant};

/// 基于 token bucket 的速度限制器
/// 所有代理连接共享同一个实例，限制整个节点的总带宽
pub struct SpeedLimiter {
    /// 速率限制(bytes/sec)，0 = 不限速
    rate: AtomicU64,
    /// 当前可用 token 数（字节）
    available: std::sync::Mutex<f64>,
    /// 上次补充 token 的时间
    last_refill: std::sync::Mutex<Instant>,
    /// 通知等待中的消费者有新 token
    notify: Notify,
}

impl SpeedLimiter {
    pub fn new(rate: u64) -> Arc<Self> {
        let limiter = Arc::new(Self {
            rate: AtomicU64::new(rate),
            available: std::sync::Mutex::new(rate as f64),
            last_refill: std::sync::Mutex::new(Instant::now()),
            notify: Notify::new(),
        });

        // 启动后台补充 token 任务
        let limiter_clone = limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                limiter_clone.refill();
            }
        });

        limiter
    }

    /// 补充 token
    fn refill(&self) {
        let rate = self.rate.load(Ordering::Relaxed);
        if rate == 0 {
            return;
        }

        let now = Instant::now();
        let elapsed = {
            let mut last = self.last_refill.lock().unwrap();
            let elapsed = now.duration_since(*last);
            *last = now;
            elapsed
        };

        let tokens_to_add = rate as f64 * elapsed.as_secs_f64();
        let max_tokens = rate as f64; // 最多积攒 1 秒的量

        {
            let mut available = self.available.lock().unwrap();
            *available = (*available + tokens_to_add).min(max_tokens);
        }

        self.notify.notify_waiters();
    }

    /// 消费指定字节数的 token，如果不够则等待
    pub async fn consume(&self, bytes: usize) {
        let rate = self.rate.load(Ordering::Relaxed);
        if rate == 0 {
            return; // 不限速
        }

        let mut remaining = bytes as f64;

        loop {
            {
                let mut available = self.available.lock().unwrap();
                if *available >= remaining {
                    *available -= remaining;
                    return;
                }
                // 消费所有可用的 token
                remaining -= *available;
                *available = 0.0;
            }

            // 等待补充
            self.notify.notified().await;
        }
    }

    /// 动态更新速率
    pub fn update_rate(&self, new_rate: u64) {
        self.rate.store(new_rate, Ordering::Relaxed);
        if new_rate == 0 {
            // 切换到不限速时，唤醒所有等待者
            self.notify.notify_waiters();
        }
    }

    /// 获取当前速率
    pub fn get_rate(&self) -> u64 {
        self.rate.load(Ordering::Relaxed)
    }
}
