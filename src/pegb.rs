//! Binary serialization for [`crate::pegvm::Program`].
//!
//! Sibling of [`crate::pegc`]: `pegc` produces a `Program` from grammar
//! source, `pegb` round-trips a `Program` through bytes. Depends on
//! `pegvm`; `pegvm` has no reverse dependency. Useful for shipping
//! pre-compiled grammars and for any consumer that wants to skip
//! parse/compile at startup.
//!
//! # Stability scope
//!
//! The format is **not** stable. There's no header, no magic, no version
//! field. Any change to the encoder is silently incompatible with
//! previously-saved bytes; the decoder will fail with [`Error`] variants
//! like [`Error::InvalidOpcode`] or [`Error::TruncatedInput`] on a
//! mismatched buffer. Bytecode artifacts are tied to the exact crate
//! build that produced them. When format stability becomes a deliverable
//! we'll introduce a header (magic + version) at the same time.
//!
//! # Wire format v0
//!
//! Little-endian, byte-packed, no alignment requirements. Layout:
//!
//! ```text
//! capture-name table:
//!   u16 LE      n = capture_kinds.len()
//!   for each name: NUL-terminated UTF-8 bytes
//!
//! rule-name table:
//!   u16 LE      n = rule_names.len()
//!   for each name: NUL-terminated UTF-8 bytes
//!
//! instruction stream:
//!   u32 LE      k = code.len()
//!   for each instruction: u8 tag, then variant-specific payload
//! ```
//!
//! `Label` and `MemoId` go on the wire as LEB128 unsigned varints (1–5
//! bytes for a `u32` value); every program `pegc` produces today fits in
//! 2 varint bytes per Label/MemoId. `CaptureKind` stays `u16`.
//! `CharSet` payloads are 32 raw bytes (bitmaps don't compress under
//! varint).
//!
//! # No zero-copy
//!
//! `decode` allocates a `Vec<Instruction>` and walks the input. We don't
//! attempt `&[u8] → &[Instruction]` zero-copy: `Instruction` is a Rust
//! enum with a `CharSet([u8;32])` payload, so the in-memory layout
//! requires a parallel `repr(C)` opcode struct, padded strings, and an
//! aligned input buffer to support that. None of the bundled grammars
//! is large enough for the load-time allocation to matter — sqlite at
//! 5,622 instructions decodes in sub-millisecond time.

use crate::pegvm::{CaptureKind, CharSet, Instruction, Label, MemoId, Program, RuleKind};

/// Failure modes for [`decode`]. [`encode`] is infallible for any
/// well-formed `Program` (`pegc::compile` always produces one).
#[derive(Debug)]
pub enum Error {
    /// Decoder ran out of bytes mid-read. `needed` is the byte count the
    /// in-progress read wanted; `position` is where it started.
    TruncatedInput { needed: usize, position: usize },
    /// Decoder encountered an opcode tag it doesn't know.
    InvalidOpcode { tag: u8, position: usize },
    /// `RuleEnter`'s `RuleKind` discriminator was neither `0` (`Memo`)
    /// nor `1` (`Lr`).
    InvalidRuleKind { tag: u8, position: usize },
    /// Varint read either ran past the 5-byte upper bound for a `u32`
    /// value, or the 5th byte had bits set that overflow `u32`.
    MalformedVarint { position: usize },
    /// Capture name bytes were not valid UTF-8.
    InvalidCaptureName(std::str::Utf8Error),
    /// Rule name bytes were not valid UTF-8.
    InvalidRuleName(std::str::Utf8Error),
    /// Decoder reached a sensible stopping point but bytes remain in the
    /// buffer — usually means the producer wrote a slightly different
    /// format than the one this decoder reads.
    TrailingBytes { remaining: usize },
    /// Declared instruction count exceeds [`MAX_INSTRUCTION_COUNT`]. The
    /// decoder reads `code.len()` as a `u32` from the wire and sizes a
    /// `Vec` for it; without a sanity cap a 7-byte input can request a
    /// multi-GB allocation. The cap is generous (~3 orders of magnitude
    /// above the largest bundled grammar's instruction count); legitimate
    /// programs never hit it.
    ProgramTooLarge { count: u32 },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TruncatedInput { needed, position } => write!(
                f,
                "truncated input: needed {} bytes at position {}",
                needed, position
            ),
            Error::InvalidOpcode { tag, position } => {
                write!(
                    f,
                    "invalid opcode tag 0x{:02x} at position {}",
                    tag, position
                )
            }
            Error::InvalidRuleKind { tag, position } => write!(
                f,
                "invalid RuleKind discriminant {} at position {}",
                tag, position
            ),
            Error::MalformedVarint { position } => {
                write!(f, "malformed varint at position {}", position)
            }
            Error::InvalidCaptureName(e) => write!(f, "invalid capture name UTF-8: {}", e),
            Error::InvalidRuleName(e) => write!(f, "invalid rule name UTF-8: {}", e),
            Error::TrailingBytes { remaining } => {
                write!(f, "{} trailing byte(s) after end of program", remaining)
            }
            Error::ProgramTooLarge { count } => write!(
                f,
                "declared instruction count {} exceeds the {} sanity cap",
                count, MAX_INSTRUCTION_COUNT
            ),
        }
    }
}

