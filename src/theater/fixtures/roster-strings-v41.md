# Strict roster-string adapter oracle

Pinned reference: JGtm/LevelUp feat/v75 at
43a01721e8a02c87c955e175936f0ccf8dd97a81, replay/roster_xuids.go.
Generator: ../reference/halo_rust_roster_strings_test.go.txt invokes the actual
RosterXUIDsOf function on 256 ordered string lists.

Cases include signs, surrounding whitespace, tabs/newlines, hexadecimal prefixes,
underscores, Unicode digits, embedded NUL, empty/zero strings, leading zeroes,
maximum u64 and overflow. Repeated valid values test order and duplicate
retention. Native nil output is normalized to an empty vector; values are not
sorted or deduplicated. Rust's weapon-string resolver is intentionally different
and must not replace this strict adapter.

This adapter accepts externally supplied roster identifiers; parsing an identifier
is not evidence that the recording independently establishes that player.
