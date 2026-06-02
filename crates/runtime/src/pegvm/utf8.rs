//! Pinpoint UTF-8 decode helper used by the [`CharClass`](
//! crate::pegvm::Instruction::CharSet) opcode.
//!
//! Returns either the decoded scalar and its byte width, or the byte
//! width of the *maximal invalid prefix* per the WHATWG UTF-8 decode
//! algorithm. The latter is what the VM uses to advance past a bad
//! sequence and emit a recovery span.
//!
//! Single-byte ASCII (`< 0x80`) decodes in one step without table lookups.
//! Non-ASCII leads validate (a) lead-byte class, (b) continuation-byte
//! count, (c) overlong-encoding rejection, (d) surrogate / out-of-range
//! rejection — matching `std::str::from_utf8`'s acceptance.

/// Outcome of [`decode_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8DecodeResult {
    /// A valid Unicode scalar value at `pos`, of `bytes` UTF-8 byte width
    /// (1..=4).
    Valid { scalar: char, bytes: usize },
    /// The bytes starting at `pos` form an invalid UTF-8 sequence whose
    /// maximal invalid prefix is `bytes` long (1..=3). The caller should
    /// emit a recovery span over `pos..pos+bytes` and advance `pos` by
    /// `bytes`.
    Invalid { bytes: usize },
    /// End of input — `pos >= input.len()`. The caller treats this as a
    /// match failure (the opcode can't make progress).
    Eof,
}

