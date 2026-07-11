/// Declares a copyable enum whose kebab-case label is the single source of
/// truth for display and serialization.
macro_rules! labeled_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $label:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis enum $name {
            $($variant),+
        }

        impl $name {
            #[must_use]
            $vis const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.label())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.label())
            }
        }
    };
}

pub(crate) use labeled_enum;
