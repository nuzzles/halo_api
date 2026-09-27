//! Ground-weapon ammo via the native production component traversal.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeGroundAmmoOverride {
    Calibrated(i64),
    Stub(i64),
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativeGroundAmmoComponent {
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    pub end_bit: i64,
    pub ported: bool,
    pub read: Option<DecodedComponent>,
    pub width_override: Option<NativeGroundAmmoOverride>,
    pub corruption_fields: Vec<ComponentField>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativeGroundWeaponAmmoRead {
    pub components: Vec<NativeGroundAmmoComponent>,
    pub end_bit: i64,
    pub desync_at: Option<usize>,
    pub ammo: Option<GroundWeaponAmmo>,
    /// The native secondary cursor rereads only the first 19 bits of i20.
    pub ammo_start_bit: Option<i64>,
}
/// Replays the complete production loop, including components after i20, using
/// its own cursor. The returned ammo is reread at i20's attempted start, even if
/// a subsequent component failed. Registry-name and exact-index gates precede
/// mask-bit aliasing. The caller's observer receives the native loop callbacks.
pub fn read_native_ground_weapon_ammo(
    payload: &[u8],
    start: i64,
    mask: &[usize],
    arch: &FilmArchetype,
    context: &NativeReaderContext,
) -> Result<NativeGroundWeaponAmmoRead, NativeReaderProfileError> {
    let mut out = NativeGroundWeaponAmmoRead {
        components: Vec::new(),
        end_bit: start,
        desync_at: None,
        ammo: None,
        ammo_start_bit: None,
    };
    if arch.components.get(20).map(String::as_str) != Some("weapon-ammo-component")
        || arch.components.get(9).map(String::as_str)
            != Some("object-multiplayer-properties-component")
        || !mask.contains(&20)
    {
        return Ok(out);
    }
    let bits = mask.iter().fold(0u64, |bits, i| bits | (1u64 << (i & 63)));
    let mut reader = NativeFilmReader::with_context(payload, context.clone());
    reader.set_native_bit_position(start);
    for (index, name) in arch.components.iter().enumerate() {
        if bits & (1u64 << (index & 63)) == 0 {
            continue;
        }
        let at = reader.native_bit_position();
        let calibrated = context
            .profile
            .grammar
            .calibrated_widths
            .as_ref()
            .and_then(|w| w.get(name));
        let mut component = None;
        let mut width_override = None;
        let ported = if let Some(width) = calibrated {
            reader.skip_signed(width);
            width_override = Some(NativeGroundAmmoOverride::Calibrated(width));
            true
        } else {
            let (status, read) =
                reader.read_component(name, arch.levels.get(index).copied().unwrap_or(0), 42)?;
            component = Some(read);
            if status == Some(true) {
                true
            } else if let Some(width) = context
                .profile
                .grammar
                .stub_widths
                .as_ref()
                .and_then(|w| w.get(name))
            {
                reader.skip_signed(width);
                width_override = Some(NativeGroundAmmoOverride::Stub(width));
                true
            } else {
                false
            }
        };
        let mut corruption_fields = Vec::new();
        if ported && context.profile.grammar.corruption_check {
            let bit = reader.native_bit_position();
            let present = reader.read_bits_wide(1);
            corruption_fields.push(ComponentField {
                name: "component.corruption_check.present".into(),
                bit,
                width: 1,
                raw: present,
            });
            if present != 0 {
                let bit = reader.native_bit_position();
                let raw = reader.read_bits_wide(32);
                corruption_fields.push(ComponentField {
                    name: "component.corruption_check".into(),
                    bit,
                    width: 32,
                    raw,
                });
            }
        }
        out.components.push(NativeGroundAmmoComponent {
            index,
            name: name.clone(),
            start_bit: at,
            end_bit: reader.native_bit_position(),
            ported,
            read: component,
            width_override,
            corruption_fields,
        });
        if !ported {
            out.desync_at = Some(index);
            break;
        }
    }
    out.end_bit = reader.native_bit_position();
    if let Some(c) = out.components.iter().find(|c| c.index == 20) {
        let mut reread = NativeFilmReader::with_context(payload, context.clone());
        reread.set_native_bit_position(c.start_bit);
        out.ammo = Some(GroundWeaponAmmo {
            mag: reread.read_bits_wide(8) as u32,
            res: reread.read_bits_wide(11) as u32,
        });
        out.ammo_start_bit = Some(c.start_bit);
    }
    Ok(out)
}
