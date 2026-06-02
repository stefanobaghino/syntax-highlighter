//! Unicode property registry backing `\p{...}` / `\P{...}` syntax.
//!
//! Each property name resolves to a [`CharSet`] that the compiler
//! consumes when emitting `CharSet` bytecode. Sets are built
//! from build-script-generated interval tables (no runtime UCD scan).
//! The build script lives at `build.rs`; its output is included via
//! `include!(concat!(env!("OUT_DIR"), "/unicode_properties.rs"))`.
//!
//! Supported property names (case-sensitive) and aliases:
//!
//! - `L`, `Letter`, `gc=L`, `gc=Letter`, `General_Category=Letter` —
//!   General_Category=Letter (built from [`char::is_alphabetic`], a
//!   superset of strict GC=L that also includes Other_Alphabetic code
//!   points — acceptable imprecision for v1 highlighter use).
//! - `Nd`, `Decimal_Number`, `Number_Decimal_Digit`, `gc=Nd` —
//!   General_Category=Nd via [`char::is_numeric`] (similarly a
//!   superset).
//! - `XID_Start`, `XID_Continue`, `ID_Start`, `ID_Continue` — UAX #31
//!   identifier properties from `unicode-ident` (build-time dep).
//! - `Any` — every Unicode scalar value.
//!
//! Long form `\p{Property=Value}` is parsed identically to the
//! shorthand: this registry handles aliases. New aliases can be added
//! without touching parser code.
//!
//! Caching: each property's [`CharSet`] is materialised on first
//! lookup and stored in a [`OnceLock`]. The materialisation cost is
//! linear in the property's interval count (a few hundred ranges),
//! not the full code-point space.

use std::sync::OnceLock;

use syntax_highlighter::pegvm::CharSet;

include!(concat!(env!("OUT_DIR"), "/unicode_properties.rs"));

/// Look up a property name (canonical or alias) and return the matching
/// [`CharSet`]. Returns `None` if the name is unknown.
pub fn lookup(name: &str) -> Option<&'static CharSet> {
    let normalized = canonicalize(name)?;
    Some(match normalized {
        Property::Letter => letter_set(),
        Property::Nd => nd_set(),
        Property::XidStart => xid_start_set(),
        Property::XidContinue => xid_continue_set(),
        Property::IdStart => id_start_set(),
        Property::IdContinue => id_continue_set(),
        Property::Any => any_set(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Property {
    Letter,
    Nd,
    XidStart,
    XidContinue,
    IdStart,
    IdContinue,
    Any,
}

fn canonicalize(name: &str) -> Option<Property> {
    // Strip optional `gc=` / `General_Category=` prefix for the General
    // Category properties; long form is otherwise spelled the same way
    // as shorthand.
    let stripped = name
        .strip_prefix("gc=")
        .or_else(|| name.strip_prefix("General_Category="))
        .unwrap_or(name);
    match stripped {
        "L" | "Letter" => Some(Property::Letter),
        "Nd" | "Decimal_Number" | "Number_Decimal_Digit" => Some(Property::Nd),
        "XID_Start" => Some(Property::XidStart),
        "XID_Continue" => Some(Property::XidContinue),
        "ID_Start" => Some(Property::IdStart),
        "ID_Continue" => Some(Property::IdContinue),
        "Any" => Some(Property::Any),
        _ => None,
    }
}

fn set_from_table(ranges: &[(u32, u32)]) -> CharSet {
    let char_ranges: Vec<(char, char)> = ranges
        .iter()
        .map(|&(lo, hi)| {
            (
                char::from_u32(lo).expect("build-script property bound is a valid scalar"),
                char::from_u32(hi).expect("build-script property bound is a valid scalar"),
            )
        })
        .collect();
    CharSet::from_ranges(&char_ranges).expect("build-script property table produced invalid ranges")
}

fn letter_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(L_RANGES))
}

fn nd_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(ND_RANGES))
}

fn xid_start_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(XID_START_RANGES))
}

fn xid_continue_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(XID_CONTINUE_RANGES))
}

fn id_start_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(ID_START_RANGES))
}

fn id_continue_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(|| set_from_table(ID_CONTINUE_RANGES))
}

fn any_set() -> &'static CharSet {
    static SET: OnceLock<CharSet> = OnceLock::new();
    SET.get_or_init(CharSet::any)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_letter_in_letter() {
        let set = lookup("L").unwrap();
        assert!(set.contains_char('A'));
        assert!(set.contains_char('z'));
        assert!(!set.contains_char('0'));
        assert!(!set.contains_char(' '));
    }

    #[test]
    fn long_form_letter() {
        let set = lookup("General_Category=Letter").unwrap();
        assert!(set.contains_char('A'));
    }

    #[test]
    fn non_ascii_letter_in_letter() {
        let set = lookup("L").unwrap();
        assert!(set.contains_char('世'), "U+4E16 should be a letter");
        assert!(set.contains_char('é'), "U+00E9 should be a letter");
    }

    #[test]
    fn ascii_digit_in_nd() {
        let set = lookup("Nd").unwrap();
        assert!(set.contains_char('0'));
        assert!(set.contains_char('9'));
        assert!(!set.contains_char('A'));
    }

    #[test]
    fn xid_start_excludes_digit() {
        let set = lookup("XID_Start").unwrap();
        assert!(set.contains_char('a'));
        assert!(!set.contains_char('0'));
        assert!(!set.contains_char('_'));
    }

    #[test]
    fn xid_continue_includes_digit() {
        let set = lookup("XID_Continue").unwrap();
        assert!(set.contains_char('a'));
        assert!(set.contains_char('0'));
    }

    #[test]
    fn unknown_property_returns_none() {
        assert!(lookup("NotARealProperty").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn any_property_matches_basic_codepoints() {
        let set = lookup("Any").unwrap();
        assert!(set.contains_char('A'));
        assert!(set.contains_char('世'));
        // Surrogates are not Unicode scalar values; the set excludes them
        // by construction.
        assert!(!set.contains(0xD800));
    }
}
