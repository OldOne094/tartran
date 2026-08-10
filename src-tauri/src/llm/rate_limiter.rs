use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Token-bucket rate limiter. Thread-safe. Used to respect per-model RPM budgets.
pub struct RateLimiter {
    state: Mutex<Bucket>,
}

struct Bucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last: Instant,
}

impl RateLimiter {
    /// `rpm` = requests per minute budget.
    pub fn new(rpm: f64) -> Self {
        let refill_per_sec = (rpm / 60.0).max(0.0001);
        RateLimiter {
            state: Mutex::new(Bucket {
                capacity: refill_per_sec * 60.0,
                tokens: refill_per_sec * 60.0,
                refill_per_sec,
                last: Instant::now(),
            }),
        }
    }

    /// Block until a token is available.
    pub fn acquire(&self) {
        let mut b = self.state.lock().unwrap();
        loop {
            let now = Instant::now();
            let elapsed = now.duration_since(b.last).as_secs_f64();
            b.last = now;
            b.tokens = (b.tokens + elapsed * b.refill_per_sec).min(b.capacity);
            if b.tokens >= 1.0 {
                b.tokens -= 1.0;
                return;
            }
            let need = 1.0 - b.tokens;
            let wait = Duration::from_secs_f64(need / b.refill_per_sec);
            drop(b);
            std::thread::sleep(wait);
            b = self.state.lock().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn allows_immediate_burst_within_capacity() {
        let limiter = RateLimiter::new(120.0);
        let start = Instant::now();
        limiter.acquire();
        limiter.acquire();
        assert!(start.elapsed() < Duration::from_millis(500));
    }
}
