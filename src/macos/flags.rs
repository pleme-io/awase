//! macOS CGEventFlags ↔ awase Modifiers conversion.
//!
//! CGEventFlags bit layout (from IOLLEvent.h / CGEventTypes.h):
//!   Cmd:   0x0010_0000 (device flags: left 0x08, right 0x10)
//!   Shift: 0x0002_0000 (device flags: left 0x02, right 0x04)
//!   Alt:   0x0008_0000 (device flags: left 0x20, right 0x40)
//!   Ctrl:  0x0004_0000 (device flags: left 0x01, right 0x2000)
//!   Fn:    0x0080_0000

use crate::Modifiers;

/// Bitmask entries: (awase modifier, [main flag, left device flag, right device flag]).
const MODIFIER_MASKS: [(Modifiers, [u64; 3]); 5] = [
    (Modifiers::CMD, [0x0010_0000, 0x0000_0008, 0x0000_0010]),
    (Modifiers::SHIFT, [0x0002_0000, 0x0000_0002, 0x0000_0004]),
    (Modifiers::ALT, [0x0008_0000, 0x0000_0020, 0x0000_0040]),
    (Modifiers::CTRL, [0x0004_0000, 0x0000_0001, 0x0000_2000]),
    (Modifiers::FN, [0x0080_0000, 0x0080_0000, 0x0080_0000]),
];

/// Convert a raw `CGEventFlags` bitmask to awase `Modifiers`.
///
/// Checks both the main modifier flag and the left/right device-dependent
/// flags for each modifier.
#[must_use]
pub fn cg_flags_to_modifiers(flags: u64) -> Modifiers {
    let mut out = Modifiers::NONE;
    for &(modifier, masks) in &MODIFIER_MASKS {
        if masks.iter().any(|&m| (flags & m) == m && m != 0) {
            out |= modifier;
        }
    }
    out
}

