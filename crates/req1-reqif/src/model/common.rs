/// ISO 8601 date-time string, e.g. `"2024-01-15T10:30:00+01:00"`.
/// Kept as `String` to avoid lossy parsing; the mapping layer converts when needed.
pub type ReqifDateTime = String;

/// Defines a struct with the four standard identifiable fields shared by nearly
/// all `ReqIF` elements (`IDENTIFIER`, `LONG-NAME`, `LAST-CHANGE`, `DESC`),
/// plus any extra fields. Extra fields should omit `pub` — the macro adds it.
macro_rules! identifiable_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty,
            )*
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            #[serde(rename = "@IDENTIFIER")]
            pub identifier: String,

            #[serde(rename = "@LONG-NAME", skip_serializing_if = "Option::is_none", default)]
            pub long_name: Option<String>,

            #[serde(rename = "@LAST-CHANGE", skip_serializing_if = "Option::is_none", default)]
            pub last_change: Option<$crate::model::common::ReqifDateTime>,

            #[serde(rename = "@DESC", skip_serializing_if = "Option::is_none", default)]
            pub desc: Option<String>,

            $(
                $(#[$field_meta])*
                pub $field : $ty,
            )*
        }
    };
}

pub(crate) use identifiable_struct;
