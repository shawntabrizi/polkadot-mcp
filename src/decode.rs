#![allow(dead_code)]
//! Dynamic value decoding: convert subxt's DecodedValueThunk into human-readable output.
//!
//! This is the hardest module in the codebase. subxt dynamic mode returns
//! `scale_value::Value` types that need to be navigated and formatted
//! contextually (balances as DOT, block numbers as time estimates, etc.).

use scale_info::form::PortableForm;
use scale_info::TypeDef;
use subxt::dynamic::{At, DecodedValue};
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};

use crate::network::ChainConfig;
use crate::types::format_balance;

// ---------------------------------------------------------------------------
// Value helpers (extract typed data from DecodedValue)
// ---------------------------------------------------------------------------

/// Extract a u128 from a dynamic value, returning 0 if not found.
pub fn value_as_u128(value: &DecodedValue) -> u128 {
    value.as_u128().unwrap_or(0)
}

/// Extract a string from a dynamic value.
pub fn value_as_string(value: &DecodedValue) -> Option<String> {
    value.as_str().map(|s| s.to_string())
}

/// Check if a dynamic value is a specific enum variant.
pub fn is_variant(value: &DecodedValue, variant_name: &str) -> bool {
    match &value.value {
        ValueDef::Variant(v) => v.name == variant_name,
        _ => false,
    }
}

/// Get the variant name and fields from a dynamic value.
pub fn as_variant(value: &DecodedValue) -> Option<(&str, &Composite<u32>)> {
    match &value.value {
        ValueDef::Variant(v) => Some((&v.name, &v.values)),
        _ => None,
    }
}

/// Format a balance field from a dynamic value using chain config.
pub fn format_balance_field(
    value: &DecodedValue,
    field_name: &str,
    config: &ChainConfig,
) -> String {
    let planck = value.at(field_name).map(value_as_u128).unwrap_or(0);
    format_balance(planck, config.token_decimals, &config.token_symbol)
}

// ---------------------------------------------------------------------------
// format_value: render a DecodedValue as a human-readable string
// ---------------------------------------------------------------------------

/// Format a dynamic value as a human-readable string.
/// This is the generic formatter — domain-specific tools can format values
/// differently (e.g., rendering balances as "10.5 DOT" instead of raw planck).
pub fn format_value(value: &DecodedValue) -> String {
    format_value_inner(value, 0)
}

fn format_value_inner(value: &DecodedValue, depth: usize) -> String {
    if depth > 8 {
        return "...".to_string();
    }

    match &value.value {
        ValueDef::Primitive(p) => match p {
            Primitive::Bool(b) => b.to_string(),
            Primitive::Char(c) => c.to_string(),
            Primitive::String(s) => s.clone(),
            Primitive::U128(n) => n.to_string(),
            Primitive::I128(n) => n.to_string(),
            Primitive::U256(bytes) => format!("0x{}", hex::encode(bytes)),
            Primitive::I256(bytes) => format!("0x{}", hex::encode(bytes)),
        },
        ValueDef::Composite(composite) => match composite {
            Composite::Named(fields) => {
                let formatted: Vec<String> = fields
                    .iter()
                    .map(|(name, val)| {
                        format!("{}: {}", name, format_value_inner(val, depth + 1))
                    })
                    .collect();
                format!("{{ {} }}", formatted.join(", "))
            }
            Composite::Unnamed(fields) => {
                if fields.len() == 1 {
                    format_value_inner(&fields[0], depth + 1)
                } else {
                    let formatted: Vec<String> = fields
                        .iter()
                        .map(|v| format_value_inner(v, depth + 1))
                        .collect();
                    format!("({})", formatted.join(", "))
                }
            }
        },
        ValueDef::Variant(v) => {
            let inner = match &v.values {
                Composite::Named(fields) if fields.is_empty() => String::new(),
                Composite::Unnamed(fields) if fields.is_empty() => String::new(),
                Composite::Named(fields) => {
                    let formatted: Vec<String> = fields
                        .iter()
                        .map(|(name, val)| {
                            format!("{}: {}", name, format_value_inner(val, depth + 1))
                        })
                        .collect();
                    format!(" {{ {} }}", formatted.join(", "))
                }
                Composite::Unnamed(fields) => {
                    let formatted: Vec<String> = fields
                        .iter()
                        .map(|v| format_value_inner(v, depth + 1))
                        .collect();
                    format!("({})", formatted.join(", "))
                }
            };
            format!("{}{}", v.name, inner)
        }
        ValueDef::BitSequence(_) => "BitVec<...>".to_string(),
    }
}

