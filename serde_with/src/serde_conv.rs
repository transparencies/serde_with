/// Create new conversion adapters from functions
///
/// The macro lets you create a new converter, which is usable for serde's with-attribute and `#[serde_as]`.
/// Its main use case is to write simple converters for types, which are not serializable.
/// Another use-case is to change the serialization behavior if the implemented `Serialize`/`Deserialize` trait is insufficient.
///
/// The macro takes four arguments:
///
/// 1. The name of the converter type.
///    The type can be prefixed with a visibility modifies like `pub` or `pub(crate)`.
///    By default, the type is not marked as public (`pub(self)`).
/// 2. The type `T` we want to extend with custom behavior.
/// 3. A function or macro taking a `&T` and returning a serializable type.
/// 4. A function or macro taking a deserializable type and returning a `Result<T, E>`.
///    The error type `E` must implement [`Display`].
///
/// [`Display`]: std::fmt::Display
///
/// # Example
///
/// In this example, we write custom serialization behavior for a `Rgb` type.
/// We want to serialize it as a `[u8; 3]`.
///
/// ```rust
/// # #[cfg(feature = "macros")] {
/// # use serde::{Serialize, Deserialize};
/// # use serde_with::serde_as;
///
/// #[derive(Clone, Copy, Debug, PartialEq)]
/// struct Rgb {
///     red: u8,
///     green: u8,
///     blue: u8,
/// }
///
/// serde_with::serde_conv!(
///     RgbAsArray,
///     Rgb,
///     |rgb: &Rgb| [rgb.red, rgb.green, rgb.blue],
///     |value: [u8; 3]| -> Result<_, std::convert::Infallible> {
///         Ok(Rgb {
///             red: value[0],
///             green: value[1],
///             blue: value[2],
///         })
///     }
/// );
///
/// //////////////////////////////////////////////////
///
/// // We define some colors to be used later
///
/// let green = Rgb {red: 0, green: 255, blue: 0};
/// let orange = Rgb {red: 255, green: 128, blue: 0};
/// let pink = Rgb {red: 255, green: 0, blue: 255};
///
/// //////////////////////////////////////////////////
///
/// // We can now use the `RgbAsArray` adapter with `serde_as`.
///
/// #[serde_as]
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Colors {
///     #[serde_as(as = "RgbAsArray")]
///     one_rgb: Rgb,
///     #[serde_as(as = "Vec<RgbAsArray>")]
///     rgbs_in_vec: Vec<Rgb>,
/// }
///
/// let data = Colors {
///     one_rgb: orange,
///     rgbs_in_vec: vec![green, pink],
/// };
/// let json = serde_json::json!({
///     "one_rgb": [255, 128, 0],
///     "rgbs_in_vec": [
///         [0, 255, 0],
///         [255, 0, 255]
///     ]
/// });
///
/// assert_eq!(json, serde_json::to_value(&data).unwrap());
/// assert_eq!(data, serde_json::from_value(json).unwrap());
///
/// //////////////////////////////////////////////////
///
/// // The types generated by `serde_conv` is also compatible with serde's with attribute
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct ColorsWith {
///     #[serde(with = "RgbAsArray")]
///     rgb_with: Rgb,
/// }
///
/// let data = ColorsWith {
///     rgb_with: pink,
/// };
/// let json = serde_json::json!({
///     "rgb_with": [255, 0, 255]
/// });
///
/// assert_eq!(json, serde_json::to_value(&data).unwrap());
/// assert_eq!(data, serde_json::from_value(json).unwrap());
/// # }
/// ```
#[macro_export]
macro_rules! serde_conv {
    ($m:ident, $t:ty, $ser:expr, $de:expr) => {$crate::serde_conv!(pub(self) $m, $t, $ser, $de);};
    ($vis:vis $m:ident, $t:ty, $ser:expr, $de:expr) => {
        #[allow(non_camel_case_types)]
        $vis struct $m;

        // Prevent clippy lints triggering because of the template here
        // https://github.com/jonasbb/serde_with/pull/320
        // https://github.com/jonasbb/serde_with/pull/729
        #[allow(clippy::all)]
        const _:() = {
            impl $m {
                $vis fn serialize<S>(x: &$t, serializer: S) -> $crate::__private__::Result<S::Ok, S::Error>
                where
                    S: $crate::serde::Serializer,
                {
                    let y = $ser(x);
                    $crate::serde::Serialize::serialize(&y, serializer)
                }

                $vis fn deserialize<'de, D>(deserializer: D) -> $crate::__private__::Result<$t, D::Error>
                where
                    D: $crate::serde::Deserializer<'de>,
                {
                    let y = $crate::serde::Deserialize::deserialize(deserializer)?;
                    $de(y).map_err($crate::serde::de::Error::custom)
                }
            }

            impl $crate::SerializeAs<$t> for $m {
                fn serialize_as<S>(x: &$t, serializer: S) -> $crate::__private__::Result<S::Ok, S::Error>
                where
                    S: $crate::serde::Serializer,
                {
                    Self::serialize(x, serializer)
                }
            }

            impl<'de> $crate::DeserializeAs<'de, $t> for $m {
                fn deserialize_as<D>(deserializer: D) -> $crate::__private__::Result<$t, D::Error>
                where
                    D: $crate::serde::Deserializer<'de>,
                {
                    Self::deserialize(deserializer)
                }
            }
        };
    };
}
