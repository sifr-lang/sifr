//! Native backing for `_sifr.crypto` random helper declarations.

use std::sync::{LazyLock, Mutex, MutexGuard};

use rand::RngExt as _;
use rand_distr::Distribution as _;
use sifr_runtime::{SifrInt, interop::SifrIntBridge};

static RANDOM_MODULE_STATE: LazyLock<Mutex<RandomModuleState>> =
    LazyLock::new(|| Mutex::new(RandomModuleState::default()));

#[derive(Clone)]
struct RandomModuleState {
    words: Vec<SifrIntBridge>,
    index: SifrIntBridge,
    gauss_next: Option<f64>,
}

impl Default for RandomModuleState {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            index: SifrIntBridge::from(0),
            gauss_next: None,
        }
    }
}

#[must_use]
pub fn random_int(min: SifrIntBridge, max: SifrIntBridge) -> SifrIntBridge {
    let min = min.to_i64_saturating();
    let max = max.to_i64_saturating();
    if min >= max {
        return SifrIntBridge::from(min);
    }

    let exclusive_max = max.saturating_add(1);
    if exclusive_max <= min {
        return SifrIntBridge::from(min);
    }
    SifrIntBridge::from(rand::rng().random_range(min..exclusive_max))
}

#[must_use]
pub fn random_float() -> f64 {
    rand::random::<f64>()
}

/// Maps the low 32 bits of an exact integer to the half-open unit interval.
///
/// The Mersenne Twister implementation uses this after its word-level state
/// transition. Masking here makes the conversion total for every `SifrInt`
/// while preserving the generator's exact 32-bit semantics.
#[must_use]
pub fn random_word_to_unit_float(value: SifrIntBridge) -> f64 {
    let masked = value.into_sifr_int() & SifrInt::from(u32::MAX);
    let (_, digits) = masked.as_bigint().to_u32_digits();
    let word = match digits.first() {
        Some(word) => *word,
        None => 0,
    };
    f64::from(word) / 4_294_967_296.0
}

#[must_use]
pub fn random_seed() -> SifrIntBridge {
    SifrIntBridge::from(SifrInt::from(rand::random::<u64>()))
}

#[must_use]
pub fn random_uniform(min: f64, max: f64) -> f64 {
    min + (max - min) * random_float()
}

pub fn random_randrange(
    start: SifrIntBridge,
    stop: SifrIntBridge,
    step: SifrIntBridge,
) -> Result<SifrIntBridge, String> {
    let start = start.to_i64_saturating();
    let stop = stop.to_i64_saturating();
    let step = step.to_i64_saturating();
    if step == 0 {
        return Err("randrange: step must not be zero".to_string());
    }

    let width = stop.saturating_sub(start);
    if (step > 0 && width <= 0) || (step < 0 && width >= 0) {
        return Err("randrange: empty range".to_string());
    }

    let abs_width = width.unsigned_abs();
    let abs_step = step.unsigned_abs();
    let count = (abs_width + abs_step - 1) / abs_step;
    if count == 0 || count > i64::MAX as u64 {
        return Err("randrange: empty range".to_string());
    }

    let pick = rand::rng().random_range(0..count as i64);
    Ok(SifrIntBridge::from(
        start.saturating_add(pick.saturating_mul(step)),
    ))
}

#[must_use]
pub fn random_gauss(mu: f64, sigma: f64) -> f64 {
    rand_distr::Normal::new(mu, sigma)
        .map(|distribution| distribution.sample(&mut rand::rng()))
        .unwrap_or(mu)
}

#[must_use]
pub fn random_module_state_words() -> Vec<SifrIntBridge> {
    random_module_state().words.clone()
}

#[must_use]
pub fn random_module_state_index() -> SifrIntBridge {
    random_module_state().index.clone()
}

#[must_use]
pub fn random_module_state_gauss_next() -> Option<f64> {
    random_module_state().gauss_next
}

pub fn random_module_set_state(
    words: &[SifrIntBridge],
    index: SifrIntBridge,
    gauss_next: Option<f64>,
) -> Result<(), String> {
    let index_i64 = index.to_i64_saturating();
    if !(0..=624).contains(&index_i64) {
        return Err("random module state index must be in range [0, 624]".to_string());
    }
    if words.len() != 624 {
        return Err("random module state words must have length 624".to_string());
    }

    let mut state = random_module_state();
    state.words = words.to_vec();
    state.index = index;
    state.gauss_next = gauss_next;
    Ok(())
}

fn random_module_state() -> MutexGuard<'static, RandomModuleState> {
    RANDOM_MODULE_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_to_unit_float_uses_exact_low_32_bit_semantics() {
        let denominator = 4_294_967_296.0;
        assert_eq!(
            random_word_to_unit_float(SifrIntBridge::from(SifrInt::from(u32::MAX))),
            f64::from(u32::MAX) / denominator
        );
        assert_eq!(
            random_word_to_unit_float(SifrIntBridge::from(SifrInt::from(1_u64 << 32))),
            0.0
        );
        assert_eq!(
            random_word_to_unit_float(SifrIntBridge::from(SifrInt::from_i64(-1))),
            f64::from(u32::MAX) / denominator
        );
    }
}
