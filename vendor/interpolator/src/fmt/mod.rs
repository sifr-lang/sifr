#![allow(non_snake_case)]
use super::*;

macro_rules! format_value {
    ($name:ident($out:ident, $v:ident, $Trait:ident): $($tts:tt)*) => {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn $name(
            $out: &mut impl std::fmt::Write,
            $v: impl std::fmt::$Trait,
            width: Option<usize>,
            precision: Option<usize>,
            alignment: crate::Alignment,
            sign: crate::Sign,
            hash: bool,
            zero: bool,
        ) -> crate::Result<(), std::fmt::Error> {
            use {crate::Alignment as A, crate::Sign as S};

            match (width, precision, alignment, sign, hash, zero) {
                $($tts)*
            }
        }
    };
}

macro_rules! branch {
    (($_:ident), $($tts:tt)*) => {
        $($tts)*
    };
    (($idx:ident return), $($tts:tt)*) => {
        return $($tts)*.map_err(|e|e.add_idx($idx))
    };
}

macro_rules! call_format_value {
    (
        match
        $out:ident,
        $value:ident,
        $width:ident,
        $precision:ident,
        $alignment:ident,
        $sign:ident,
        $hash:ident,
        $zero:ident,
        $trait_:ident,
        $idx:ident {
            $($Trait:ident$(($($fields:tt)*))? => $fn:ident($getter:ident $(, $feature:literal)? $(, $args:ident)*) $($ret:ident)?,)*
        }
    ) => {
        match $trait_ {
            $($(#[cfg(feature = $feature)])? TraitSpec::$Trait$(($($fields)*))? => {
                let value = match $value.$getter() {
                    Ok(v) => v,
                    Err(e) => return Err(Error::MissingTraitImpl(e, $idx))
                };
                branch!(($idx $($ret)?),$fn::$fn($out, value, $width, $precision, $alignment, $sign, $hash, $zero, $($($fields)*)? $(, $args)*))
            },)*
            TraitSpec::Phantom(_) => unreachable!()
        }
    };
}

#[cfg(feature = "number")]
mod binary;
#[cfg(feature = "debug")]
mod debug;
mod display;
#[cfg(feature = "iter")]
mod iter;
#[cfg(feature = "number")]
mod lower_exp;
#[cfg(feature = "number")]
mod lower_hex;
#[cfg(feature = "debug")]
mod lower_hex_question;
#[cfg(feature = "number")]
mod octal;
#[cfg(feature = "pointer")]
mod pointer;
#[cfg(feature = "number")]
mod upper_exp;
#[cfg(feature = "number")]
mod upper_hex;
#[cfg(feature = "debug")]
mod upper_hex_debug;

#[allow(clippy::too_many_arguments)]
pub(crate) fn format_value(
    out: &mut impl Write,
    value: &Formattable,
    width: Option<usize>,
    precision: Option<usize>,
    alignment: Alignment,
    sign: Sign,
    hash: bool,
    zero: bool,
    trait_: TraitSpec,
    idx: usize,
    #[allow(unused)] context: &impl Context,
) -> Result<()> {
    call_format_value! {
        match out, value, width, precision, alignment, sign, hash, zero, trait_, idx {
            Display => display(get_display),
            Question => debug(get_debug, "debug"),
            LowerHexQuestion => lower_hex_question(get_debug, "debug"),
            UpperHexQuestion => upper_hex_debug(get_debug, "debug"),
            LowerHex => lower_hex(get_lower_hex, "number"),
            UpperHex => upper_hex(get_upper_hex, "number"),
            Binary => binary(get_binary, "number"),
            Octal => octal(get_octal, "number"),
            LowerExp => lower_exp(get_lower_exp, "number"),
            UpperExp => upper_exp(get_upper_exp, "number"),
            Pointer => pointer(get_pointer, "pointer"),
            Iter(range, format, join) => iter(get_iter, "iter", context) return,
        }
    }
    .map_err(|e| Error::Fmt(e, idx))
}
