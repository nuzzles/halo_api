//! Managed navigation filters, preserving every tagged payload and ordering field.
use super::Reader;

pub(super) fn component(r: &mut Reader<'_>, name: &str, level: u32) -> Option<bool> {
    let distance = name == "managed-navpoint-visibility-distance-filters-component";
    let boolean = matches!(
        name,
        "managed-navpoint-visible-offscreen-filters-component"
            | "managed-navpoint-can-be-occluded-filters-component"
    );
    match name {
        "managed-navpoint-flags-component" => {
            r.r("flags", 8)?;
        }
        "managed-navpoint-formatted-text-component" => {
            let count = r.r("entries.count", 8)?;
            for i in 0..count {
                r.r(&format!("entries[{i}].string_id"), 32)?;
                let first = r.fields.len();
                super::basic::formatted_text(r)?;
                for field in &mut r.fields[first..] {
                    field.name = format!("entries[{i}].{}", field.name);
                }
            }
        }
        "managed-navpoint-visibility-distance-filters-component"
        | "managed-navpoint-visible-offscreen-filters-component"
        | "managed-navpoint-can-be-occluded-filters-component"
        | "managed-navpoint-visibility-filter-component"
        | "managed-navpoint-docking-filter-component" => {
            let recent = level > if distance { 2 } else { 1 };
            let mask = r.r("filters.mask", 4)?;
            r.r("filters.flag", if recent { 1 } else { 32 })?;
            for i in 0..4 {
                if mask & (1 << i) == 0 {
                    continue;
                }
                let prefix = format!("filters[{i}]");
                let tag = r.r(&format!("{prefix}.tag"), 4)?;
                if tag == 0 {
                    continue;
                }
                r.bit(&format!("{prefix}.flag"))?;
                if !payload(r, &prefix, tag)? {
                    return Some(false);
                }
            }
            let count = mask.count_ones() as usize;
            if distance {
                r.words("distance", 2, 16)?;
                for i in 0..count {
                    r.words(&format!("distances[{i}]"), 2, 16)?;
                }
            } else if boolean {
                r.bit("value")?;
                r.words("values", count, 1)?;
            }
            if distance || boolean {
                if !recent {
                    r.words("legacy", count, 4)?;
                }
                r.words("order", count, if recent { 3 } else { 2 })?;
            }
        }
        _ => return Some(false),
    }
    Some(true)
}

fn payload(r: &mut Reader<'_>, prefix: &str, tag: u64) -> Option<bool> {
    let start = r.fields.len();
    match tag {
        1 => {
            r.bit("value")?;
        }
        2 | 9 => {
            r.r("value", 32)?;
        }
        3 | 7 => {
            r.r("value", 4)?;
        }
        4 => {
            r.r("value", 9)?;
        }
        5 => {
            let count = r.r("count", 3)?;
            for i in 0..count {
                r.gate(&format!("strings[{i}]"), 13, false)?;
            }
        }
        6 => {
            let count = r.r("count", 4)?;
            for i in 0..count {
                r.inline_optional_handle(&format!("references[{i}]"), 0)?;
            }
        }
        8 => {
            r.r("value", 8)?;
        }
        10 => {
            r.words("enums", 2, 4)?;
            r.r("value", 32)?;
        }
        11 => {
            r.gate("index", 5, false)?;
        }
        12..=14 => {
            r.inline_optional_handle("reference", 0)?;
            r.r("value", 32)?;
            if tag == 14 {
                r.bit("tail")?;
            }
        }
        _ => return Some(false),
    }
    for field in &mut r.fields[start..] {
        field.name = format!("{prefix}.{}", field.name);
    }
    Some(true)
}
