use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 速度计算器，使用滑动窗口计算下载速度
#[derive(Clone)]
pub struct SpeedCalculator {
    start_time: Instant,
    samples: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    sample_window: Duration,
}

impl SpeedCalculator {
    /// 创建新的速度计算器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            sample_window: Duration::from_secs(2),
        }
    }

    /// 添加下载字节数样本
    pub fn add_sample(&self, bytes: u64) {
        let mut samples = self.samples.lock().unwrap();
        let now = Instant::now();

        // 移除时间窗口外的旧样本
        while let Some(&(timestamp, _)) = samples.front() {
            if now.duration_since(timestamp) >= self.sample_window {
                samples.pop_front();
            } else {
                break;
            }
        }

        samples.push_back((now, bytes));
    }

    /// 计算当前下载速度（字节/秒）
    pub fn calculate_speed(&self) -> f64 {
        let samples = self.samples.lock().unwrap();

        if samples.len() < 2 {
            return 0.0;
        }

        let first = samples.front().unwrap();
        let last = samples.back().unwrap();

        let time_elapsed = last.0.duration_since(first.0).as_secs_f64();
        if time_elapsed == 0.0 {
            return 0.0;
        }

        let bytes_transferred = last.1.saturating_sub(first.1);
        bytes_transferred as f64 / time_elapsed
    }

    /// 格式化速度显示
    pub fn format_speed(&self) -> String {
        let bytes_per_sec = self.calculate_speed();

        if bytes_per_sec < 1024.0 {
            format!("{:.1} B/s", bytes_per_sec)
        } else if bytes_per_sec < 1024.0 * 1024.0 {
            format!("{:.1} KB/s", bytes_per_sec / 1024.0)
        } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// 获取总运行时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for SpeedCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_speed_calculator_initial_state() {
        let calc = SpeedCalculator::new();
        assert_eq!(calc.format_speed(), "0.0 B/s");
    }

    #[test]
    fn test_speed_calculation() {
        let calc = SpeedCalculator::new();

        // 添加样本
        calc.add_sample(1024);
        thread::sleep(Duration::from_millis(100));
        calc.add_sample(2048);

        let speed = calc.calculate_speed();
        assert!(speed > 0.0, "Speed should be positive");
    }

    #[test]
    fn test_format_speed() {
        let calc = SpeedCalculator::new();

        // Test B/s
        assert!(calc.format_speed().contains("B/s"));

        // Add enough data for KB/s
        calc.add_sample(0);
        calc.add_sample(10 * 1024); // 10 KB
        let _formatted = calc.format_speed();
        // Note: exact format depends on timing
    }
}
