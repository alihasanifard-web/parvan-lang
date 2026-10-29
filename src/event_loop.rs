// ==============================================================================
// ⚡ PARVAN ASYNC EVENT-LOOP RUNTIME (موتور رویدادمحور ناهمگام پروان)
// معماران: علی حسنی‌فرد، احسان فضلی‌نژاد، امیرمهدی شهبازی
// ==============================================================================

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type AsyncTask = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone)]
pub struct EventLoop {
    tasks: Arc<Mutex<VecDeque<AsyncTask>>>,
    timers: Arc<Mutex<Vec<(Instant, AsyncTask)>>>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            timers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// اضافه کردن یک تسک ناهمگام به صف اجرای رویدادها
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut queue) = self.tasks.lock() {
            queue.push_back(Box::new(task));
        }
    }

    /// اضافه کردن یک تایمر با تاخیر میلی‌ثانیه‌ای (setTimeout)
    pub fn set_timeout<F>(&self, delay: Duration, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut timers) = self.timers.lock() {
            timers.push((Instant::now() + delay, Box::new(task)));
        }
    }

    /// اجرای چرخه رویدادها تا زمان اتمام تمام کارهای موجود در صف
    pub fn run_until_complete(&self) {
        loop {
            // 1. چک کردن تایمرهای فرا رسیده
            let mut ready_timers = Vec::new();
            if let Ok(mut timers) = self.timers.lock() {
                let now = Instant::now();
                let mut remaining = Vec::new();
                for (fire_at, task) in timers.drain(..) {
                    if now >= fire_at {
                        ready_timers.push(task);
                    } else {
                        remaining.push((fire_at, task));
                    }
                }
                *timers = remaining;
            }

            for t in ready_timers {
                t();
            }

            // 2. اجرای تسک‌های صف
            let next_task = if let Ok(mut queue) = self.tasks.lock() {
                queue.pop_front()
            } else {
                None
            };

            if let Some(task) = next_task {
                task();
            } else {
                // اگر کاری نبود، چک کن ببین تایمری مونده یا کلا تموم شدیم
                let has_timers = if let Ok(timers) = self.timers.lock() {
                    !timers.is_empty()
                } else {
                    false
                };

                if !has_timers {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
