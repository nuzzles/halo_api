//! Runtime sink for the parser's retained native diagnostic records.
//! Callsite schemas contain only parser-defined field names, never input values.
use super::StatborgDiagnostic;
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use tracing::{
    Event, Level, Metadata,
    callsite::{Callsite, Identifier},
    field::{FieldSet, Value},
    metadata::Kind,
    subscriber::Interest,
};

struct Site(OnceLock<Metadata<'static>>);
impl Callsite for Site {
    // Dispatch checks the current subscriber, including after it changes.
    fn set_interest(&self, _: Interest) {}
    fn metadata(&self) -> &Metadata<'_> {
        self.0.get().expect("initialized diagnostic schema")
    }
}
type Schemas = BTreeMap<(String, Vec<String>), &'static Site>;

/// Internal only: schemas come from the finite set of parser diagnostics. Values
/// and message text are never interned. Each schema lives for tracing's required
/// static callsite lifetime and is reused by every film and subscriber.
pub(super) fn emit_replay_diagnostic(record: &StatborgDiagnostic) {
    static SCHEMAS: OnceLock<Mutex<Schemas>> = OnceLock::new();
    let level = match record.level.as_str() {
        "ERROR" => Level::ERROR,
        "WARN" => Level::WARN,
        "DEBUG" => Level::DEBUG,
        "TRACE" => Level::TRACE,
        _ => Level::INFO,
    };
    let names: Vec<_> = std::iter::once("message".to_owned())
        .chain(record.attributes.iter().map(|(key, _)| key.clone()))
        .collect();
    let key = (level.to_string(), names);
    let (site, created) = {
        let mut schemas = SCHEMAS.get_or_init(Mutex::default).lock().unwrap();
        if let Some(&site) = schemas.get(&key) {
            (site, false)
        } else {
            let names: &'static [&'static str] = Box::leak(
                key.1
                    .iter()
                    .map(|s| &*Box::leak(s.clone().into_boxed_str()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            let site: &'static Site = Box::leak(Box::new(Site(OnceLock::new())));
            site.0
                .set(Metadata::new(
                    "native replay diagnostic",
                    "halo_api::theater",
                    level,
                    Some(file!()),
                    None,
                    Some(module_path!()),
                    FieldSet::new(names, Identifier(site)),
                    Kind::EVENT,
                ))
                .unwrap();
            schemas.insert(key, site);
            (site, true)
        }
    };
    // Subscriber callbacks may reenter logging; never call them under the lock.
    if created {
        tracing::callsite::register(site);
    }
    let metadata = site.0.get().unwrap();
    tracing::dispatcher::get_default(|dispatch| {
        if !dispatch.enabled(metadata) {
            return;
        }
        let prepared: Vec<_> = record
            .attributes
            .iter()
            .map(|(_, v)| Prepared::from(v))
            .collect();
        let message = tracing::field::display(&record.message);
        let values: Vec<Option<&dyn Value>> = std::iter::once(Some(&message as &dyn Value))
            .chain(prepared.iter().map(|v| Some(v.value())))
            .collect();
        dispatch.event(&Event::new(
            metadata,
            &metadata.fields().value_set_all(&values),
        ));
    });
}

enum Prepared {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Text(String),
    Json(tracing::field::DisplayValue<serde_json::Value>),
}
impl From<&serde_json::Value> for Prepared {
    fn from(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Bool(v) => Self::Bool(*v),
            serde_json::Value::String(v) => Self::Text(v.clone()),
            serde_json::Value::Number(v) if v.is_i64() => Self::I64(v.as_i64().unwrap()),
            serde_json::Value::Number(v) if v.is_u64() => Self::U64(v.as_u64().unwrap()),
            serde_json::Value::Number(v) => Self::F64(v.as_f64().unwrap()),
            _ => Self::Json(tracing::field::display(v.clone())),
        }
    }
}
impl Prepared {
    fn value(&self) -> &dyn Value {
        match self {
            Self::Bool(v) => v,
            Self::I64(v) => v,
            Self::U64(v) => v,
            Self::F64(v) => v,
            Self::Text(v) => v,
            Self::Json(v) => v,
        }
    }
}
