#[cfg(doc)]
use crate::{format, write};

/// [`format()`] as a macro to allow specifying the context
/// directly.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use interpolator::iformat;
///
/// assert_eq!(
///     iformat!("{a}, {b:?}", a = 10, b:? = "test").unwrap(),
///     "10, \"test\""
/// )
/// ```
#[macro_export]
macro_rules! iformat {
    ($fmt:expr, $($context:tt)*) => {
        $crate::format($fmt, &$crate::context!($($context)*))
    };
}

/// [`write()`] as a macro to allow specifying the context
/// directly.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use std::fmt::Write;
/// use interpolator::iwrite;
///
/// let mut buf = String::new();
/// iwrite!(&mut buf, "{a}, {b:?}", a = 10, b:? = "test").unwrap();
///
/// assert_eq!( buf, "10, \"test\"" );
/// ```
#[macro_export]
macro_rules! iwrite {
    ($buf: expr, $fmt:expr, $($context:tt)*) => {
        $crate::write($buf, $fmt, &$crate::context!($($context)*))
    };
}

/// Like [`writeln!`] but using [`write()`].
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use std::fmt::Write;
/// use interpolator::iwriteln;
///
/// let mut buf = String::new();
/// iwriteln!(&mut buf, "{a}, {b:?}", a = 10, b:? = "test").unwrap();
///
/// assert_eq!( buf, "10, \"test\"\n" );
/// ```
#[macro_export]
macro_rules! iwriteln {
    ($buf: expr, $($fmt:tt)*) => {
        match $crate::iwrite!($buf, $($fmt)*) {
            // Cannot think of a better error index
            Ok(()) => std::write!($buf, "\n").map_err(|e| $crate::Error::Fmt(e, 0)),
            e => e
        }
    };
}

/// [`print!`] but using [`iformat!`].
///
/// Currently this allocates the complete formatted string.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use interpolator::iprint;
/// iprint!("{a}, {b:?}", a = 10, b:? = "test").unwrap();
/// ```
#[macro_export]
macro_rules! iprint {
    ($($fmt:tt)*) => {
        $crate::iformat!($($fmt)*).map(|f| std::print!("{f}"))
    };
}

/// [`println!`] but using [`iformat!`].
///
/// Currently this allocates the complete formatted string.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use interpolator::iprintln;
/// iprintln!("{a}, {b:?}", a = 10, b:? = "test").unwrap();
/// ```
#[macro_export]
macro_rules! iprintln {
    ($($fmt:tt)*) => {
        $crate::iformat!($($fmt)*).map(|f| std::println!("{f}"))
    };
}

/// [`eprint!`] but using [`iformat!`].
///
/// Currently this allocates the complete formatted string.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use interpolator::ieprint;
/// ieprint!("{a}, {b:?}", a = 10, b:? = "test").unwrap();
/// ```
#[macro_export]
macro_rules! ieprint {
    ($($fmt:tt)*) => {
        $crate::iformat!($($fmt)*).map(|f| std::eprint!("{f}"))
    };
}

/// [`eprintln!`] but using [`iformat!`].
///
/// Currently this allocates the complete formatted string.
///
/// For details on the context syntax see [`context!`](crate::context).
///
/// ```
/// use interpolator::ieprintln;
/// ieprintln!("{a}, {b:?}", a = 10, b:? = "test").unwrap();
/// ```
#[macro_export]
macro_rules! ieprintln {
    ($($fmt:tt)*) => {
        $crate::iformat!($($fmt)*).map(|f| std::eprintln!("{f}"))
    };
}

/// Creates a context for use with this crate's [`format()`] and [`write()`]
/// functions.
///
/// It takes a comma seperated list of idents, followed by optional format
/// specifiers and an optional value.
///
/// - `a` => key = `"a"`, value = `a`, traits = `[Display]`
/// - `a:?` => key = `"a"`, value = `a`, traits = `[Debug]`
/// - `a: :?` => key = `"a"`, value = `a`, traits = `[Debug, Display]`
/// - `a = 5` => key = `"a"`, value = `5`, traits = `[Display]`
/// - `a:?:x = 5` => key = `"a"`, value = `5`, traits = `[Debug, LowerHex]`
///
/// ```
/// use interpolator::context;
///
/// let a = "hello";
/// let context = context! {
///     a,
///     a:?, // This does not override the display implementation, only debug
///     a: :?, // This overrides both display and debug
///     a = 5, // Only overrides display
///     a:?:x = 5, // Sets debug and lower hex implementation
/// };
/// ```
///
/// Due to borrow rules you might need to create let bindings for the values,
/// i.e.
///
/// ```
/// # use interpolator::context;
/// let test = String::from("test");
/// let context = context!(test);
/// ```
///
/// instead of
///
/// ```compile_fail
/// # use interpolator::context;
/// let context = context!(test = String::from("test"));
/// ```
#[macro_export]
macro_rules! context {
    ($($tts:tt)*) => {
        {
            #[allow(unused)]
            let mut map = ::std::collections::HashMap::<&str, $crate::Formattable>::new();
            $crate::inner_context!(map; $($tts)*);
            map
        }
    };
}