impl std::error::Error for Error {}

// ─── Opcode tags ──────────────────────────────────────────────────────
//
// Stable u8 values keyed off the variant. Numbering leaves gaps between
// groups so a future variant can slot in alphabetically without
// renumbering everything.

const TAG_CHAR: u8 = 0x01;
const TAG_SET: u8 = 0x02;
const TAG_ANY: u8 = 0x03;
const TAG_TEST_CHAR: u8 = 0x04;
const TAG_TEST_SET: u8 = 0x05;

const TAG_JUMP: u8 = 0x10;
const TAG_CHOICE: u8 = 0x11;
const TAG_COMMIT: u8 = 0x12;
const TAG_PARTIAL_COMMIT: u8 = 0x13;
const TAG_BACK_COMMIT: u8 = 0x14;
const TAG_FAIL_TWICE: u8 = 0x15;
const TAG_FAIL: u8 = 0x16;

const TAG_CALL: u8 = 0x20;
const TAG_RETURN: u8 = 0x21;
const TAG_RULE_ENTER: u8 = 0x22;
const TAG_MEMO_CLOSE: u8 = 0x23;
const TAG_LR_TAIL: u8 = 0x24;

const TAG_CAPTURE_BEGIN: u8 = 0x30;
const TAG_CAPTURE_END: u8 = 0x31;

// Per-iteration recovery-scope opcodes for `p*^`. See `Instruction`
// docs and `src/pegc/compiler.rs` for usage. No payload: all state is
// on the VM stack.
const TAG_RECOVER_SCOPE_BEGIN: u8 = 0x40;
const TAG_RECOVER_TO_SCOPED_MAX: u8 = 0x41;
const TAG_RECOVER_SCOPE_END: u8 = 0x42;

// `End` is the program-termination sentinel; placing it at the top of the
// `u8` range (rather than in a grouped sub-range) keeps the
// "I'm done, no more instructions" tag visually distinct from the
// matching / control-flow / call / capture groups above.
const TAG_END: u8 = 0xFF;

const RULE_KIND_MEMO: u8 = 0;
const RULE_KIND_LR: u8 = 1;

/// Sanity cap on the declared instruction count read from the wire,
/// enforced by [`decode`]. Without it, the decoder would happily
/// `Vec::with_capacity(u32::MAX)` based on 4 attacker-controlled bytes
/// and request a multi-GB allocation on a few-byte input. ~3 orders of
/// magnitude above the largest bundled grammar (sqlite, ~5,600
/// instructions); raise it deliberately if a real grammar ever
/// approaches the limit.
pub const MAX_INSTRUCTION_COUNT: u32 = 1 << 24;

// ─── Encoder ─────────────────────────────────────────────────────────

/// Serialize a `Program` to a freshly-allocated `Vec<u8>` in the v0
/// wire format.
pub fn encode(program: &Program) -> Vec<u8> {
    let mut out = Vec::new();
    write_name_table(&mut out, &program.capture_kinds, "capture");
    write_name_table(&mut out, &program.rule_names, "rule");
    write_instructions(&mut out, &program.code);
    out
}