// ---------------------------------------------------------------------------
// type_to_string: render a scale-info type ID as a readable type name
// ---------------------------------------------------------------------------

/// Render a type ID from the metadata type registry as a human-readable string.
/// E.g., `"AccountId32"`, `"Vec<BalanceLock>"`, `"Option<u128>"`.
pub fn type_to_string(
    type_id: u32,
    types: &scale_info::PortableRegistry,
) -> String {
    type_to_string_inner(type_id, types, 0)
}

fn type_to_string_inner(
    type_id: u32,
    types: &scale_info::PortableRegistry,
    depth: usize,
) -> String {
    if depth > 6 {
        return "...".to_string();
    }

    let ty = match types.resolve(type_id) {
        Some(t) => t,
        None => return format!("Type({})", type_id),
    };

    let path = &ty.path.segments;

    match &ty.type_def {
        TypeDef::Primitive(p) => format_primitive(p),

        TypeDef::Composite(_c) => {
            // Named type (struct) — use its short name
            if !path.is_empty() {
                path.last().unwrap().to_string()
            } else {
                // Anonymous tuple struct
                format_composite_fields(_c, types, depth)
            }
        }

        TypeDef::Variant(_v) => {
            let name = path.last().map(|s| s.as_str()).unwrap_or("Enum");

            // Special handling for generic wrapper types
            match name {
                "Option" => {
                    if let Some(inner) = find_variant_field(_v, "Some", types, depth) {
                        format!("Option<{}>", inner)
                    } else {
                        "Option<?>".to_string()
                    }
                }
                "Result" => {
                    let ok = find_variant_field(_v, "Ok", types, depth)
                        .unwrap_or_else(|| "?".to_string());
                    let err = find_variant_field(_v, "Err", types, depth)
                        .unwrap_or_else(|| "?".to_string());
                    format!("Result<{}, {}>", ok, err)
                }
                _ => name.to_string(),
            }
        }

        TypeDef::Sequence(s) => {
            format!(
                "Vec<{}>",
                type_to_string_inner(s.type_param.id, types, depth + 1)
            )
        }

        TypeDef::Array(a) => {
            format!(
                "[{}; {}]",
                type_to_string_inner(a.type_param.id, types, depth + 1),
                a.len
            )
        }

        TypeDef::Tuple(t) => {
            if t.fields.is_empty() {
                "()".to_string()
            } else {
                let fields: Vec<String> = t
                    .fields
                    .iter()
                    .map(|f| type_to_string_inner(f.id, types, depth + 1))
                    .collect();
                format!("({})", fields.join(", "))
            }
        }

        TypeDef::Compact(c) => {
            format!(
                "Compact<{}>",
                type_to_string_inner(c.type_param.id, types, depth + 1)
            )
        }

        TypeDef::BitSequence(_) => "BitVec".to_string(),
    }
}

fn format_primitive(p: &scale_info::TypeDefPrimitive) -> String {
    match p {
        scale_info::TypeDefPrimitive::Bool => "bool",
        scale_info::TypeDefPrimitive::Char => "char",
        scale_info::TypeDefPrimitive::Str => "str",
        scale_info::TypeDefPrimitive::U8 => "u8",
        scale_info::TypeDefPrimitive::U16 => "u16",
        scale_info::TypeDefPrimitive::U32 => "u32",
        scale_info::TypeDefPrimitive::U64 => "u64",
        scale_info::TypeDefPrimitive::U128 => "u128",
        scale_info::TypeDefPrimitive::U256 => "u256",
        scale_info::TypeDefPrimitive::I8 => "i8",
        scale_info::TypeDefPrimitive::I16 => "i16",
        scale_info::TypeDefPrimitive::I32 => "i32",
        scale_info::TypeDefPrimitive::I64 => "i64",
        scale_info::TypeDefPrimitive::I128 => "i128",
        scale_info::TypeDefPrimitive::I256 => "i256",
    }
    .to_string()
}