/// Convert awase `Modifiers` to a raw `CGEventFlags` bitmask.
///
/// Uses the main (non-device-specific) flag for each modifier.
#[must_use]
pub fn modifiers_to_cg_flags(modifiers: Modifiers) -> u64 {
    let mut flags: u64 = 0;
    if modifiers.contains(Modifiers::CMD) {
        flags |= 0x0010_0000;
    }
    if modifiers.contains(Modifiers::SHIFT) {
        flags |= 0x0002_0000;
    }
    if modifiers.contains(Modifiers::ALT) {
        flags |= 0x0008_0000;
    }
    if modifiers.contains(Modifiers::CTRL) {
        flags |= 0x0004_0000;
    }
    if modifiers.contains(Modifiers::FN) {
        flags |= 0x0080_0000;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_modifiers() {
        assert_eq!(cg_flags_to_modifiers(0), Modifiers::NONE);
    }

    #[test]
    fn cmd_from_main_flag() {
        let mods = cg_flags_to_modifiers(0x0010_0000);
        assert!(mods.contains(Modifiers::CMD));
        assert!(!mods.contains(Modifiers::SHIFT));
    }

    #[test]
    fn cmd_from_left_device_flag() {
        let mods = cg_flags_to_modifiers(0x0000_0008);
        assert!(mods.contains(Modifiers::CMD));
    }

    #[test]
    fn cmd_from_right_device_flag() {
        let mods = cg_flags_to_modifiers(0x0000_0010);
        assert!(mods.contains(Modifiers::CMD));
    }

    #[test]
    fn shift_from_main_flag() {
        let mods = cg_flags_to_modifiers(0x0002_0000);
        assert!(mods.contains(Modifiers::SHIFT));
    }

    #[test]
    fn alt_from_main_flag() {
        let mods = cg_flags_to_modifiers(0x0008_0000);
        assert!(mods.contains(Modifiers::ALT));
    }

    #[test]
    fn ctrl_from_main_flag() {
        let mods = cg_flags_to_modifiers(0x0004_0000);
        assert!(mods.contains(Modifiers::CTRL));
    }

    #[test]
    fn fn_modifier() {
        let mods = cg_flags_to_modifiers(0x0080_0000);
        assert!(mods.contains(Modifiers::FN));
    }

    #[test]
    fn combined_flags() {
        // cmd + alt + shift
        let mods = cg_flags_to_modifiers(0x0010_0000 | 0x0008_0000 | 0x0002_0000);
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::ALT));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(!mods.contains(Modifiers::CTRL));
        assert!(!mods.contains(Modifiers::FN));
    }

    #[test]
    fn roundtrip_modifiers() {
        let cases = [
            Modifiers::NONE,
            Modifiers::CMD,
            Modifiers::SHIFT,
            Modifiers::ALT,
            Modifiers::CTRL,
            Modifiers::FN,
            Modifiers::CMD | Modifiers::SHIFT,
            Modifiers::CMD | Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT,
            Modifiers::HYPER,
        ];
        for mods in cases {
            let flags = modifiers_to_cg_flags(mods);
            let back = cg_flags_to_modifiers(flags);
            assert_eq!(back, mods, "roundtrip failed for {mods:?}");
        }
    }

    #[test]
    fn modifiers_to_flags_cmd() {
        assert_eq!(modifiers_to_cg_flags(Modifiers::CMD), 0x0010_0000);
    }

    #[test]
    fn modifiers_to_flags_hyper() {
        let flags = modifiers_to_cg_flags(Modifiers::HYPER);
        assert_eq!(
            flags,
            0x0010_0000 | 0x0004_0000 | 0x0008_0000 | 0x0002_0000
        );
    }

    // ── Additional flags tests ──────────────────────────────────────

    #[test]
    fn modifiers_to_flags_none() {
        assert_eq!(modifiers_to_cg_flags(Modifiers::NONE), 0);
    }

    #[test]
    fn modifiers_to_flags_each_modifier() {
        assert_eq!(modifiers_to_cg_flags(Modifiers::CMD), 0x0010_0000);
        assert_eq!(modifiers_to_cg_flags(Modifiers::SHIFT), 0x0002_0000);
        assert_eq!(modifiers_to_cg_flags(Modifiers::ALT), 0x0008_0000);
        assert_eq!(modifiers_to_cg_flags(Modifiers::CTRL), 0x0004_0000);
        assert_eq!(modifiers_to_cg_flags(Modifiers::FN), 0x0080_0000);
    }

    #[test]
    fn modifiers_to_flags_caps_lock_not_included() {
        // CAPS_LOCK is not in MODIFIER_MASKS, so it should produce 0
        assert_eq!(modifiers_to_cg_flags(Modifiers::CAPS_LOCK), 0);
    }

    #[test]
    fn left_device_flags_recognized() {
        // Left shift device flag
        let mods = cg_flags_to_modifiers(0x0000_0002);
        assert!(mods.contains(Modifiers::SHIFT));

        // Left alt device flag
        let mods = cg_flags_to_modifiers(0x0000_0020);
        assert!(mods.contains(Modifiers::ALT));

        // Left ctrl device flag
        let mods = cg_flags_to_modifiers(0x0000_0001);
        assert!(mods.contains(Modifiers::CTRL));
    }

    #[test]
    fn right_device_flags_recognized() {
        // Right shift device flag
        let mods = cg_flags_to_modifiers(0x0000_0004);
        assert!(mods.contains(Modifiers::SHIFT));

        // Right alt device flag
        let mods = cg_flags_to_modifiers(0x0000_0040);
        assert!(mods.contains(Modifiers::ALT));

        // Right ctrl device flag
        let mods = cg_flags_to_modifiers(0x0000_2000);
        assert!(mods.contains(Modifiers::CTRL));
    }

    #[test]
    fn mixed_device_and_main_flags() {
        // Left cmd device flag + main shift flag
        let mods = cg_flags_to_modifiers(0x0000_0008 | 0x0002_0000);
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(!mods.contains(Modifiers::ALT));
    }

    #[test]
    fn irrelevant_bits_ignored() {
        // Set bits that are not modifier flags
        let mods = cg_flags_to_modifiers(0x0000_0100);
        assert_eq!(mods, Modifiers::NONE);
    }

    #[test]
    fn all_device_flags_at_once() {
        // All left device flags combined
        let flags = 0x0000_0008 // left cmd
                  | 0x0000_0002 // left shift
                  | 0x0000_0020 // left alt
                  | 0x0000_0001 // left ctrl
                  | 0x0080_0000; // fn
        let mods = cg_flags_to_modifiers(flags);
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(mods.contains(Modifiers::ALT));
        assert!(mods.contains(Modifiers::CTRL));
        assert!(mods.contains(Modifiers::FN));
    }

    #[test]
    fn roundtrip_fn_modifier() {
        let mods = Modifiers::FN;
        let flags = modifiers_to_cg_flags(mods);
        let back = cg_flags_to_modifiers(flags);
        assert_eq!(back, mods);
    }

    #[test]
    fn roundtrip_single_modifiers() {
        for mods in [
            Modifiers::CMD,
            Modifiers::CTRL,
            Modifiers::ALT,
            Modifiers::SHIFT,
            Modifiers::FN,
        ] {
            let flags = modifiers_to_cg_flags(mods);
            let back = cg_flags_to_modifiers(flags);
            assert_eq!(back, mods, "roundtrip failed for {mods:?}");
        }
    }
}
