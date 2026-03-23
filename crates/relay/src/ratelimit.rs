//! Rate limiting — token bucket per user

use std::collections::HashMap;
use std::time::{Duration, Instant};

use uuid::Uuid;

/// A token bucket rate limiter
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

/// Rate limit categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitKind {
    /// Channel messages
    Message,
    /// Typing indicators
    Typing,
    /// Presence updates
    Presence,
    /// Reactions
    Reaction,
    /// Voice state changes
    Voice,
    /// Global (all actions)
    Global,
}

/// Per-user rate limiter
pub struct RateLimiter {
    buckets: HashMap<(Uuid, RateLimitKind), TokenBucket>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Check if an action is allowed (consumes a token if yes)
    pub fn check(&mut self, user_id: Uuid, kind: RateLimitKind) -> bool {
        let bucket = self
            .buckets
            .entry((user_id, kind))
            .or_insert_with(|| Self::default_bucket(kind));

        if !bucket.try_consume() {
            return false;
        }

        // Also check global limit
        if kind != RateLimitKind::Global {
            let global = self
                .buckets
                .entry((user_id, RateLimitKind::Global))
                .or_insert_with(|| Self::default_bucket(RateLimitKind::Global));

            if !global.try_consume() {
                return false;
            }
        }

        true
    }

    fn default_bucket(kind: RateLimitKind) -> TokenBucket {
        match kind {
            // 5 messages/sec burst, 2/sec sustained
            RateLimitKind::Message => TokenBucket::new(5.0, 2.0),
            // 1 typing event per 5 seconds
            RateLimitKind::Typing => TokenBucket::new(1.0, 0.2),
            // 2 presence updates per 10 seconds
            RateLimitKind::Presence => TokenBucket::new(2.0, 0.2),
            // 10 reactions/sec burst
            RateLimitKind::Reaction => TokenBucket::new(10.0, 3.0),
            // 5 voice state changes per 10 seconds
            RateLimitKind::Voice => TokenBucket::new(5.0, 0.5),
            // 20 actions/sec global
            RateLimitKind::Global => TokenBucket::new(20.0, 10.0),
        }
    }

    /// Clean up stale buckets (users who haven't been seen in a while)
    pub fn cleanup(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.buckets
            .retain(|_, bucket| bucket.last_refill > cutoff);
    }
}
