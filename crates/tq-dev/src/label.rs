/// Declares a copyable enum with stable display and serialization labels.
macro_rules! labeled_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $label:literal),+ $(,)?
        }
    ) => {
        labeled_enum! {
            @impl
            $(#[$meta])*
            $vis enum $name {
                $($variant => ($label, $label)),+
            }
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => ($value:literal, $label:literal)),+ $(,)?
        }
    ) => {
        labeled_enum! {
            @impl
            $(#[$meta])*
            $vis enum $name {
                $($variant => ($value, $label)),+
            }
        }
    };
    (
        @impl
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => ($value:literal, $label:literal)),+ $(,)?
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
                let value = match self {
                    $(Self::$variant => $value),+
                };
                serializer.serialize_str(value)
            }
        }
    };
}

pub(crate) use labeled_enum;
