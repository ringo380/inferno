use serde::{Deserialize, Serialize};
use tracing::debug;

/// Sampling strategies for token generation
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SamplingStrategy {
    /// Always pick the highest probability token (deterministic)
    Greedy,
    /// Apply temperature scaling, then sample from probabilities
    Temperature,
    /// Keep only top K tokens by probability
    TopK,
    /// Keep tokens until cumulative probability reaches P (nucleus sampling)
    TopP,
    /// Combination of top-k and top-p
    TopKP,
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        SamplingStrategy::Temperature
    }
}

/// Configuration for token sampling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Which sampling strategy to use
    pub strategy: SamplingStrategy,

    /// Temperature parameter (0.0 = greedy, 1.0 = no change, > 1.0 = more random)
    /// Lower values make model more confident, higher values make it more creative
    pub temperature: f32,

    /// Top-K: Keep only top K tokens
    pub top_k: u32,

    /// Top-P (nucleus): Keep tokens until cumulative prob >= P
    pub top_p: f32,

    /// Penalty for repeating tokens (1.0 = no penalty, > 1.0 = discourage repetition)
    pub repeat_penalty: f32,

    /// Optional seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            strategy: SamplingStrategy::Temperature,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.9,
            repeat_penalty: 1.1,
            seed: None,
        }
    }
}

/// Simple token data structure for sampling
#[derive(Clone, Debug)]
pub struct TokenCandidate {
    pub id: i32,
    pub logit: f32,
    pub p: f32,
}

/// Token sampling engine
pub struct Sampler {
    config: SamplingConfig,
    recent_tokens: Vec<i32>,
    // TODO: Implement proper RNG for stochastic sampling
    // rng: Box<dyn std::any::Any>, // Can be upgraded to actual RNG later
}

impl Sampler {
    pub fn new(config: SamplingConfig) -> Self {
        Self {
            config,
            recent_tokens: Vec::new(),
        }
    }

    /// Sample a token based on configured strategy
    /// Accepts a generic token candidate with id, logit, and probability
    pub fn sample_from_candidates<T: AsRef<[(i32, f32, f32)]>>(
        &mut self,
        candidates_data: T,
    ) -> Option<i32> {
        let candidates_ref = candidates_data.as_ref();
        if candidates_ref.is_empty() {
            return None;
        }

        let mut candidates: Vec<TokenCandidate> = candidates_ref
            .iter()
            .map(|(id, logit, p)| TokenCandidate {
                id: *id,
                logit: *logit,
                p: *p,
            })
            .collect();

        self.sample_internal(&mut candidates)
    }

    /// Sample a token based on configured strategy (internal implementation)
    fn sample_internal(&mut self, candidates: &mut [TokenCandidate]) -> Option<i32> {
        if candidates.is_empty() {
            return None;
        }

        // Apply temperature scaling if not greedy
        if matches!(
            self.config.strategy,
            SamplingStrategy::Temperature | SamplingStrategy::TopKP
        ) {
            Self::apply_temperature(candidates, self.config.temperature);
        }

        // Apply top-k filtering
        let mut adjusted = candidates.to_vec();
        if matches!(
            self.config.strategy,
            SamplingStrategy::TopK | SamplingStrategy::TopKP
        ) && self.config.top_k > 0
        {
            Self::apply_top_k(&mut adjusted, self.config.top_k as usize);
        }

        // Apply top-p (nucleus) filtering
        if matches!(
            self.config.strategy,
            SamplingStrategy::TopP | SamplingStrategy::TopKP
        ) && self.config.top_p > 0.0
            && self.config.top_p < 1.0
        {
            Self::apply_top_p(&mut adjusted, self.config.top_p);
        }

        // Sample based on strategy
        let token = match self.config.strategy {
            SamplingStrategy::Greedy => Self::greedy_sample(&adjusted),
            _ => Self::probabilistic_sample(&adjusted),
        };

        // Track for repeat penalty
        if let Some(t) = token {
            self.recent_tokens.push(t);
            // Keep only recent history (last 50 tokens)
            if self.recent_tokens.len() > 50 {
                self.recent_tokens.remove(0);
            }
        }

        token
    }

    /// Sample a token based on configured strategy
    pub fn sample(&mut self, candidates: &[TokenCandidate]) -> Option<i32> {
        let mut candidates_vec = candidates.to_vec();
        self.sample_internal(&mut candidates_vec)
    }

    /// Apply temperature scaling to logits
    fn apply_temperature(candidates: &mut [TokenCandidate], temperature: f32) {
        if temperature <= 0.0 {
            return; // Invalid temperature
        }

        for token in candidates.iter_mut() {
            token.logit /= temperature;
        }

        debug!(
            "Applied temperature scaling: {} (adjusted logits for {} candidates)",
            temperature,
            candidates.len()
        );
    }

    /// Apply top-k filtering (keep only top k tokens)
    fn apply_top_k(candidates: &mut Vec<TokenCandidate>, k: usize) {
        if candidates.len() <= k {
            return; // Already small enough
        }

        // Sort by probability (descending)
        candidates.sort_by(|a, b| b.p.partial_cmp(&a.p).unwrap_or(std::cmp::Ordering::Equal));

        // Keep only top k
        candidates.truncate(k);

        debug!("Applied top-k filtering: kept {} tokens out of original", k);
    }

