#[macro_export]
macro_rules! newtype_id {
    ($(#[$meta:meta])* $vis:vis $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $name(::uuid::Uuid);

        impl $name {
            #[must_use]
            pub fn random() -> Self {
                ::rand::random()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl ::rand::prelude::Distribution<$name> for ::rand::distributions::Standard {
             fn sample<R: ::rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                 $name(::uuid::Uuid::from_bytes(rng.gen()))
             }
        }
    };
}

#[macro_export]
macro_rules! newtype {
    ($(#[$meta:meta])* $vis:vis $name:ident: $ty:ty) => {
        $(#[$meta])*
        $vis struct $name($ty);

        impl $name {
            pub fn new(inner: $ty) -> Self {
                Self(inner)
            }

            pub fn into_inner(self) -> $ty {
                self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> std::convert::From<T> for $name
        where
            $ty: From<T>,
        {
            fn from(other: T) -> Self {
                Self(other.into())
            }
        }
    };
}

#[macro_export]
macro_rules! amqp_todo {
    () => {
        return Err(
            ::amqp_core::error::ConException::NotImplemented(concat!(file!(), ":", line!())).into(),
        )
    };
}
