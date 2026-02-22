//! Random intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_random_int(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand::Rng; rand::thread_rng().gen_range({}..={}) }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_random_float(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ use rand::Rng; rand::thread_rng().gen::<f64>() }".to_string(),
    ))
}

pub(super) fn lower_random_choice(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand::Rng; let items = {}; items[rand::thread_rng().gen_range(0..items.len())].clone() }}",
        args[0]
    )))
}

pub(super) fn lower_random_uniform(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand::Rng; rand::thread_rng().gen_range({}..={}) }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_random_shuffle(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand::seq::SliceRandom; let mut __v = {}.clone(); __v.shuffle(&mut rand::thread_rng()); __v }}",
        args[0]
    )))
}

pub(super) fn lower_random_sample(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand::seq::SliceRandom; let __items = &{}; let __k = {} as usize; if __k > __items.len() {{ Err(ValueError {{ message: format!(\"sample larger than population: {{}} > {{}}\", __k, __items.len()) }}) }} else {{ Ok(__items.choose_multiple(&mut rand::thread_rng(), __k).cloned().collect::<Vec<_>>()) }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_random_randrange(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ let __start = {}; let __stop = {}; let __step = {}; if __step == 0 {{ Err(ValueError {{ message: \"randrange: step must not be zero\".to_string() }}) }} else if __start >= __stop && __step > 0 {{ Err(ValueError {{ message: \"randrange: empty range\".to_string() }}) }} else {{ use rand::Rng; let __n = ((__stop - __start + __step - 1) / __step).abs(); Ok(__start + rand::thread_rng().gen_range(0..__n) * __step) }} }}",
        args[0], args[1], args[2]
    )))
}

pub(super) fn lower_random_gauss(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use rand_distr::{{Normal, Distribution}}; let __mu = {}; let __sigma = {}; Normal::new(__mu, __sigma).map(|d| d.sample(&mut rand::thread_rng())).unwrap_or(__mu) }}",
        args[0], args[1]
    )))
}
