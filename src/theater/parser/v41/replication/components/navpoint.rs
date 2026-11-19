//! Managed navigation filters, preserving every tagged payload and ordering field.
//! Additional layouts and interleaving. Keep legacy and versioned wire forms distinct.
use super::basic::formatted_text;
use super::reader::ComponentReader;

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str, level: u32) -> Option<bool> {
    let distance = name == "managed-navpoint-visibility-distance-filters-component";
    let boolean = matches!(
        name,
        "managed-navpoint-visible-offscreen-filters-component"
            | "managed-navpoint-can-be-occluded-filters-component"
    );
    match name {
        "managed-navpoint-position-offset" => return super::position::point(r, "position"),
        "managed-navpoint-flags-component" => {
            r.r("flags", 8)?;
        }
        "managed-navpoint-formatted-text-component" => {
            let count = r.r("entries.count", 8)?;
            for i in 0..count {
                r.r(&format!("entries[{i}].string_id"), 32)?;
                let first = r.fields.len();
                let result = formatted_text(r);
                for field in &mut r.fields[first..] {
                    field.name = format!("entries[{i}].{}", field.name);
                }
                result?;
            }
        }
        "managed-object-interaction-filter-component"
        | "managed-objective-interaction-filter-component"
        | "managed-navpoint-visibility-distance-filters-component"
        | "managed-navpoint-visible-offscreen-filters-component"
        | "managed-navpoint-can-be-occluded-filters-component"
        | "managed-navpoint-visibility-filter-component"
        | "managed-navpoint-docking-filter-component" => {
            let recent = level > if distance { 2 } else { 1 };
            let Some(mask) = filter(r, "filters", recent)? else {
                return Some(false);
            };
            let count = mask.count_ones() as usize;
            if distance {
                r.words("distance", 2, 16)?;
            }
            if boolean {
                r.bit("value")?;
            }
            if distance || boolean {
                // In legacy rows, metadata follows EACH value on wire. Grouping
                // all values first preserves the endpoint but mislabels fields.
                for i in 0..count {
                    if distance {
                        r.words(&format!("distances[{i}]"), 2, 16)?;
                    } else {
                        r.bit(&format!("values[{i}]"))?;
                    }
                    if !recent {
                        r.r(&format!("legacy[{i}]"), 4)?;
                    }
                }
                r.words("order", count, if recent { 3 } else { 2 })?;
            }
        }
        "managed-navpoint-visual-states-component" => {
            if level < 2 {
                let mask = r.r("states.mask", 8)?;
                for i in 0..8 {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    let name = format!("states[{i}]");
                    r.r(&format!("{name}.id"), 32)?;
                    let Some(mask) = filter(r, &format!("{name}.filters"), false)? else {
                        return Some(false);
                    };
                    r.r(&format!("{name}.value"), 32)?;
                    for j in 0..mask.count_ones() {
                        r.r(&format!("{name}.items[{j}].value"), 32)?;
                        r.r(&format!("{name}.items[{j}].legacy"), 4)?;
                        r.r(&format!("{name}.items[{j}].order"), 2)?;
                    }
                }
            }
        }
        name if name
            .strip_prefix("managed-navpoint-visual-state-groups-component-")
            .is_some_and(|suffix| {
                matches!(suffix, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7")
            }) =>
        {
            if r.bit("present")? {
                r.r("id", 32)?;
                let Some(mask) = filter(r, "filters", true)? else {
                    return Some(false);
                };
                r.r("value", 32)?;
                for i in 0..mask.count_ones() {
                    r.r(&format!("items[{i}].value"), 32)?;
                    r.r(&format!("items[{i}].order"), 3)?;
                }
            }
        }
        _ => return Some(false),
    }
    Some(true)
}

// Outer None is truncation; an inner None is an unsupported condition tag.
fn filter(r: &mut ComponentReader<'_>, prefix: &str, narrow: bool) -> Option<Option<u64>> {
    let mask = r.r(&format!("{prefix}.mask"), 4)?;
    r.r(&format!("{prefix}.flag"), if narrow { 1 } else { 32 })?;
    for i in 0..4 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let name = format!("{prefix}[{i}]");
        let tag = r.r(&format!("{name}.tag"), 4)?;
        if tag == 0 {
            continue;
        }
        r.bit(&format!("{name}.flag"))?;
        if !payload(r, &name, tag)? {
            return Some(None);
        }
    }
    Some(Some(mask))
}

fn payload(r: &mut ComponentReader<'_>, prefix: &str, tag: u64) -> Option<bool> {
    let start = r.fields.len();
    let result = (|| {
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
        Some(true)
    })();
    for field in &mut r.fields[start..] {
        field.name = format!("{prefix}.{}", field.name);
    }
    result
}
