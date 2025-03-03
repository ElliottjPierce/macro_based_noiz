//! This lets us name a particular array and its indices.

pub use flagset::*;

/// A series of utils to make bitwise manipulation of flagsets more readable
pub trait FlagSetUtils: Sized {
    /// The underlying flag type
    type F: Flags;

    /// forces each of the given `flags` on
    fn set_flags_on(&mut self, flags: impl Into<Self>);

    /// forces each of the given `flags` off
    fn set_flags_off(&mut self, flags: impl Into<Self>);

    /// Returns true if there is any overlap between the flags
    fn has_any(&self, flags: impl Into<Self>) -> bool;

    /// Sets all `flags` to `value`
    #[inline]
    fn set_flags_value(&mut self, flags: impl Into<Self>, value: bool) {
        if value {
            self.set_flags_on(flags);
        } else {
            self.set_flags_off(flags);
        }
    }
}

impl<T: Flags> FlagSetUtils for FlagSet<T> {
    type F = T;

    #[inline]
    fn set_flags_on(&mut self, flags: impl Into<Self>) {
        *self |= flags.into();
    }

    #[inline]
    fn set_flags_off(&mut self, flags: impl Into<Self>) {
        *self &= !(flags.into());
    }

    /// The opposite of is_disjoint
    #[inline]
    fn has_any(&self, flags: impl Into<Self>) -> bool {
        !self.is_disjoint(flags)
    }
}

/// Represents the special indices of a named array
pub trait NamedArrayIndices: 'static + Sized {
    /// The inner type of the index
    type Inner;

    /// An Identity slice mapping an index to its name
    const INDEX_TO_NAME: &'static [Self];
    /// The length of this named array
    const LEN: usize;
    /// The maximum array index
    const MAX: Self::Inner;

    /// Attempty to convert an index to its name
    fn try_from_index(index: Self::Inner) -> Option<Self>;

    /// forces a name from an index. If the index is invalid, it will be the `MAX` index's name
    fn force_from_index(index: Self::Inner) -> Self;

    /// Creates a new values from a known valid index.
    ///
    /// # Safety
    ///
    /// The index must be valid.
    unsafe fn from_index(index: Self::Inner) -> Self;

    /// return the index of a given name
    fn get_index(self) -> Self::Inner;
}