/// Creates a list of formattable values, applying th specified traits, or
/// display if without a trait specifier.
/// ```
/// # use assert_dbg::assert_dbg;
/// use interpolator::{iformat, list};
/// // a list implementing Display
/// let list = list![: ;"test", 10, "another"];
/// # assert_dbg!(list, "[Formattable(display), Formattable(display), Formattable(display)]");
/// let list = list![;"test", 10, "another"];
/// # assert_dbg!(list, "[Formattable(display), Formattable(display), Formattable(display)]");
/// let list = list!["test", 10, "another"];
/// assert_eq!(
///     iformat!("{list:i({})(, )}", list:i).unwrap(),
///     "test, 10, another"
/// );
/// // a list implementing Display, Debug and UpperHex
/// let list = list![: :?:X; 4, 0x10, 0xa4];
/// assert_eq!(
///     iformat!("{list:i({} {:?} {:X})(, )}", list:i).unwrap(),
///     "4 4 4, 16 16 10, 164 164 A4"
/// );
/// ```
///
/// Due to borrow rules you might need to create let bindings for the items,
/// i.e.
///
/// ```
/// # use interpolator::list;
/// let test = String::from("test");
/// let list = list![test];
/// ```
///
/// instead of
///
/// ```compile_fail
/// # use interpolator::list;
/// let list = list![String::from("test")];
/// ```
#[macro_export]
#[cfg(doc)]
macro_rules! list {
    ($($(: $($trait:tt)?)*;)?($values:expr), *) => {};
}