fn format_composite_fields(
    c: &scale_info::TypeDefComposite<PortableForm>,
    types: &scale_info::PortableRegistry,
    depth: usize,
) -> String {
    let fields: Vec<String> = c
        .fields
        .iter()
        .map(|f| {
            let ty = type_to_string_inner(f.ty.id, types, depth + 1);
            if let Some(name) = &f.name {
                format!("{}: {}", name, ty)
            } else {
                ty
            }
        })
        .collect();

    if c.fields.iter().any(|f| f.name.is_some()) {
        format!("{{ {} }}", fields.join(", "))
    } else {
        format!("({})", fields.join(", "))
    }
}

fn find_variant_field(
    v: &scale_info::TypeDefVariant<PortableForm>,
    variant_name: &str,
    types: &scale_info::PortableRegistry,
    depth: usize,
) -> Option<String> {
    v.variants
        .iter()
        .find(|var| var.name == variant_name)
        .and_then(|var| var.fields.first())
        .map(|f| type_to_string_inner(f.ty.id, types, depth + 1))
}

// ---------------------------------------------------------------------------
// Lock ID decoding
// ---------------------------------------------------------------------------

/// Extract lock ID from a dynamic value that represents [u8; 8].
/// Lock IDs are 8-byte arrays that often contain ASCII text like "staking ".
pub fn decode_lock_id(id_value: &DecodedValue) -> String {
    // Try to extract bytes from an unnamed composite (array of u8 values)
    if let ValueDef::Composite(Composite::Unnamed(fields)) = &id_value.value {
        let bytes: Vec<u8> = fields
            .iter()
            .filter_map(|v| v.as_u128().map(|n| n as u8))
            .collect();
        if !bytes.is_empty() {
            let s = String::from_utf8_lossy(&bytes);
            return lock_id_to_name(s.trim_end_matches('\0')).to_string();
        }
    }
    format_value(id_value)
}

/// Map common lock ID strings to human-readable names.
pub fn lock_id_to_name(raw: &str) -> &str {
    match raw.trim() {
        "staking" | "staking " => "Staking",
        "pyconvot" => "Governance (conviction voting)",
        "vesting " | "vesting" => "Vesting",
        "democrac" => "Democracy (legacy)",
        "phrelect" => "Phragmen election",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to convert Value<()> to Value<u32> (DecodedValue).
    fn to_decoded(val: subxt::ext::scale_value::Value) -> DecodedValue {
        val.map_context(|_| 0u32)
    }

    #[test]
    fn test_format_value_primitive() {
        use subxt::ext::scale_value::Value;
        assert_eq!(format_value(&to_decoded(Value::u128(42))), "42");
        assert_eq!(format_value(&to_decoded(Value::bool(true))), "true");
        assert_eq!(format_value(&to_decoded(Value::string("hello"))), "hello");
    }

    #[test]
    fn test_format_value_named_composite() {
        use subxt::ext::scale_value::Value;
        let val = to_decoded(Value::named_composite(vec![
            ("free", Value::u128(1000)),
            ("reserved", Value::u128(0)),
        ]));
        assert_eq!(format_value(&val), "{ free: 1000, reserved: 0 }");
    }

    #[test]
    fn test_format_value_variant() {
        use subxt::ext::scale_value::Value;
        let val = to_decoded(Value::unnamed_variant("Some", vec![Value::u128(42)]));
        assert_eq!(format_value(&val), "Some(42)");

        let val = to_decoded(Value::unnamed_variant("None", vec![]));
        assert_eq!(format_value(&val), "None");
    }
}
