use std::collections::HashMap;
use std::sync::Mutex;

use tokio::time::Instant;

use crate::error::{OutClawError, Result};

/// Token-bucket rate limiter for Tauri commands.
/// Prevents rapid-fire command invocation from UI bugs or automation abuse.
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    limits: HashMap<String, RateConfig>,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Clone)]
struct RateConfig {
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

/// Command categories and their rate limits
const MUTATING_COMMANDS: &[&str] = &[
    "create_instance",
    "update_instance",
    "delete_instance",
    "rename_instance",
];

const LIFECYCLE_COMMANDS: &[&str] = &[
    "start_instance",
    "stop_instance",
    "restart_instance",
    "restart_gateway",
];

const BUILD_COMMANDS: &[&str] = &["build_instance"];

impl RateLimiter {
    /// Create a new rate limiter with default limits:
    /// - Mutating: 10/min
    /// - Lifecycle: 20/min
    /// - Build: 5/min
    /// - Default (read-only): 60/min
    pub fn new() -> Self {
        let mut limits = HashMap::new();

        let mutating = RateConfig {
            max_tokens: 10.0,
            refill_rate: 10.0 / 60.0,
        };
        for cmd in MUTATING_COMMANDS {
            limits.insert(cmd.to_string(), mutating.clone());
        }

        let lifecycle = RateConfig {
            max_tokens: 20.0,
            refill_rate: 20.0 / 60.0,
        };
        for cmd in LIFECYCLE_COMMANDS {
            limits.insert(cmd.to_string(), lifecycle.clone());
        }

        let build = RateConfig {
            max_tokens: 5.0,
            refill_rate: 5.0 / 60.0,
        };
        for cmd in BUILD_COMMANDS {
            limits.insert(cmd.to_string(), build.clone());
        }

        Self {
            buckets: Mutex::new(HashMap::new()),
            limits,
        }
    }

    /// Check if a command is allowed under rate limits.
    /// Returns Ok(()) if allowed, Err(RateLimitExceeded) if blocked.
    pub fn check(&self, command: &str) -> Result<()> {
        let config = match self.limits.get(command) {
            Some(c) => c.clone(),
            None => {
                // Default: 60 requests per minute for unlisted commands
                RateConfig {
                    max_tokens: 60.0,
                    refill_rate: 1.0,
                }
            }
        };

        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();

        let bucket = buckets.entry(command.to_string()).or_insert(TokenBucket {
            tokens: config.max_tokens,
            last_refill: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * config.refill_rate).min(config.max_tokens);
        bucket.last_refill = now;

        // Try to consume a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            Err(OutClawError::RateLimitExceeded)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_normal_usage() {
        let limiter = RateLimiter::new();

        // Should allow several requests
        for _ in 0..5 {
            assert!(limiter.check("create_instance").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_excessive_usage() {
        let limiter = RateLimiter::new();

        // Exhaust the mutating command bucket (10 tokens)
        for _ in 0..10 {
            assert!(limiter.check("create_instance").is_ok());
        }

        // Next request should be rate-limited
        assert!(limiter.check("create_instance").is_err());
    }

    #[test]
    fn test_rate_limiter_default_for_unknown_commands() {
        let limiter = RateLimiter::new();

        // Unknown commands get the default limit (60/min)
        for _ in 0..30 {
            assert!(limiter.check("list_instances").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_independent_buckets() {
        let limiter = RateLimiter::new();

        // Exhaust create_instance
        for _ in 0..10 {
            assert!(limiter.check("create_instance").is_ok());
        }
        assert!(limiter.check("create_instance").is_err());

        // start_instance should still work (different bucket)
        assert!(limiter.check("start_instance").is_ok());
    }
}