    /// Apply top-p (nucleus) filtering (keep tokens until cumulative prob >= p)
    fn apply_top_p(candidates: &mut Vec<TokenCandidate>, p: f32) {
        if candidates.is_empty() {
            return;
        }

        // Sort by probability (descending)
        candidates.sort_by(|a, b| b.p.partial_cmp(&a.p).unwrap_or(std::cmp::Ordering::Equal));

        // Find the cutoff point
        let mut cumsum = 0.0;
        let mut cutoff_idx = candidates.len();

        for (i, token) in candidates.iter().enumerate() {
            cumsum += token.p;
            if cumsum >= p {
                cutoff_idx = i + 1;
                break;
            }
        }

        candidates.truncate(cutoff_idx);

        debug!(
            "Applied top-p filtering: kept {} tokens for p={}",
            cutoff_idx, p
        );
    }

    /// Greedy sampling: pick token with highest probability
    fn greedy_sample(candidates: &[TokenCandidate]) -> Option<i32> {
        candidates
            .iter()
            .max_by(|a, b| a.p.partial_cmp(&b.p).unwrap_or(std::cmp::Ordering::Equal))
            .map(|t| t.id)
    }

    /// Probabilistic sampling: sample from probability distribution
    fn probabilistic_sample(candidates: &[TokenCandidate]) -> Option<i32> {
        if candidates.is_empty() {
            return None;
        }

        // Calculate probabilities from logits (softmax)
        let max_logit = candidates
            .iter()
            .map(|c| c.logit)
            .fold(f32::NEG_INFINITY, f32::max);

        let scores: Vec<f32> = candidates
            .iter()
            .map(|c| (c.logit - max_logit).exp())
            .collect();

        let sum: f32 = scores.iter().sum();
        if sum <= 0.0 {
            return None;
        }

        let probs: Vec<f32> = scores.iter().map(|s| s / sum).collect();

        // Sample using cumulative distribution
        // For now, use simple deterministic sampling (pick highest after temperature)
        // In production, would use proper random number generation
        let cumsum = probs.iter().map(|p| p.abs()).sum::<f32>();

        let mut cum = 0.0;
        for (i, prob) in probs.iter().enumerate() {
            cum += prob.abs();
            // Simple deterministic approach: use probability as weight
            if cum >= cumsum * 0.5 {
                return Some(candidates[i].id);
            }
        }

        // Fallback to highest probability
        candidates
            .iter()
            .max_by(|a, b| a.p.partial_cmp(&b.p).unwrap_or(std::cmp::Ordering::Equal))
            .map(|t| t.id)
    }

    /// Get recent token history
    pub fn get_recent_tokens(&self) -> &[i32] {
        &self.recent_tokens
    }

    /// Clear recent token history
    pub fn clear_history(&mut self) {
        self.recent_tokens.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy_sampling() {
        let candidates = vec![
            TokenCandidate {
                id: 1,
                logit: 0.1,
                p: 0.1,
            },
            TokenCandidate {
                id: 2,
                logit: 0.5,
                p: 0.5,
            },
            TokenCandidate {
                id: 3,
                logit: 0.3,
                p: 0.3,
            },
        ];

        let token = Sampler::greedy_sample(&candidates);
        assert_eq!(token, Some(2)); // Should pick highest probability
    }

    #[test]
    fn test_top_k_filtering() {
        let mut candidates = vec![
            TokenCandidate {
                id: 1,
                logit: 0.1,
                p: 0.1,
            },
            TokenCandidate {
                id: 2,
                logit: 0.5,
                p: 0.5,
            },
            TokenCandidate {
                id: 3,
                logit: 0.3,
                p: 0.3,
            },
            TokenCandidate {
                id: 4,
                logit: 0.05,
                p: 0.05,
            },
        ];

        Sampler::apply_top_k(&mut candidates, 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].id, 2); // Highest
        assert_eq!(candidates[1].id, 3); // Second highest
    }

    #[test]
    fn test_top_p_filtering() {
        let mut candidates = vec![
            TokenCandidate {
                id: 1,
                logit: 0.0,
                p: 0.5,
            },
            TokenCandidate {
                id: 2,
                logit: 0.0,
                p: 0.3,
            },
            TokenCandidate {
                id: 3,
                logit: 0.0,
                p: 0.15,
            },
            TokenCandidate {
                id: 4,
                logit: 0.0,
                p: 0.05,
            },
        ];

        Sampler::apply_top_p(&mut candidates, 0.8);
        // Should keep tokens 1, 2, 3 (0.5 + 0.3 + 0.15 = 0.95 > 0.8)
        assert_eq!(candidates.len(), 3);
    }

    #[test]
    fn test_sampler_with_config() {
        let config = SamplingConfig {
            strategy: SamplingStrategy::Greedy,
            temperature: 1.0,
            top_k: 40,
            top_p: 0.9,
            repeat_penalty: 1.1,
            seed: None,
        };

        let mut sampler = Sampler::new(config);

        let candidates = vec![TokenCandidate {
            id: 5,
            logit: 0.8,
            p: 0.8,
        }];

        let token = sampler.sample(&candidates);
        assert_eq!(token, Some(5));
        assert_eq!(sampler.get_recent_tokens(), &[5]);
    }

    #[test]
    fn test_temperature_scaling() {
        let mut candidates = vec![
            TokenCandidate {
                id: 1,
                logit: 2.0,
                p: 0.1,
            },
            TokenCandidate {
                id: 2,
                logit: 1.0,
                p: 0.5,
            },
        ];

        let original_logits = candidates.iter().map(|c| c.logit).collect::<Vec<_>>();

        Sampler::apply_temperature(&mut candidates, 2.0); // Higher temp = lower logits

        let scaled_logits = candidates.iter().map(|c| c.logit).collect::<Vec<_>>();

        // With temp=2.0, logits should be halved
        for (original, scaled) in original_logits.iter().zip(scaled_logits.iter()) {
            assert!(scaled < original);
        }
    }
}
