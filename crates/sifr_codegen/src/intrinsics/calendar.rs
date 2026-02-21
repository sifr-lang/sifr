//! Calendar intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_calendar_isleap(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __y = {}; (__y % 4 == 0 && __y % 100 != 0) || (__y % 400 == 0) }}",
        args[0]
    )))
}

pub(super) fn lower_calendar_weekday(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __y0 = {}; let __m0 = {}; let __d0 = {}; let __t = [0i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]; let __y = if __m0 < 3 {{ __y0 - 1 }} else {{ __y0 }}; ((__y + __y/4 - __y/100 + __y/400 + __t[(__m0-1) as usize] + __d0) % 7 + 6) % 7 }}",
        args[0], args[1], args[2]
    )))
}

pub(super) fn lower_calendar_monthrange(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __y = {}; let __m = {}; let __days = match __m {{ 1|3|5|7|8|10|12 => 31i64, 4|6|9|11 => 30, 2 => if (__y%4==0 && __y%100!=0)||(__y%400==0) {{ 29 }} else {{ 28 }}, _ => 30 }}; let __t = [0i64,3,2,5,0,3,5,1,4,6,2,4]; let __y2 = if __m < 3 {{ __y-1 }} else {{ __y }}; let __wd = ((__y2+__y2/4-__y2/100+__y2/400+__t[(__m-1) as usize]+1)%7+6)%7; vec![__wd, __days] }}",
        args[0], args[1]
    )))
}
