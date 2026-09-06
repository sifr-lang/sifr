// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        SifrGeneratedStdlibSifrX2erandomX2eRandom, SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn random_word_to_unit_float(value: SifrInt) -> f64 {
        ::sifr_stdlib::random::random_word_to_unit_float(
            ::sifr_runtime::interop::SifrIntBridge::from(value),
        )
    }
    pub(crate) fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub(crate) fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(crate) fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub(crate) fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub(crate) fn random_module_set_state(
        words: &[SifrInt],
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next,
        )
        .map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4e() -> SifrInt {
        SifrInt::from_i64(624)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d() -> SifrInt {
        SifrInt::from_i64(397)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d41545249585f41() -> SifrInt {
        SifrInt::from_i64(2_567_483_615)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f55505045525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_648)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4c4f5745525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_647)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f46() -> SifrInt {
        SifrInt::from_i64(1_812_433_253)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f574f52445f4d41534b() -> SifrInt {
        SifrInt::from_i64(4_294_967_295)
    }
    pub(crate) fn sifr_generated_state_word_at(words: &[SifrInt], index: SifrInt) -> SifrInt {
        let value: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &words;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value.clone() else {
            return SifrInt::from_i64(0);
        };
        value_value_7ce4fd9430e80cea
    }
    pub(crate) fn sifr_generated_clone_words(words: &[SifrInt]) -> Vec<SifrInt> {
        let mut copied: Vec<SifrInt> = Vec::new();
        for word in words.iter().cloned() {
            copied.push(word);
        }
        copied
    }
    pub(crate) fn sifr_generated_normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
        let Some(seed_value) = seed_value.clone() else {
            return random_seed();
        };
        seed_value.clone()
    }
    pub(crate) fn sifr_generated_seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
        let mut words: Vec<SifrInt> =
            vec![&seed_value & &sifr_generated_const_5f4d545f574f52445f4d41534b()];
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &sifr_generated_const_5f4d545f4e() {
            let prev: SifrInt = sifr_generated_state_word_at(&words, &i - &SifrInt::from_i64(1));
            let next_word: SifrInt = &(&(&sifr_generated_const_5f4d545f46()
                * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1_073_741_824))))
                + &i)
                & &sifr_generated_const_5f4d545f574f52445f4d41534b();
            words.push(next_word);
            i = &i + &SifrInt::from_i64(1);
        }
        words
    }
    pub(crate) fn sifr_generated_build_state_from_module_storage()
    -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
            SifrInt::from_i64(3),
            random_module_state_words(),
            random_module_state_index(),
            random_module_state_gauss_next(),
        )
    }
    pub(crate) fn sifr_generated_store_state_into_module_storage(
        state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
    ) {
        let sifr_generated_set_result: Result<(), ValueError> = random_module_set_state(
            &sifr_generated_clone_words(&state.state_words.clone()),
            state.index.clone(),
            state.gauss_next,
        );
        let _ = sifr_generated_set_result;
    }
    pub(crate) fn sifr_generated_ensure_module_state_initialized() {
        let words: Vec<SifrInt> = random_module_state_words();
        if &SifrInt::from(words.len()) == &sifr_generated_const_5f4d545f4e() {
            return;
        }
        let bootstrap: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(5489)));
        sifr_generated_store_state_into_module_storage(&bootstrap.getstate());
    }
    pub(crate) fn sifr_generated_module_random() -> SifrGeneratedStdlibSifrX2erandomX2eRandom {
        sifr_generated_ensure_module_state_initialized();
        let mut r: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(0)));
        let sifr_generated_set_result: Result<(), ValueError> =
            r.setstate(&sifr_generated_build_state_from_module_storage());
        let _ = sifr_generated_set_result;
        r
    }
    pub(crate) fn sifr_generated_sync_module_random(
        generator: &mut SifrGeneratedStdlibSifrX2erandomX2eRandom,
    ) {
        sifr_generated_store_state_into_module_storage(&generator.getstate());
    }
    pub(crate) fn seed(seed_value: Option<SifrInt>) {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(seed_value.clone());
        sifr_generated_sync_module_random(&mut generator);
    }
    pub(crate) fn getstate() -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        sifr_generated_ensure_module_state_initialized();
        sifr_generated_build_state_from_module_storage()
    }
    pub(crate) fn setstate(
        state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
    ) -> Result<(), ValueError> {
        let mut probe: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(0)));
        let result: Result<(), ValueError> = probe.setstate(state);
        sifr_generated_sync_module_random(&mut probe);
        result
    }
    pub(crate) fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
        sifr_generated_sync_module_random(&mut generator);
        value
    }
    pub(crate) fn random() -> f64 {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: f64 = generator.random();
        sifr_generated_sync_module_random(&mut generator);
        value
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        pub version: SifrInt,
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        #[must_use]
        pub fn new(
            version: SifrInt,
            state_words: Vec<SifrInt>,
            index: SifrInt,
            gauss_next: Option<f64>,
        ) -> Self {
            let sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e: SifrInt =
                version.clone();
            let sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473: Vec<SifrInt> =
                state_words;
            let sifr_generated_field_value_83cf8e8f9081468b_696e646578: SifrInt = index.clone();
            let sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874: Option<f64> =
                gauss_next;
            Self {
                version: sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e,
                state_words: sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473,
                index: sifr_generated_field_value_83cf8e8f9081468b_696e646578,
                gauss_next: sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn new(seed_value: Option<SifrInt>) -> Self {
            let normalized_seed: SifrInt = sifr_generated_normalize_seed_input(seed_value.clone());
            let sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473: Vec<SifrInt> =
                sifr_generated_seed_words_from_seed(normalized_seed.clone());
            let sifr_generated_field_value_497043933c8a2d12_5f696e646578: SifrInt =
                sifr_generated_const_5f4d545f4e().clone();
            let sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874: Option<f64> =
                None;
            Self {
                state_words: sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473,
                index: sifr_generated_field_value_497043933c8a2d12_5f696e646578,
                gauss_next: sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub fn sifr_generated_twist(&mut self) {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(self.state_words.len()) {
                let y: SifrInt = &(&sifr_generated_state_word_at(&self.state_words, i.clone())
                    & &sifr_generated_const_5f4d545f55505045525f4d41534b())
                    + &(&sifr_generated_state_word_at(
                        &self.state_words,
                        (&i + &SifrInt::from_i64(1))
                            .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                    ) & &sifr_generated_const_5f4d545f4c4f5745525f4d41534b());
                let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
                if &y.floor_mod_known_nonzero(&SifrInt::from_i64(2)) != &SifrInt::from_i64(0) {
                    x_a = &x_a ^ &sifr_generated_const_5f4d545f4d41545249585f41();
                }
                let new_word: SifrInt = &sifr_generated_state_word_at(
                    &self.state_words,
                    (&i + &sifr_generated_const_5f4d545f4d())
                        .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                ) ^ &x_a;
                {
                    let sifr_generated_assign_value =
                        &new_word & &sifr_generated_const_5f4d545f574f52445f4d41534b();
                    {
                        let sifr_generated_index_raw = i.clone();
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(self.state_words.len());
                        if let Some(sifr_generated_elem) =
                            self.state_words.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            self.index = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn sifr_generated_next_u32(&mut self) -> SifrInt {
            if &self.index.clone() >= &sifr_generated_const_5f4d545f4e() {
                self.sifr_generated_twist();
            }
            let mut y: SifrInt =
                sifr_generated_state_word_at(&self.state_words, self.index.clone());
            self.index = &self.index.clone() + &SifrInt::from_i64(1);
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
            y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2_636_928_640));
            y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4_022_730_752));
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262_144));
            &y & &sifr_generated_const_5f4d545f574f52445f4d41534b()
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn random(&mut self) -> f64 {
            random_word_to_unit_float(self.sifr_generated_next_u32())
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randrange(
            &mut self,
            start: &SifrInt,
            stop: &Option<SifrInt>,
            step_argument_af0b4e191da20cef: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if step_argument_af0b4e191da20cef == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let mut actual_start: SifrInt = start.clone();
            let mut actual_stop_value_351bdef5a4961be0: SifrInt = start.clone();
            if stop.is_none() {
                actual_start = SifrInt::from_i64(0);
            } else if let Some(stop) = stop.as_ref() {
                actual_stop_value_351bdef5a4961be0 = stop.clone();
            }
            let width: SifrInt = &actual_stop_value_351bdef5a4961be0 - &actual_start;
            if step_argument_af0b4e191da20cef > &SifrInt::from_i64(0) {
                if &width <= &SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else if &width >= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let mut abs_width: SifrInt = width.clone();
            if &abs_width < &SifrInt::from_i64(0) {
                abs_width = &SifrInt::from_i64(0) - &abs_width;
            }
            let mut abs_step: SifrInt = step_argument_af0b4e191da20cef.clone();
            if &abs_step < &SifrInt::from_i64(0) {
                abs_step = &SifrInt::from_i64(0) - &abs_step;
            }
            if &abs_step == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&abs_step);
            if &count <= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            if &count == &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: SifrInt = self
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&count);
            Ok(&actual_start + &(&pick * step_argument_af0b4e191da20cef))
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randint(
            &mut self,
            minimum: &SifrInt,
            maximum: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if *minimum > *maximum {
                return Err(ValueError::new("randint: min must be <= max".to_string()));
            }
            self.randrange(
                minimum,
                &Some((maximum + &SifrInt::from_i64(1)).clone()),
                &SifrInt::from_i64(1),
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn getstate(&self) -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
            SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
                SifrInt::from_i64(3),
                sifr_generated_clone_words(&self.state_words),
                self.index.clone(),
                self.gauss_next,
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn setstate(
            &mut self,
            state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ) -> Result<(), ValueError> {
            if &state.version.clone() != &SifrInt::from_i64(3) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if &SifrInt::from(state.state_words.len()) != &sifr_generated_const_5f4d545f4e() {
                return Err(ValueError::new(
                    "setstate: state_words must have length 624".to_string(),
                ));
            }
            if &state.index.clone() < &SifrInt::from_i64(0)
                || &state.index.clone() > &sifr_generated_const_5f4d545f4e()
            {
                return Err(ValueError::new(
                    "setstate: index must be in range [0, 624]".to_string(),
                ));
            }
            let mut normalized: Vec<SifrInt> = Vec::new();
            for word in state.state_words.iter().cloned() {
                if &word < &SifrInt::from_i64(0)
                    || &word > &sifr_generated_const_5f4d545f574f52445f4d41534b()
                {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(&word & &sifr_generated_const_5f4d545f574f52445f4d41534b());
            }
            self.state_words = normalized;
            self.index = state.index.clone();
            self.gauss_next = state.gauss_next;
            Ok(())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::ValueError;
fn main() {
    let mut rng: SifrGeneratedStdlibSifrX2erandomX2eRandom =
        SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(77)));
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let state_before: SifrGeneratedStdlibSifrX2erandomX2eRandomState = rng.getstate();
        let next_one: SifrInt =
            (&mut rng).randint(&SifrInt::from_i64(0), &SifrInt::from_i64(100_000))?;
        let sifr_generated_rng_set_result: Result<(), ValueError> =
            (&mut rng).setstate(&state_before);
        let _ = sifr_generated_rng_set_result;
        let replay_one: SifrInt =
            (&mut rng).randint(&SifrInt::from_i64(0), &SifrInt::from_i64(100_000))?;
        assert_eq!(&next_one, &replay_one);
        seed(Some(SifrInt::from_i64(1234)));
        let first_module_random: f64 = random();
        let second_module_int: SifrInt = randint(SifrInt::from_i64(0), SifrInt::from_i64(100_000))?;
        let module_state: SifrGeneratedStdlibSifrX2erandomX2eRandomState = getstate();
        let after_state_int: SifrInt = randint(SifrInt::from_i64(0), SifrInt::from_i64(100_000))?;
        let sifr_generated_module_set_result: Result<(), ValueError> = setstate(&module_state);
        let _ = sifr_generated_module_set_result;
        let replay_after_state_int: SifrInt =
            randint(SifrInt::from_i64(0), SifrInt::from_i64(100_000))?;
        assert_eq!(&after_state_int, &replay_after_state_int);
        seed(Some(SifrInt::from_i64(1234)));
        assert_eq!(first_module_random, random());
        let replay_second_module_int: SifrInt =
            randint(SifrInt::from_i64(0), SifrInt::from_i64(100_000))?;
        assert_eq!(&second_module_int, &replay_second_module_int);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        assert_eq!(
            format!("unexpected random error: {}", e.message.clone()),
            "rng_random_state_object_model_demo: pass"
        );
    }
    assert_eq!(
        "rng_random_state_object_model_demo: pass".to_string(),
        "rng_random_state_object_model_demo: pass"
    );
}