#[cfg(not(doc))]
#[allow(missing_docs)]
#[macro_export]
macro_rules! list {
    ($($traits:ident),* : ; $($values:tt)*) => {
        $crate::list!($($traits,)* display; $($values)*)
    };
    ($($traits:ident),* :: $($values:tt)*) => {
        $crate::list!($($traits,)* display : $($values)*)
    };
    ($($traits:ident),* : : $($values:tt)*) => {
        $crate::list!($($traits,)* display : $($values)*)
    };
    ($($traits:ident),* : ? $($values:tt)*) => {
        $crate::list!($($traits,)* debug $($values)*)
    };
    ($($traits:ident),* : $trait:ident $($values:tt)*) => {
        $crate::list!($($traits,)* $trait $($values)*)
    };
    ($(;)? $($value:expr),* $(,)?) => {
        $crate::list!(:; $($value),*)
    };
    ($($traits:ident),*; $($value:expr),*) => {
        $crate::apply!([], ($($traits)*), $($value),*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! apply {
    ($previous:tt, ($($traits:tt)*),) => {
        $previous
    };
    ([$($previous:tt),*], ($($traits:tt)*), $value:expr $(, $($values:expr),*)?) => {
        $crate::apply!( [ $($previous,)* {
            #[allow(unused)]
            let mut value = $crate::Formattable::default();
            $($crate::set_trait!(value, $traits, $value);)*
            value
        }], ($($traits)*), $($($values),*)?)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! inner_context {
    ($map:ident; $ident:ident$(:)? $(, $($tts:tt)*)?) => {
        $crate::inner_context!($map, $ident, display; , $($($tts)*)?);
        $crate::inner_context!($map; $($($tts)*)?)
    };
    ($map:ident; $ident:ident$(:)? = $($tts:tt)*) => {
        $crate::inner_context!($map, $ident, display; = $($tts)*);
    };
    ($map:ident; $ident:ident : $($tts:tt)*) => {
        $crate::inner_context!($map, $ident; : $($tts)*)
    };
    ($map:ident; $ident:ident :: $($tts:tt)*) => {
        $crate::inner_context!($map, $ident; :: $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; :: $($tts:tt)*) => {
        $crate::inner_context!($map, $ident$(, $traits)*, display; : $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; : : $($tts:tt)*) => {
        $crate::inner_context!($map, $ident$(, $traits)*, display; : $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; : = $($tts:tt)*) => {
        $crate::inner_context!($map, $ident$(, $traits)*, display; => $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; : $(, $($tts:tt)*)?) => {
        $crate::inner_context!($map, $ident$(, $traits)*, display; $(, $($tts)*)?)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; :? $($tts:tt)*) => {
        $crate::inner_context!($map, $ident$(, $traits)*, ?; $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; : $trait:ident $($tts:tt)*) => {
        $crate::inner_context!($map, $ident$(, $traits)*, $trait; $($tts)*)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; $(, $($tts:tt)*)?) => {
        $crate::inner_context!($map, $ident$(, $traits)*; = $ident, $($($tts)*)?)
    };
    ($map:ident, $ident:ident$(, $traits:tt)*; = $value:expr $(, $($tts:tt)*)?) => {
        $($crate::set_trait!($map.entry(stringify!($ident)).or_default(), $traits, $value);)*
        $crate::inner_context!($map; $($($tts)*)?)
    };
    ($map:ident;) => {};
}

/// Sets the corresponding trait to the symbol on a formattable.
#[doc(hidden)]
#[macro_export]
macro_rules! set_trait {
    ($formattable:expr, ? , $value:expr) => {
        $crate::set_trait!($formattable, set_debug, $value);
    };
    ($formattable:expr,debug, $value:expr) => {
        $crate::set_trait!($formattable, set_debug, $value);
    };
    ($formattable:expr, , $value:expr) => {
        $crate::set_trait!($formattable, set_display, $value);
    };
    ($formattable:expr,display, $value:expr) => {
        $crate::set_trait!($formattable, set_display, $value);
    };
    ($formattable:expr,integer, $value:expr) => {
        $crate::set_trait!($formattable, set_integer, $value);
    };
    ($formattable:expr,float, $value:expr) => {
        $crate::set_trait!($formattable, set_float, $value);
    };
    ($formattable:expr,b, $value:expr) => {
        $crate::set_trait!($formattable, set_binary, $value);
    };
    ($formattable:expr,binary, $value:expr) => {
        $crate::set_trait!($formattable, set_binary, $value);
    };
    ($formattable:expr,e, $value:expr) => {
        $crate::set_trait!($formattable, set_lower_exp, $value);
    };
    ($formattable:expr,lower_exp, $value:expr) => {
        $crate::set_trait!($formattable, set_lower_exp, $value);
    };
    ($formattable:expr,x, $value:expr) => {
        $crate::set_trait!($formattable, set_lower_hex, $value);
    };
    ($formattable:expr,lower_hex, $value:expr) => {
        $crate::set_trait!($formattable, set_lower_hex, $value);
    };
    ($formattable:expr,o, $value:expr) => {
        $crate::set_trait!($formattable, set_octal, $value);
    };
    ($formattable:expr,octal, $value:expr) => {
        $crate::set_trait!($formattable, set_octal, $value);
    };
    ($formattable:expr,E, $value:expr) => {
        $crate::set_trait!($formattable, set_upper_exp, $value);
    };
    ($formattable:expr,upper_exp, $value:expr) => {
        $crate::set_trait!($formattable, set_upper_exp, $value);
    };
    ($formattable:expr,X, $value:expr) => {
        $crate::set_trait!($formattable, set_upper_hex, $value);
    };
    ($formattable:expr,upper_hex, $value:expr) => {
        $crate::set_trait!($formattable, set_upper_hex, $value);
    };
    ($formattable:expr,p, $value:expr) => {
        $crate::set_trait!($formattable, set_pointer, $value);
    };
    ($formattable:expr,pointer, $value:expr) => {
        $crate::set_trait!($formattable, set_pointer, $value);
    };
    ($formattable:expr,i, $value:expr) => {
        $crate::set_trait!($formattable, set_iter, $value);
    };
    ($formattable:expr,iter, $value:expr) => {
        $crate::set_trait!($formattable, set_iter, $value);
    };
    ($formattable:expr, $setter:ident, $value:expr) => {
        $formattable.$setter(&$value);
    };
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use std::f32::consts::PI;

    #[allow(unused)]
    macro_rules! assert_fmt {
        ($fmt:tt, $context:tt. $($fn:tt),+($key:tt), $expected:tt) => {
            assert_eq!(
                format!(
                    $fmt,
                    $($context.get(stringify!($key)).unwrap().$fn().unwrap()),+),
                    $expected
                );
        };
    }

    #[test]
    #[cfg(all(feature = "debug", feature = "number", feature = "pointer"))]
    fn context() {
        let pointer = &32;
        let context = context! {
            a = "",
            a:? = "",
            b:?:b = 10,
            d: :? = "hello",
        };
        assert_fmt!("{}", context.get_display(a), "");
        assert_fmt!("{:?}", context.get_debug(a), "\"\"");
        assert_fmt!("{:?}", context.get_debug(b), "10");
        assert_fmt!("{:b}", context.get_binary(b), "1010");
        assert_fmt!("{:}", context.get_display(d), "hello");
        assert_fmt!("{:?}", context.get_debug(d), "\"hello\"");
        let context = context! {
            debug1:? = "debug",
            debug2:debug = "debug",
            display1 = "display",
            display2: = "display",
            display3:display = "display",
            integer:integer = 42,
            float:float = PI,
            binary1:b = 12,
            binary2:binary = 12,
            lower_exp1:e = 0.01,
            lower_exp2:lower_exp = 0.01,
            lower_hex1:x = 0xA9,
            lower_hex2:lower_hex = 0xA9,
            octal1:o = 0o20,
            octal2:octal = 0o20,
            upper_exp1:E = 0.01,
            upper_exp2:upper_exp = 0.01,
            upper_hex1:X = 0xA9,
            upper_hex2:upper_hex = 0xA9,
            pointer1:p = pointer,
            pointer2:pointer = pointer,
            // iter:i:? = [1, 2],
        };
        assert_fmt!("{}", context.get_display(display1), "display");
        assert_fmt!("{}", context.get_display(display2), "display");
        assert_fmt!("{}", context.get_display(display3), "display");
        assert_fmt!("{:?}", context.get_debug(debug1), "\"debug\"");
        assert_fmt!("{:?}", context.get_debug(debug2), "\"debug\"");
        assert_fmt!(
            "{:b} {:?} {:} {:e} {:x} {:o} {:E} {:X}",
            context.get_binary,
            get_debug,
            get_display,
            get_lower_exp,
            get_lower_hex,
            get_octal,
            get_upper_exp,
            get_upper_hex(integer),
            "101010 42 42 4.2e1 2a 52 4.2E1 2A"
        );
        assert_fmt!(
            "{:?} {:} {:e} {:E}",
            context.get_debug,
            get_display,
            get_lower_exp,
            get_upper_exp(float),
            "3.1415927 3.1415927 3.1415927e0 3.1415927E0"
        );
        assert_fmt!("{:b}", context.get_binary(binary1), "1100");
        assert_fmt!("{:b}", context.get_binary(binary2), "1100");
        assert_fmt!("{:e}", context.get_lower_exp(lower_exp1), "1e-2");
        assert_fmt!("{:e}", context.get_lower_exp(lower_exp2), "1e-2");
        assert_fmt!("{:x}", context.get_lower_hex(lower_hex1), "a9");
        assert_fmt!("{:x}", context.get_lower_hex(lower_hex2), "a9");
        assert_fmt!("{:o}", context.get_octal(octal1), "20");
        assert_fmt!("{:o}", context.get_octal(octal2), "20");
        assert_fmt!("{:E}", context.get_upper_exp(upper_exp1), "1E-2");
        assert_fmt!("{:E}", context.get_upper_exp(upper_exp2), "1E-2");
        assert_fmt!("{:X}", context.get_upper_hex(upper_hex1), "A9");
        assert_fmt!("{:X}", context.get_upper_hex(upper_hex2), "A9");
        assert_fmt!(
            "{:p}",
            context.get_pointer(pointer1),
            (format!("{pointer:p}"))
        );
        assert_fmt!(
            "{:p}",
            context.get_pointer(pointer2),
            (format!("{pointer:p}"))
        );
    }
    #[allow(unused)]
    macro_rules! assert_i {
        ($fmt:literal, $variable:ident, $rhs:literal) => {
            assert_eq!(
                iformat!(concat!("{list:i(", $fmt, ")(,)}"), $variable: i).unwrap(),
                $rhs
            );
        };
    }

    #[test]
    #[cfg(feature = "iter")]
    fn list() {
        let list = list!["a", 1];
        assert_i!("{}", list, "a,1");
        #[cfg(feature = "debug")]
        {
            let list = list![:?; "a", 1];
            assert_i!("{:?}", list, "\"a\",1");
            #[cfg(feature = "number")]
            {
                let list = list![::?:X; 2, 1];
                assert_i!("{}{:?}{:X}", list, "222,111");
            }
        }
    }
}
