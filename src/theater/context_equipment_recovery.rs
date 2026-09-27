//! Native sparse/dense equipment recovery probes with inherited reader profiles.
use super::*;
use std::sync::{Arc, Mutex};

/// A speculative probe is not an accepted equipment emission. Window and counter
/// validation must still admit it before it can affect equipment state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextEquipmentRecoveryProbe {
    pub dense: bool,
    pub reached_target: bool,
    /// Last hook, even when the target walk subsequently fails.
    pub observation: Option<FilmComponentObservation>,
    pub components: Vec<EquipmentRecoveryComponentRead>,
}
impl ContextEquipmentRecoveryProbe {
    pub fn candidate(&self) -> Option<(u32, i32)> {
        if !self.reached_target {
            return None;
        }
        match self.observation.as_ref()? {
            FilmComponentObservation::AbilitySet { counter, rank, .. } => {
                Some((*counter as u32, *rank))
            }
            _ => None,
        }
    }
}

/// Native walkEquipRecoveryAt. The caller admits the production header and
/// active slot/time window first. This checks the recovery mask and dense i0
/// anchor, then requires a successful target visit and an ability-set hook.
/// It does not impose a component name on index 48: an earlier component may
/// publish the hook. The supplied observer is isolated from speculative reads.
pub fn probe_context_equipment_recovery(
    payload: &[u8],
    offset: usize,
    layout: &I0Layout,
    arch: &FilmArchetype,
    context: &NativeReaderContext,
) -> Result<ContextEquipmentRecoveryProbe, NativeReaderProfileError> {
    let mut out = ContextEquipmentRecoveryProbe::default();
    let bits = super::bits::Bits(payload);
    // Includes the header and one index; also bounds the offset arithmetic below.
    if offset.checked_add(27).is_none_or(|end| end > bits.len()) {
        return Ok(out);
    }
    let mut ids = Vec::new();
    let start;
    out.dense = bits.read(offset + 17, 1) == Some(1);
    if !out.dense {
        let n = bits.read(offset + 18, 3).unwrap() as usize;
        if !(2..=7).contains(&n) || offset + 21 + 6 * n > bits.len() {
            return Ok(out);
        }
        for i in 0..n {
            let id = bits.read(offset + 21 + 6 * i, 6).unwrap() as u8;
            if ids.last().is_some_and(|&prev| prev >= id) {
                return Ok(out);
            }
            ids.push(id);
        }
        if ids[0] == 0 || !ids.contains(&48) {
            return Ok(out);
        }
        start = offset + 21 + 6 * n;
    } else {
        let i0 = offset + 82;
        let Some(total) = usize::try_from(layout.total_bits()).ok() else {
            return Ok(out);
        };
        if !(4..=36).contains(&layout.gate_bits)
            || i0.checked_add(total).is_none_or(|end| end > bits.len())
        {
            return Ok(out);
        }
        let Some(mask) = bits.read(offset + 18, 64) else {
            return Ok(out);
        };
        ids.extend((0..64).filter(|i| mask & (1u64 << i) != 0));
        if !(2..=40).contains(&ids.len()) || ids[0] != 0 || !ids.contains(&48) {
            return Ok(out);
        }
        if bits.read(i0, 4) != Some(0)
            || bits.read(i0 + 4, layout.gate_bits - 4) != Some(u64::from(layout.region))
        {
            return Ok(out);
        }
        start = i0 + total + 2;
        ids.remove(0);
    }
    let latest = Arc::new(Mutex::new(None));
    let captured = latest.clone();
    let observer = NativeFilmObserver::default();
    observer.set_hook(
        NativeHookKind::AbilitySet,
        Some(Arc::new(move |v| {
            if let NativeHookPublication::Component(value) = v {
                *captured.lock().unwrap() = Some(value.clone());
            }
        })),
    );
    let reader_context = NativeReaderContext {
        profile: context.profile.clone(),
        observer: Some(observer),
    };
    let mut at = start as i64;
    for id in ids {
        let Some(name) = arch
            .components
            .get(usize::from(id))
            .filter(|name| !name.is_empty())
        else {
            break;
        };
        let mut reader = NativeFilmReader::with_context(payload, reader_context.clone());
        reader.set_native_bit_position(at);
        let (status, component) = reader.read_component(
            name,
            arch.levels.get(usize::from(id)).copied().unwrap_or(0),
            35,
        )?;
        at = component.end_bit;
        let in_bounds = at >= 0 && at <= bits.len() as i64;
        out.components.push(EquipmentRecoveryComponentRead {
            component_index: id,
            status,
            in_bounds,
            component,
        });
        if status != Some(true) || !in_bounds {
            break;
        }
        if id == 48 {
            out.reached_target = true;
            break;
        }
    }
    out.observation = latest.lock().unwrap().take();
    Ok(out)
}
