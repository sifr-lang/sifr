use super::*;
/// Utility struct holding references to the trait implementation of a value to
/// enable runtime verification and execution of them
#[derive(Default, Clone, Copy)]
#[must_use]
pub struct Formattable<'a> {
    #[cfg(any(feature = "debug", feature = "number"))]
    debug: Option<&'a dyn Debug>,
    display: Option<&'a dyn Display>,
    #[cfg(feature = "number")]
    binary: Option<&'a dyn Binary>,
    #[cfg(feature = "number")]
    lower_exp: Option<&'a dyn LowerExp>,
    #[cfg(feature = "number")]
    lower_hex: Option<&'a dyn LowerHex>,
    #[cfg(feature = "number")]
    octal: Option<&'a dyn Octal>,
    #[cfg(feature = "number")]
    upper_exp: Option<&'a dyn UpperExp>,
    #[cfg(feature = "number")]
    upper_hex: Option<&'a dyn UpperHex>,
    #[cfg(feature = "pointer")]
    pointer: Option<PointerWrapper<'a>>,
    #[cfg(feature = "iter")]
    iter: Option<&'a [Formattable<'a>]>,
}

struct PlainString(&'static str);

impl Debug for PlainString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! formattable_debug {
    ($($field:ident $(: $feature:literal)?),+ $(,)?) => {
        impl Debug for Formattable<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut tuple = f.debug_tuple("Formattable");
                $(
                    $(#[cfg(feature = $feature)])?
                    if self.$field.is_some() {
                        tuple.field(&PlainString(stringify!($field)));
                    }
                )*
                tuple.finish()
            }
        }
    };
}

formattable_debug! {
    debug: "debug",
    display,
    binary: "number",
    lower_exp: "number",
    lower_hex: "number",
    octal: "number",
    upper_exp: "number",
    upper_hex: "number",
    pointer: "pointer",
    iter: "iter"
}

macro_rules! formattable_fn {
    (($($cfg:tt)*), ($doc:expr), $name:ident, $builder:ident, $setter:ident <$($traits:ident),*> $($fields:ident),+) => {
        /// Creates a [`Formattable`] from a value implementing
        #[doc = $doc]
        $($cfg)* pub fn $name<T: $($traits+)*>(value: &'a T) -> Self {
            #[allow(clippy::needless_update)]
            Self {
                $($fields: Some(value),)*
                ..Default::default()
            }
        }
        /// Adds implementation for
        #[doc = $doc]
        $($cfg)* pub fn $builder<T: $($traits+)*>(mut self, value: &'a T) -> Self {
            $(self.$fields = Some(value);)*
            self
        }
        /// Sets implementation for
        #[doc = $doc]
        $($cfg)* pub fn $setter<T: $($traits+)*>(&mut self, value: &'a T) {
            $(self.$fields = Some(value);)*
        }
    };
    (($($cfg:tt)*), (), $name:ident, $builder:ident, $setter:ident, $getter:ident<$trait:ident>) => {
        formattable_fn!(($($cfg)*), (concat!("[`",stringify!($trait),"`]")), $name, $builder, $setter<$trait> $name);
        $($cfg)* pub(crate) fn $getter(&self) -> Result<&dyn $trait, Trait> {
            self.$name.ok_or(Trait::$trait)
        }
    };
}
macro_rules! formattable {
    [$($($cfg:literal, $($doc:literal,)?)? $name:ident, $builder:ident, $setter:ident $(, $getter:ident)?<$($traits:ident),*> $($fields:ident),*;)*] => {
        impl<'a> Formattable<'a> {
            $(formattable_fn!(($(#[cfg(feature=$cfg)])?), ($($($doc)?)?), $name, $builder, $setter $(, $getter)?<$($traits),*> $($fields),*);)*
        }
    };
}

formattable![
    "debug", "[`Debug`] and [`Display`]", debug_display, and_debug_display, set_debug_display<Debug, Display> debug, display;
    "debug", debug, and_debug, set_debug, get_debug<Debug>;
    display, and_display, set_display, get_display<Display>;
    "number", "[`Debug`], [`Display`], [`Octal`], [`LowerHex`], [`UpperHex`], [`Binary`], [`LowerExp`] and [`UpperExp`]", integer, and_integer, set_integer<Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex>
        binary, debug, display, lower_exp, lower_hex, octal, upper_exp, upper_hex;
    "number", "[`Debug`], [`Display`], [`LowerExp`] and [`UpperExp`]", float, and_float, set_float<Debug, Display, LowerExp, UpperExp>
        debug, display, lower_exp, upper_exp;
    "number", binary, and_binary, set_binary, get_binary<Binary>;
    "number", lower_exp, and_lower_exp, set_lower_exp, get_lower_exp<LowerExp>;
    "number", lower_hex, and_lower_hex, set_lower_hex, get_lower_hex<LowerHex>;
    "number", octal, and_octal, set_octal, get_octal<Octal>;
    "number", upper_exp, and_upper_exp, set_upper_exp, get_upper_exp<UpperExp>;
    "number", upper_hex, and_upper_hex, set_upper_hex, get_upper_hex<UpperHex>;
];

#[cfg(feature = "pointer")]
impl<'a> Formattable<'a> {
    /// Creates a [`Formattable`] from a value implementing [`Pointer`].
    pub fn pointer<T: Pointer>(value: &'a T) -> Self {
        Self::default().and_pointer(value)
    }

    /// Adds implementation for [`Pointer`]
    pub fn and_pointer<T: Pointer>(mut self, value: &'a T) -> Self {
        self.pointer = Some(PointerWrapper(value));
        self
    }

    /// Sets implementation for [`Pointer`]
    pub fn set_pointer<T: Pointer>(&mut self, value: &'a T) {
        self.pointer = Some(PointerWrapper(value));
    }

    pub(crate) fn get_pointer(&self) -> Result<PointerWrapper, Trait> {
        self.pointer.ok_or(Trait::Pointer)
    }
}

#[cfg(feature = "iter")]
impl<'a> Formattable<'a> {
    /// Creates a [`Formattable`] from a list of values
    pub fn iter(value: &'a [Formattable<'a>]) -> Self {
        Self::default().and_iter(value)
    }

    /// Adds implementation for mapping operations
    pub fn and_iter(mut self, value: &'a [Formattable<'a>]) -> Self {
        self.iter = Some(value);
        self
    }

    /// Sets implementation for mapping operations
    pub fn set_iter(&mut self, value: &'a [Formattable<'a>]) {
        self.iter = Some(value);
    }

    pub(crate) fn get_iter(&self) -> Result<&'a [Formattable<'a>], Trait> {
        self.iter.ok_or(Trait::Iter)
    }
}

#[cfg(feature = "pointer")]
#[derive(Clone, Copy)]
pub(crate) struct PointerWrapper<'a>(&'a dyn Pointer);

#[cfg(feature = "pointer")]
impl Pointer for PointerWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pointer::fmt(self.0, f)
    }
}

#[cfg(all(feature = "display", feature = "debug"))]
impl<'a, T: Display + Debug> From<&'a T> for Formattable<'a> {
    fn from(value: &'a T) -> Self {
        Self::debug_display(value)
    }
}
