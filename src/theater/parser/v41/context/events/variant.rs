//! Selected schema semantics over the retained Bond AST. No defaults enter Film.
use crate::theater::parser::v41::chunks::registry::source::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct VariantContext {
    pub engine_type: Option<i32>,
    pub killcam: Option<bool>,
    pub play_of_game: Option<bool>,
}
impl VariantContext {
    /// Both writer predicates must be false regardless of unrecorded settings.
    pub(crate) fn excludes_kill_tail(self) -> bool {
        (self.killcam == Some(false) || self.engine_type == Some(1))
            && self.play_of_game == Some(false)
    }
}

pub(crate) fn recorded_variant(registry: &FilmRegistryRead) -> VariantContext {
    let Some(header) = registry.bootstrap_header.as_ref() else {
        return VariantContext::default();
    };
    let Some(identity) = header.identification.as_ref() else {
        return VariantContext::default();
    };
    let Some(options) = identity.game_options.as_ref() else {
        return VariantContext::default();
    };
    if header.stop != BootstrapHeaderStop::Complete
        || identity.stop != BootstrapIdentificationStop::Complete
        || options.stop != GameOptionsStop::Complete
    {
        return VariantContext::default();
    }
    let mut messages = options.entries.iter().filter_map(|entry| match entry {
        GameOptionsEntry::Bond {
            kind: GameOptionsBondKind::Variant,
            read,
        } => Some(read),
        _ => None,
    });
    let Some(read) = messages.next() else {
        return VariantContext::default();
    };
    if messages.next().is_some() || read.stop != BondStop::Complete {
        return VariantContext::default();
    }
    project(&read.root).unwrap_or_default()
}

/// IDs belong to their inheritance layer. The final segment is this schema's
/// own fields; an inherited field with the same ID cannot supply its value.
fn field(value: &BondValue, id: u16) -> Result<Option<&BondValue>, ()> {
    let BondValueBody::Struct { fields, .. } = &value.body else {
        return Err(());
    };
    if value.wire_type != 10 {
        return Err(());
    }
    let start = fields
        .iter()
        .rposition(|f| f.wire_type == Some(1))
        .map_or(0, |i| i + 1);
    let mut found = fields[start..]
        .iter()
        .filter(|f| f.wire_type.is_some_and(|t| t > 1) && f.id == Some(id));
    let Some(f) = found.next() else {
        return Ok(None);
    };
    if found.next().is_some() {
        return Err(());
    }
    f.value.as_ref().map(Some).ok_or(())
}
fn child(value: &BondValue, id: u16) -> Result<Option<&BondValue>, ()> {
    let child = field(value, id)?;
    if child.is_some_and(|c| c.wire_type != 10) {
        return Err(());
    }
    Ok(child)
}
fn int32(value: &BondValue, id: u16) -> Result<i32, ()> {
    let Some(value) = field(value, id)? else {
        return Ok(0);
    };
    if value.wire_type != 16 {
        return Err(());
    }
    let BondValueBody::Integer(raw) = &value.body else {
        return Err(());
    };
    let raw = u32::try_from(raw.value.ok_or(())?).map_err(|_| ())?;
    Ok((raw >> 1) as i32 ^ -((raw & 1) as i32))
}
fn boolean(value: &BondValue, id: u16) -> Result<bool, ()> {
    let Some(value) = field(value, id)? else {
        return Ok(false);
    };
    if value.wire_type != 2 {
        return Err(());
    }
    let BondValueBody::Fixed(bits) = &value.body else {
        return Err(());
    };
    Ok(bits.raw.as_ref().ok_or(())?.low_u64() != 0)
}
fn project(root: &BondValue) -> Result<VariantContext, ()> {
    // The writer selects one of four optional variant instances. Do not merge
    // conflicting instances or let later fields overwrite earlier evidence.
    let mut selected = None;
    for id in 0..4 {
        let Some(list) = field(root, id)? else {
            continue;
        };
        let BondValueBody::Sequence {
            header,
            count,
            values,
            ..
        } = &list.body
        else {
            return Err(());
        };
        if list.wire_type != 11 || header.raw.as_ref().ok_or(())?.low_u64() & 31 != 10 {
            return Err(());
        }
        match count {
            Some(0) if values.is_empty() => {}
            Some(1) if values.len() == 1 && selected.is_none() => selected = values.first(),
            _ => return Err(()),
        }
    }
    let Some(v) = selected else {
        return Ok(VariantContext::default());
    };
    let engine_type = child(v, 0)?.map(|v| int32(v, 0)).transpose()?;
    let playback = child(v, 1)?
        .map(|v| child(v, 0))
        .transpose()?
        .flatten()
        .map(|v| child(v, 5))
        .transpose()?
        .flatten();
    Ok(VariantContext {
        engine_type,
        killcam: playback.map(|v| boolean(v, 0)).transpose()?,
        play_of_game: playback.map(|v| boolean(v, 2)).transpose()?,
    })
}

#[cfg(test)]
mod tests;