/// Decode one UTF-8 scalar (or maximal invalid prefix) starting at
/// `input[pos]`. The acceptance rule matches `std::str::from_utf8`: only
/// well-formed UTF-8 returns `Valid`. Surrogates, overlongs, out-of-range
/// scalars, and stray continuation bytes return `Invalid` with the
/// maximal-invalid-prefix byte count per WHATWG.
pub fn decode_at(input: &[u8], pos: usize) -> Utf8DecodeResult {
    let Some(&lead) = input.get(pos) else {
        return Utf8DecodeResult::Eof;
    };

    // ASCII fast path.
    if lead < 0x80 {
        return Utf8DecodeResult::Valid {
            scalar: lead as char,
            bytes: 1,
        };
    }

    // Lead-byte classification (RFC 3629 / WHATWG).
    let (expected_len, lo_bound, hi_bound, lead_value): (usize, u8, u8, u32) = match lead {
        // Stray continuation byte or invalid lead.
        0x80..=0xBF | 0xC0..=0xC1 | 0xF5..=0xFF => return Utf8DecodeResult::Invalid { bytes: 1 },
        0xC2..=0xDF => (2, 0x80, 0xBF, (lead & 0x1F) as u32),
        0xE0 => (3, 0xA0, 0xBF, (lead & 0x0F) as u32),
        0xE1..=0xEC => (3, 0x80, 0xBF, (lead & 0x0F) as u32),
        0xED => (3, 0x80, 0x9F, (lead & 0x0F) as u32), // exclude surrogates
        0xEE..=0xEF => (3, 0x80, 0xBF, (lead & 0x0F) as u32),
        0xF0 => (4, 0x90, 0xBF, (lead & 0x07) as u32),
        0xF1..=0xF3 => (4, 0x80, 0xBF, (lead & 0x07) as u32),
        0xF4 => (4, 0x80, 0x8F, (lead & 0x07) as u32), // exclude > U+10FFFF
        0x00..=0x7F => unreachable!("ASCII handled above"),
    };

    let mut value = lead_value;
    let mut consumed = 1usize;
    for i in 0..(expected_len - 1) {
        let Some(&b) = input.get(pos + 1 + i) else {
            // Truncated at end of input. Invalid prefix is what we read.
            return Utf8DecodeResult::Invalid { bytes: consumed };
        };
        let (cont_lo, cont_hi) = if i == 0 {
            (lo_bound, hi_bound)
        } else {
            (0x80, 0xBF)
        };
        if b < cont_lo || b > cont_hi {
            return Utf8DecodeResult::Invalid { bytes: consumed };
        }
        value = (value << 6) | ((b & 0x3F) as u32);
        consumed += 1;
    }

    debug_assert!(consumed == expected_len);
    debug_assert!(value <= 0x10FFFF);
    // value is a valid scalar by construction (bounds checked above).
    match char::from_u32(value) {
        Some(c) => Utf8DecodeResult::Valid {
            scalar: c,
            bytes: consumed,
        },
        None => Utf8DecodeResult::Invalid { bytes: consumed },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(scalar: char, bytes: usize) -> Utf8DecodeResult {
        Utf8DecodeResult::Valid { scalar, bytes }
    }

    fn invalid(bytes: usize) -> Utf8DecodeResult {
        Utf8DecodeResult::Invalid { bytes }
    }

    #[test]
    fn ascii() {
        assert_eq!(decode_at(b"A", 0), valid('A', 1));
        assert_eq!(decode_at(b"abc", 1), valid('b', 1));
        assert_eq!(decode_at(b"\0", 0), valid('\0', 1));
        assert_eq!(decode_at(b"\x7F", 0), valid('\x7F', 1));
    }

    #[test]
    fn two_byte() {
        // U+00E9 é = C3 A9
        assert_eq!(decode_at(b"\xC3\xA9", 0), valid('\u{00E9}', 2));
        // U+0080 = C2 80 (smallest two-byte)
        assert_eq!(decode_at(b"\xC2\x80", 0), valid('\u{0080}', 2));
        // U+07FF = DF BF (largest two-byte)
        assert_eq!(decode_at(b"\xDF\xBF", 0), valid('\u{07FF}', 2));
    }

    #[test]
    fn three_byte() {
        // U+4E16 世 = E4 B8 96
        assert_eq!(decode_at(b"\xE4\xB8\x96", 0), valid('\u{4E16}', 3));
        // U+0800 = E0 A0 80 (smallest three-byte)
        assert_eq!(decode_at(b"\xE0\xA0\x80", 0), valid('\u{0800}', 3));
        // U+FFFF = EF BF BF
        assert_eq!(decode_at(b"\xEF\xBF\xBF", 0), valid('\u{FFFF}', 3));
    }

    #[test]
    fn four_byte() {
        // U+1F600 😀 = F0 9F 98 80
        assert_eq!(decode_at(b"\xF0\x9F\x98\x80", 0), valid('\u{1F600}', 4));
        // U+10000 = F0 90 80 80 (smallest four-byte)
        assert_eq!(decode_at(b"\xF0\x90\x80\x80", 0), valid('\u{10000}', 4));
        // U+10FFFF = F4 8F BF BF (largest scalar)
        assert_eq!(decode_at(b"\xF4\x8F\xBF\xBF", 0), valid('\u{10FFFF}', 4));
    }

    #[test]
    fn eof() {
        assert_eq!(decode_at(b"", 0), Utf8DecodeResult::Eof);
        assert_eq!(decode_at(b"A", 1), Utf8DecodeResult::Eof);
    }

    #[test]
    fn stray_continuation() {
        // Bare continuation bytes are invalid (1-byte prefix).
        for b in 0x80u8..=0xBFu8 {
            assert_eq!(decode_at(&[b], 0), invalid(1), "byte 0x{:02X}", b);
        }
    }

    #[test]
    fn invalid_lead() {
        // 0xC0, 0xC1, 0xF5..=0xFF are never valid lead bytes.
        for b in [0xC0u8, 0xC1u8, 0xF5u8, 0xFFu8] {
            assert_eq!(decode_at(&[b, 0x80], 0), invalid(1), "byte 0x{:02X}", b);
        }
    }

    #[test]
    fn truncated_two_byte() {
        // 0xC3 alone — no continuation.
        assert_eq!(decode_at(b"\xC3", 0), invalid(1));
        // 0xC3 followed by EOF (truncation).
        assert_eq!(decode_at(b"A\xC3", 1), invalid(1));
    }

    #[test]
    fn truncated_three_byte() {
        // 0xE4 alone.
        assert_eq!(decode_at(b"\xE4", 0), invalid(1));
        // 0xE4 B8 (one continuation, missing second).
        assert_eq!(decode_at(b"\xE4\xB8", 0), invalid(2));
        // 0xE4 B8 followed by a non-continuation byte.
        assert_eq!(decode_at(b"\xE4\xB8A", 0), invalid(2));
    }

    #[test]
    fn truncated_four_byte() {
        assert_eq!(decode_at(b"\xF0", 0), invalid(1));
        assert_eq!(decode_at(b"\xF0\x9F", 0), invalid(2));
        assert_eq!(decode_at(b"\xF0\x9F\x98", 0), invalid(3));
    }

    #[test]
    fn overlong_two_byte() {
        // C0 80 is the overlong encoding of U+0000 — rejected by C0 lead.
        assert_eq!(decode_at(b"\xC0\x80", 0), invalid(1));
        // C1 BF — overlong U+007F — rejected by C1 lead.
        assert_eq!(decode_at(b"\xC1\xBF", 0), invalid(1));
    }

    #[test]
    fn overlong_three_byte() {
        // E0 80 80 — overlong U+0000 — E0 requires lo bound 0xA0.
        assert_eq!(decode_at(b"\xE0\x80\x80", 0), invalid(1));
        // E0 9F BF — overlong of valid U+07FF region — rejected.
        assert_eq!(decode_at(b"\xE0\x9F\xBF", 0), invalid(1));
    }

    #[test]
    fn overlong_four_byte() {
        // F0 80 80 80 — overlong — F0 requires lo bound 0x90.
        assert_eq!(decode_at(b"\xF0\x80\x80\x80", 0), invalid(1));
        // F0 8F BF BF — overlong of U+FFFF — rejected.
        assert_eq!(decode_at(b"\xF0\x8F\xBF\xBF", 0), invalid(1));
    }

    #[test]
    fn surrogate_in_utf8() {
        // ED A0 80 = U+D800 (high surrogate) — rejected by ED lead's
        // restricted hi-bound (0x9F).
        assert_eq!(decode_at(b"\xED\xA0\x80", 0), invalid(1));
        // ED BF BF = U+DFFF — same.
        assert_eq!(decode_at(b"\xED\xBF\xBF", 0), invalid(1));
        // ED 9F BF = U+D7FF — legal, just below the surrogate range.
        assert_eq!(decode_at(b"\xED\x9F\xBF", 0), valid('\u{D7FF}', 3));
    }

    #[test]
    fn out_of_range_four_byte() {
        // F4 90 80 80 = U+110000 — rejected by F4 lead's restricted hi-bound (0x8F).
        assert_eq!(decode_at(b"\xF4\x90\x80\x80", 0), invalid(1));
    }

    #[test]
    fn latin1_as_utf8() {
        // The "cliché_value" scenario: Latin-1 0xE9 (é) followed by
        // ASCII bytes. 0xE9 is a 3-byte lead; next byte 0x5F is not a
        // valid continuation (must be 0x80..=0xBF). Maximal invalid
        // prefix is 1 byte.
        let input = b"clich\xE9_value";
        assert_eq!(decode_at(input, 5), invalid(1));
    }

    #[test]
    fn consecutive_bad_bytes() {
        // Multiple bad bytes in a row: each call returns one
        // maximal-invalid-prefix at a time.
        let input = b"\xC0\xC1\xF5";
        assert_eq!(decode_at(input, 0), invalid(1));
        assert_eq!(decode_at(input, 1), invalid(1));
        assert_eq!(decode_at(input, 2), invalid(1));
    }
}