fn write_name_table(out: &mut Vec<u8>, names: &[String], label: &str) {
    debug_assert!(
        names.len() <= u16::MAX as usize,
        "{}_names.len() ({}) exceeds u16::MAX",
        label,
        names.len()
    );
    out.extend_from_slice(&(names.len() as u16).to_le_bytes());
    for name in names {
        debug_assert!(
            !name.as_bytes().contains(&0),
            "{} name {:?} contains embedded NUL",
            label,
            name
        );
        out.extend_from_slice(name.as_bytes());
        out.push(0);
    }
}

fn write_instructions(out: &mut Vec<u8>, code: &[Instruction]) {
    debug_assert!(
        code.len() <= u32::MAX as usize,
        "code.len() ({}) exceeds u32::MAX",
        code.len()
    );
    out.extend_from_slice(&(code.len() as u32).to_le_bytes());
    for ins in code {
        write_instruction(out, ins);
    }
}

fn write_instruction(out: &mut Vec<u8>, ins: &Instruction) {
    match ins {
        Instruction::Char(b) => {
            out.push(TAG_CHAR);
            out.push(*b);
        }
        Instruction::Set(set) => {
            out.push(TAG_SET);
            out.extend_from_slice(set.bitmap());
        }
        Instruction::Any(n) => {
            out.push(TAG_ANY);
            out.push(*n);
        }
        Instruction::TestChar(b, label) => {
            out.push(TAG_TEST_CHAR);
            out.push(*b);
            write_varint_u32(out, label.0);
        }
        Instruction::TestSet(set, label) => {
            out.push(TAG_TEST_SET);
            out.extend_from_slice(set.bitmap());
            write_varint_u32(out, label.0);
        }
        Instruction::Jump(label) => {
            out.push(TAG_JUMP);
            write_varint_u32(out, label.0);
        }
        Instruction::Choice(label) => {
            out.push(TAG_CHOICE);
            write_varint_u32(out, label.0);
        }
        Instruction::Commit(label) => {
            out.push(TAG_COMMIT);
            write_varint_u32(out, label.0);
        }
        Instruction::PartialCommit(label) => {
            out.push(TAG_PARTIAL_COMMIT);
            write_varint_u32(out, label.0);
        }
        Instruction::BackCommit(label) => {
            out.push(TAG_BACK_COMMIT);
            write_varint_u32(out, label.0);
        }
        Instruction::FailTwice => out.push(TAG_FAIL_TWICE),
        Instruction::Fail => out.push(TAG_FAIL),
        Instruction::Call(label) => {
            out.push(TAG_CALL);
            write_varint_u32(out, label.0);
        }
        Instruction::Return => out.push(TAG_RETURN),
        Instruction::RuleEnter(memo_id, kind, label) => {
            out.push(TAG_RULE_ENTER);
            write_varint_u32(out, memo_id.0);
            out.push(match kind {
                RuleKind::Memo => RULE_KIND_MEMO,
                RuleKind::Lr => RULE_KIND_LR,
            });
            write_varint_u32(out, label.0);
        }
        Instruction::MemoClose(memo_id) => {
            out.push(TAG_MEMO_CLOSE);
            write_varint_u32(out, memo_id.0);
        }
        Instruction::LRTail(memo_id, label) => {
            out.push(TAG_LR_TAIL);
            write_varint_u32(out, memo_id.0);
            write_varint_u32(out, label.0);
        }
        Instruction::CaptureBegin(kind) => {
            out.push(TAG_CAPTURE_BEGIN);
            out.extend_from_slice(&kind.0.to_le_bytes());
        }
        Instruction::CaptureEnd => out.push(TAG_CAPTURE_END),
        Instruction::RecoverScopeBegin => out.push(TAG_RECOVER_SCOPE_BEGIN),
        Instruction::RecoverToScopedMax => out.push(TAG_RECOVER_TO_SCOPED_MAX),
        Instruction::RecoverScopeEnd => out.push(TAG_RECOVER_SCOPE_END),
        Instruction::End => out.push(TAG_END),
    }
}

