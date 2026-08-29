use crate::SifrInt;

mod sealed {
    pub trait Sealed {}
}

/// Values that generated code can narrow to `usize` after proving the bounds.
pub trait ProvenUsize: sealed::Sealed {
    fn to_usize_proven(&self) -> usize;
}

#[must_use]
pub fn to_usize_proven<T: ProvenUsize + ?Sized>(value: &T) -> usize {
    value.to_usize_proven()
}

impl sealed::Sealed for SifrInt {}
impl ProvenUsize for SifrInt {
    fn to_usize_proven(&self) -> usize {
        self.to_usize_proven_in_bounds()
    }
}

impl<T: ProvenUsize + ?Sized> sealed::Sealed for &T {}
impl<T: ProvenUsize + ?Sized> ProvenUsize for &T {
    fn to_usize_proven(&self) -> usize {
        (*self).to_usize_proven()
    }
}

macro_rules! impl_proven_usize {
    ($($source:ty),* $(,)?) => {
        $(
            impl sealed::Sealed for $source {}
            impl ProvenUsize for $source {
                fn to_usize_proven(&self) -> usize {
                    match usize::try_from(*self) {
                        Ok(value) => value,
                        Err(_) => panic!("compiler usize bound proof was invalid"),
                    }
                }
            }
        )*
    };
}

impl_proven_usize!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
