use std::sync::{Mutex, MutexGuard, OnceLock};

#[derive(Debug, Clone)]
struct ValueError(String);

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RandomState {
    state: u64,
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn randint_from_state(state: &mut u64, low: i64, high: i64) -> Result<i64, ValueError> {
    if low > high {
        return Err(ValueError(
            "random.randint: low must be <= high".to_string(),
        ));
    }
    let span = (high - low + 1) as u64;
    Ok(low + (next_u64(state) % span) as i64)
}

fn random_from_state(state: &mut u64) -> f64 {
    next_u64(state) as f64 / u64::MAX as f64
}

#[derive(Debug, Clone)]
struct Random {
    state: u64,
}

impl Random {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn getstate(&self) -> RandomState {
        RandomState { state: self.state }
    }

    fn setstate(&mut self, state: RandomState) -> Result<(), ValueError> {
        self.state = state.state;
        Ok(())
    }

    fn randint(&mut self, low: i64, high: i64) -> Result<i64, ValueError> {
        randint_from_state(&mut self.state, low, high)
    }
}

fn module_state() -> &'static Mutex<u64> {
    static STATE: OnceLock<Mutex<u64>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(0))
}

fn lock_module_state() -> MutexGuard<'static, u64> {
    match module_state().lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn seed(seed: u64) {
    *lock_module_state() = seed;
}

fn random() -> f64 {
    let mut state = lock_module_state();
    random_from_state(&mut state)
}

fn randint(low: i64, high: i64) -> Result<i64, ValueError> {
    let mut state = lock_module_state();
    randint_from_state(&mut state, low, high)
}

fn getstate() -> RandomState {
    RandomState {
        state: *lock_module_state(),
    }
}

fn setstate(state: RandomState) -> Result<(), ValueError> {
    *lock_module_state() = state.state;
    Ok(())
}

fn main() -> Result<(), ValueError> {
    let mut rng = Random::new(77);
    let state_before = rng.getstate();
    let next_one = rng.randint(0, 100000)?;
    rng.setstate(state_before)?;
    let replay_one = rng.randint(0, 100000)?;
    assert_eq!(next_one, replay_one);

    seed(1234);
    let first_module_random = random();
    let second_module_int = randint(0, 100000)?;
    let module_state = getstate();

    let after_state_int = randint(0, 100000)?;
    setstate(module_state)?;
    let replay_after_state_int = randint(0, 100000)?;
    assert_eq!(after_state_int, replay_after_state_int);

    seed(1234);
    assert_eq!(first_module_random, random());
    let replay_second_module_int = randint(0, 100000)?;
    assert_eq!(second_module_int, replay_second_module_int);

    println!("rng_random_state_object_model_demo: pass");
    Ok(())
}