/// creates an array with special meaning.
///
/// For example:
/// ```
/// name_array!{
///     visibility struct MyNamedArray,
///     visibility enum MyIndexNames: prepresentation_of_indexes (usually u8), type_of_flagset (optional) {
///         IndexNameOne,
///         IndexNameTwo,
///         IndexNameThree,
///     }
/// }
/// ```
#[macro_export]
macro_rules! name_array {
    () => {};

    ($c:ident, $i:ident: $t:ty { $($(#[$km:meta])*$k:ident),+ $(,)* } $($next:tt)*) => {
        impl $crate::spatial::named_array::NamedArrayIndices for $i {
            const INDEX_TO_NAME: &'static [Self] = &[$(Self::$k),+];
            const LEN: usize = Self::INDEX_TO_NAME.len();
            const MAX: $t = Self::LEN as $t - 1;
            type Inner = $t;

            #[inline]
            fn try_from_index(index: Self::Inner) -> Option<Self> {
                if index <= Self::MAX {
                    // SAFETY: the index is valid, and the enum is just tags, no data.
                    Some(unsafe { std::mem::transmute::<$t, Self>(index) })
                } else {
                    None
                }
            }

            #[inline]
            fn force_from_index(index: Self::Inner) -> Self {
                let index = index.min(Self::MAX);
                // SAFETY: the index is clamped to be valid, and the enum is just tags, no data.
                unsafe { std::mem::transmute(index) }
            }

            #[inline]
            unsafe fn from_index(index: Self::Inner) -> Self {
                debug_assert!(index <= Self::MAX, "Invalid from index enum conversion. Index requested: {index}, max: {}", Self::MAX);
                // SAFETY: caller ensures index is valid, and the enum is just tags, no data.
                unsafe { std::mem::transmute(index) }
            }

            #[inline]
            fn get_index(self) -> Self::Inner {
                self as $t
            }
        }

        impl $i {
            /// The identity. Similar to `Self::NamedArrayIndices::INDEX_TO_NAME` but as the special array.
            pub const IDENTITY: $c<Self> = $c([$(Self::$k),+]);
            /// The inner identity. Similar `Self::IDENTITY` but with the indices instead of the names
            pub const INNER_IDENTITY: $c<$t> = $c([$(Self::$k as $t),+]);
        }

        impl<T> std::ops::Index<$i> for $c<T> {
            type Output = T;

            #[inline]
            fn index(&self, index: $i) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl<T> std::ops::IndexMut<$i> for $c<T> {
            #[inline]
            fn index_mut(&mut self, index: $i) -> &mut Self::Output {
                &mut self.0[index as usize]
            }
        }

        impl<T> std::iter::IntoIterator for $c<T> {
            type Item = T;

            type IntoIter = <[T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN] as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<T> $c<T> {
            /// performs a map of self using `f`
            #[inline]
            pub fn map<O>(self, f: impl FnMut(T) -> O) -> $c<O> {
                $c::<O>(self.0.map(f))
            }

            /// converts to references
            #[inline]
            pub fn each_ref(&self) -> $c<&T> {
                $c(self.0.each_ref())
            }


            /// converts to mutable references
            #[inline]
            pub fn each_mut(&mut self) -> $c<&mut T> {
                $c(self.0.each_mut())
            }
        }

        impl<T> From<[T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN]> for $c<T> {
            #[inline]
            fn from(value: [T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN]) -> Self {
                Self(value)
            }
        }

        impl<T> From<$c<T>> for [T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN] {
            #[inline]
            fn from(value: $c<T>) -> Self {
                value.0
            }
        }

        $crate::name_array!($($next)*);
    };

    ($(#[$cm:meta])* $cp:vis struct $c:ident, $(#[$im:meta])* $np:vis enum $i:ident: $t:ty { $($(#[$km:meta])*$k:ident),+ $(,)* } $($next:tt)*) => {
        $(#[$im])*
        #[repr($t)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $np enum $i { $($(#[$km])* $k),+ }

        $(#[$cm])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
        $cp struct $c<T>(pub [T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN]);

        $crate::name_array! {$c, $i: $t { $($(#[$km])*$k),+ } $($next)*}
    };

    ($(#[$cm:meta])* $cp:vis struct $c:ident, $(#[$im:meta])* $np:vis enum $i:ident: $t:ty, $mt:ty { $($(#[$km:meta])*$k:ident),+ $(,)* }  $($next:tt)*) => {
        $crate::spatial::named_array::flags!{
            $(#[$im])*
            #[repr($t)]
            #[derive(PartialOrd, Ord, Hash)]
            $np enum $i: $mt { $($(#[$km])* $k),+ }
        }

        $(#[$cm])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
        $cp struct $c<T>(pub [T; <$i as $crate::spatial::named_array::NamedArrayIndices>::LEN]);

        $crate::name_array! {$c, $i: $t { $($(#[$km])*$k),+ } $($next)*}
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    name_array! {
        pub struct TestCollection,
        pub enum TestIndices: u8, u32 {
            A,
            B,
            C,
            D,
            E,
            F,
            G,
            H,
            I,
            J,
            K,
            L,
            M,
            N,
            O,
            P,
            Q,
            R,
            S,
            T,
            U,
            V,
            W,
            X,
            Y,
            Z,
        }
    }

    #[test]
    fn test_indices() {
        for index in TestIndices::INDEX_TO_NAME {
            let inner = index.get_index();
            assert_eq!(Some(*index), TestIndices::try_from_index(inner));
            assert_eq!(*index, TestIndices::force_from_index(inner));
        }
        let invalid = TestIndices::MAX + 1;
        assert_eq!(None, TestIndices::try_from_index(invalid));
        assert_eq!(TestIndices::Z, TestIndices::force_from_index(invalid));
    }

    #[test]
    fn test_lens() {
        assert_eq!(
            TestCollection([0u8; TestIndices::LEN]).map(|x| x + 1),
            TestCollection([1u8; TestIndices::LEN])
        );
    }

    #[test]
    fn test_identities() {
        for index in TestIndices::IDENTITY {
            assert_eq!(
                index,
                TestIndices::INDEX_TO_NAME[index.get_index() as usize]
            );
            assert_eq!(index.get_index(), TestIndices::INNER_IDENTITY[index]);
        }
    }

    #[test]
    fn test_mask_conversions() {
        let mut flags = FlagSet::default();
        for index in TestIndices::IDENTITY {
            flags.set_flags_on(index);
        }
        assert!(flags.is_full());
    }
}