fn write_varint_u32(out: &mut Vec<u8>, mut v: u32) {
    while v >= 0x80 {
        out.push(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

// ─── Decoder ─────────────────────────────────────────────────────────

/// Deserialize a `Program` from the v0 wire format. Validates magic-free
/// structure: opcode tags, varint widths, UTF-8 in capture names, and
/// requires the buffer to be exactly consumed.
pub fn decode(bytes: &[u8]) -> Result<Program, Error> {
    let mut cur = Cursor::new(bytes);
    let capture_kinds = read_name_table(&mut cur, Error::InvalidCaptureName)?;
    let rule_names = read_name_table(&mut cur, Error::InvalidRuleName)?;
    let code = read_instructions(&mut cur)?;
    if cur.pos != bytes.len() {
        return Err(Error::TrailingBytes {
            remaining: bytes.len() - cur.pos,
        });
    }
    let rule_count = derive_rule_count(&code);
    Ok(Program {
        code,
        capture_kinds,
        rule_count,
        rule_names,
    })
}

fn read_name_table(
    cur: &mut Cursor<'_>,
    err_ctor: fn(std::str::Utf8Error) -> Error,
) -> Result<Vec<String>, Error> {
    let n = cur.read_u16_le()? as usize;
    let mut names = Vec::with_capacity(n);
    for _ in 0..n {
        let bytes = cur.read_until_nul()?;
        let s = std::str::from_utf8(bytes).map_err(err_ctor)?;
        names.push(s.to_string());
    }
    Ok(names)
}

fn read_instructions(cur: &mut Cursor<'_>) -> Result<Vec<Instruction>, Error> {
    let k = cur.read_u32_le()?;
    if k > MAX_INSTRUCTION_COUNT {
        return Err(Error::ProgramTooLarge { count: k });
    }
    let k = k as usize;
    let mut code = Vec::with_capacity(k);
    for _ in 0..k {
        code.push(read_instruction(cur)?);
    }
    Ok(code)
}

fn read_instruction(cur: &mut Cursor<'_>) -> Result<Instruction, Error> {
    let tag_pos = cur.pos;
    let tag = cur.read_u8()?;
    Ok(match tag {
        TAG_CHAR => Instruction::Char(cur.read_u8()?),
        TAG_SET => Instruction::Set(CharSet::from_bitmap(cur.read_array::<32>()?)),
        TAG_ANY => Instruction::Any(cur.read_u8()?),
        TAG_TEST_CHAR => {
            let b = cur.read_u8()?;
            let label = Label(cur.read_varint_u32()?);
            Instruction::TestChar(b, label)
        }
        TAG_TEST_SET => {
            let set = CharSet::from_bitmap(cur.read_array::<32>()?);
            let label = Label(cur.read_varint_u32()?);
            Instruction::TestSet(set, label)
        }
        TAG_JUMP => Instruction::Jump(Label(cur.read_varint_u32()?)),
        TAG_CHOICE => Instruction::Choice(Label(cur.read_varint_u32()?)),
        TAG_COMMIT => Instruction::Commit(Label(cur.read_varint_u32()?)),
        TAG_PARTIAL_COMMIT => Instruction::PartialCommit(Label(cur.read_varint_u32()?)),
        TAG_BACK_COMMIT => Instruction::BackCommit(Label(cur.read_varint_u32()?)),
        TAG_FAIL_TWICE => Instruction::FailTwice,
        TAG_FAIL => Instruction::Fail,
        TAG_CALL => Instruction::Call(Label(cur.read_varint_u32()?)),
        TAG_RETURN => Instruction::Return,
        TAG_RULE_ENTER => {
            let memo_id = MemoId(cur.read_varint_u32()?);
            let kind_pos = cur.pos;
            let kind_tag = cur.read_u8()?;
            let kind = match kind_tag {
                RULE_KIND_MEMO => RuleKind::Memo,
                RULE_KIND_LR => RuleKind::Lr,
                _ => {
                    return Err(Error::InvalidRuleKind {
                        tag: kind_tag,
                        position: kind_pos,
                    })
                }
            };
            let label = Label(cur.read_varint_u32()?);
            Instruction::RuleEnter(memo_id, kind, label)
        }
        TAG_MEMO_CLOSE => Instruction::MemoClose(MemoId(cur.read_varint_u32()?)),
        TAG_LR_TAIL => {
            let memo_id = MemoId(cur.read_varint_u32()?);
            let label = Label(cur.read_varint_u32()?);
            Instruction::LRTail(memo_id, label)
        }
        TAG_CAPTURE_BEGIN => {
            let kind = CaptureKind(u16::from_le_bytes(cur.read_array::<2>()?));
            Instruction::CaptureBegin(kind)
        }
        TAG_CAPTURE_END => Instruction::CaptureEnd,
        TAG_RECOVER_SCOPE_BEGIN => Instruction::RecoverScopeBegin,
        TAG_RECOVER_TO_SCOPED_MAX => Instruction::RecoverToScopedMax,
        TAG_RECOVER_SCOPE_END => Instruction::RecoverScopeEnd,
        TAG_END => Instruction::End,
        _ => {
            return Err(Error::InvalidOpcode {
                tag,
                position: tag_pos,
            })
        }
    })
}

fn derive_rule_count(code: &[Instruction]) -> usize {
    code.iter()
        .filter_map(|ins| match ins {
            Instruction::RuleEnter(id, _, _)
            | Instruction::MemoClose(id)
            | Instruction::LRTail(id, _) => Some(id.0),
            _ => None,
        })
        .max()
        .map(|m| m as usize + 1)
        .unwrap_or(0)
}

// ─── Cursor + read helpers ───────────────────────────────────────────

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Cursor { bytes, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        let position = self.pos;
        let b = *self.bytes.get(self.pos).ok_or(Error::TruncatedInput {
            needed: 1,
            position,
        })?;
        self.pos += 1;
        Ok(b)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let position = self.pos;
        let end = self
            .pos
            .checked_add(N)
            .filter(|e| *e <= self.bytes.len())
            .ok_or(Error::TruncatedInput {
                needed: N,
                position,
            })?;
        let arr: [u8; N] = self.bytes[self.pos..end].try_into().unwrap();
        self.pos = end;
        Ok(arr)
    }

    fn read_u16_le(&mut self) -> Result<u16, Error> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }

    fn read_u32_le(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }

    fn read_varint_u32(&mut self) -> Result<u32, Error> {
        let position = self.pos;
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        for byte_index in 0..5 {
            let b = self.read_u8()?;
            let chunk = (b & 0x7F) as u32;
            // For the 5th byte, only the low 4 bits are valid (4 × 7 + 4
            // = 32). Higher bits would overflow u32.
            if byte_index == 4 && (b & 0x7F) > 0x0F {
                return Err(Error::MalformedVarint { position });
            }
            result |= chunk
                .checked_shl(shift)
                .ok_or(Error::MalformedVarint { position })?;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
        }
        // 5 bytes consumed but the 5th still had its continuation bit
        // set — varint extends past the u32 limit.
        Err(Error::MalformedVarint { position })
    }

    fn read_until_nul(&mut self) -> Result<&'a [u8], Error> {
        let start = self.pos;
        let nul =
            self.bytes[start..]
                .iter()
                .position(|&b| b == 0)
                .ok_or(Error::TruncatedInput {
                    needed: 1,
                    position: start,
                })?;
        let bytes = &self.bytes[start..start + nul];
        self.pos = start + nul + 1;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pegc;

    // ─── varint helpers ────────────────────────────────────────────

    #[test]
    fn varint_roundtrip_at_boundary_values() {
        for v in [
            0u32,
            1,
            127,
            128,
            16383,
            16384,
            2_097_151,
            2_097_152,
            268_435_455,
            268_435_456,
            u32::MAX,
        ] {
            let mut buf = Vec::new();
            write_varint_u32(&mut buf, v);
            let mut cur = Cursor::new(&buf);
            let read = cur.read_varint_u32().expect("varint round-trips");
            assert_eq!(read, v, "varint round-trip mismatch at v={}", v);
            assert_eq!(cur.pos, buf.len(), "varint reader stops at end-of-encoding");
        }
    }

    #[test]
    fn varint_widths_are_minimal() {
        for (v, expected_len) in [
            (0u32, 1),
            (127, 1),
            (128, 2),
            (16383, 2),
            (16384, 3),
            (u32::MAX, 5),
        ] {
            let mut buf = Vec::new();
            write_varint_u32(&mut buf, v);
            assert_eq!(
                buf.len(),
                expected_len,
                "v={} encoded to {} bytes, expected {}",
                v,
                buf.len(),
                expected_len
            );
        }
    }

    #[test]
    fn varint_rejects_six_byte_encoding() {
        // Six bytes all with the continuation bit set — illegal for u32.
        let buf = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x01];
        let err = Cursor::new(&buf).read_varint_u32().unwrap_err();
        assert!(matches!(err, Error::MalformedVarint { position: 0 }));
    }

    #[test]
    fn varint_rejects_fifth_byte_overflow_bits() {
        // 5 bytes where the 5th has bits 4..7 set — the encoded value
        // would exceed u32::MAX.
        let buf = [0x80u8, 0x80, 0x80, 0x80, 0x10];
        let err = Cursor::new(&buf).read_varint_u32().unwrap_err();
        assert!(matches!(err, Error::MalformedVarint { position: 0 }));
    }

    // ─── round-trip ────────────────────────────────────────────────

    fn assert_roundtrip(p: &Program) {
        let bytes = encode(p);
        let p2 = decode(&bytes).expect("decode succeeds on freshly-encoded bytes");
        assert_eq!(p.code, p2.code, "code mismatch after round-trip");
        assert_eq!(
            p.capture_kinds, p2.capture_kinds,
            "capture_kinds mismatch after round-trip"
        );
        assert_eq!(
            p.rule_names, p2.rule_names,
            "rule_names mismatch after round-trip"
        );
        assert_eq!(
            p.rule_count, p2.rule_count,
            "rule_count mismatch after round-trip (encoder vs derived-on-decode)"
        );
        // Determinism: encoding twice yields the same bytes.
        assert_eq!(bytes, encode(p), "encoding is deterministic");
    }

    #[test]
    fn smallest_program_round_trips() {
        // Trivial one-rule grammar: `start <- "x"`. Compiles to a
        // bootstrap `Call` + `End` plus the rule body wrapped in
        // `RuleEnter`/`MemoClose`/`Return`.
        let p = pegc::compile("start <- \"x\"").unwrap();
        assert_roundtrip(&p);
    }

    #[test]
    fn every_opcode_round_trips() {
        // Hand-build a Program containing one of every Instruction
        // variant. Validity at the VM level isn't required — we only
        // care that encode/decode preserves the bytes. capture_kinds
        // covers the CaptureBegin reference.
        let p = Program {
            code: vec![
                Instruction::Char(b'a'),
                Instruction::Set(CharSet::from_bytes(b"abc")),
                Instruction::Any(1),
                Instruction::TestChar(b'b', Label(7)),
                Instruction::TestSet(CharSet::from_bytes(b"xyz"), Label(11)),
                Instruction::Jump(Label(13)),
                Instruction::Choice(Label(17)),
                Instruction::Commit(Label(19)),
                Instruction::PartialCommit(Label(23)),
                Instruction::BackCommit(Label(29)),
                Instruction::FailTwice,
                Instruction::Fail,
                Instruction::Call(Label(31)),
                Instruction::Return,
                Instruction::RuleEnter(MemoId(0), RuleKind::Memo, Label(37)),
                Instruction::RuleEnter(MemoId(1), RuleKind::Lr, Label(41)),
                Instruction::MemoClose(MemoId(0)),
                Instruction::LRTail(MemoId(1), Label(43)),
                Instruction::CaptureBegin(CaptureKind(0)),
                Instruction::CaptureEnd,
                Instruction::RecoverScopeBegin,
                Instruction::RecoverToScopedMax,
                Instruction::RecoverScopeEnd,
                Instruction::End,
            ],
            capture_kinds: vec!["alpha".to_string()],
            rule_count: 2,
            rule_names: vec!["start".to_string(), "other".to_string()],
        };
        assert_roundtrip(&p);
    }

    // ─── error paths ───────────────────────────────────────────────

    fn json_program_bytes() -> Vec<u8> {
        let p = pegc::compile(include_str!("../grammars/json.peg")).unwrap();
        encode(&p)
    }

    #[test]
    fn truncated_at_capture_count_errors() {
        let err = decode(&[]).unwrap_err();
        assert!(matches!(err, Error::TruncatedInput { .. }));
        let err = decode(&[0x01]).unwrap_err(); // half a u16
        assert!(matches!(err, Error::TruncatedInput { .. }));
    }

    #[test]
    fn truncated_in_capture_name_errors() {
        // n=1, then "abc" with no NUL terminator.
        let buf = [0x01u8, 0x00, b'a', b'b', b'c'];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, Error::TruncatedInput { .. }));
    }

    #[test]
    fn truncated_in_instruction_payload_errors() {
        let mut bytes = json_program_bytes();
        bytes.pop(); // drop the last byte of the last instruction
        let err = decode(&bytes).unwrap_err();
        assert!(matches!(
            err,
            Error::TruncatedInput { .. } | Error::MalformedVarint { .. }
        ));
    }

    #[test]
    fn invalid_opcode_errors() {
        // Empty capture table + empty rule table + one instruction with a bogus tag.
        let buf = [
            0x00u8, 0x00, // capture count = 0
            0x00, 0x00, // rule count = 0
            0x01, 0x00, 0x00, 0x00, // instruction count = 1
            0x77, // unknown opcode
        ];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, Error::InvalidOpcode { tag: 0x77, .. }));
    }

    #[test]
    fn invalid_rule_kind_errors() {
        // RuleEnter with kind discriminator = 2.
        let buf = [
            0x00u8,
            0x00, // capture count = 0
            0x00,
            0x00, // rule count = 0
            0x01,
            0x00,
            0x00,
            0x00, // instruction count = 1
            TAG_RULE_ENTER,
            0x00, // varint memo_id = 0
            0x02, // bogus rule kind
            0x00, // varint label = 0
        ];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, Error::InvalidRuleKind { tag: 2, .. }));
    }

    #[test]
    fn invalid_capture_name_utf8_errors() {
        // n=1, then 0xFF (invalid UTF-8 leading byte) + NUL.
        let buf = [0x01u8, 0x00, 0xFF, 0x00];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, Error::InvalidCaptureName(_)));
    }

    #[test]
    fn invalid_rule_name_utf8_errors() {
        // Empty capture table, then rule table with one bad-UTF-8 name.
        let buf = [0x00u8, 0x00, 0x01, 0x00, 0xFF, 0x00];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, Error::InvalidRuleName(_)));
    }

    #[test]
    fn trailing_bytes_errors() {
        let mut bytes = json_program_bytes();
        bytes.push(0x42);
        let err = decode(&bytes).unwrap_err();
        assert!(matches!(err, Error::TrailingBytes { remaining: 1 }));
    }

    /// Closes the DoS-shaped allocation surface: the decoder reads
    /// `code.len()` from the wire as a `u32`, and without a cap a
    /// 7-byte adversarial input would request a multi-GB allocation.
    #[test]
    fn instruction_count_above_cap_errors() {
        let count = MAX_INSTRUCTION_COUNT + 1;
        let mut buf = vec![0x00u8, 0x00, 0x00, 0x00]; // empty capture table + empty rule table
        buf.extend_from_slice(&count.to_le_bytes());
        let err = decode(&buf).unwrap_err();
        assert!(
            matches!(err, Error::ProgramTooLarge { count: c } if c == count),
            "expected ProgramTooLarge, got {err:?}"
        );
    }

    #[test]
    fn empty_program_round_trips() {
        let p = Program {
            code: vec![],
            capture_kinds: vec![],
            rule_count: 0,
            rule_names: vec![],
        };
        assert_roundtrip(&p);
    }

    /// Locks in the contract: `encode` does **not** carry `rule_count`;
    /// `decode` reconstructs it as `max(memo_id) + 1` over `RuleEnter` /
    /// `MemoClose` / `LRTail`. A `Program` whose `rule_count` doesn't
    /// match the code stream round-trips with the *recomputed* value,
    /// not the input one — the wire format is the source of truth.
    #[test]
    fn decode_recomputes_rule_count_from_code_stream() {
        let p = Program {
            code: vec![
                Instruction::RuleEnter(MemoId(0), RuleKind::Memo, Label(2)),
                Instruction::MemoClose(MemoId(0)),
                Instruction::Return,
            ],
            capture_kinds: vec![],
            rule_count: 99, // intentionally inconsistent with the code
            rule_names: vec![],
        };
        let bytes = encode(&p);
        let decoded = decode(&bytes).expect("decode succeeds");
        assert_eq!(decoded.code, p.code, "code preserved");
        assert_eq!(decoded.rule_count, 1, "rule_count recomputed from code");
    }
}
