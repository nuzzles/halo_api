//! Test-only capture of native-equivalent structured tracing events.
pub(crate) fn capture_logs(action: impl FnOnce()) -> Vec<serde_json::Value> {
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = events.clone();
    capture_logs_observed(action, move |event| captured.lock().unwrap().push(event));
    let mut events = events.lock().unwrap();
    std::mem::take(&mut *events)
}

pub(crate) fn capture_logs_observed(
    action: impl FnOnce(),
    observe: impl Fn(serde_json::Value) + Send + Sync + 'static,
) {
    use tracing::{Event, Metadata, Subscriber, field, span};
    struct Capture(Box<dyn Fn(serde_json::Value) + Send + Sync>);
    struct Fields(serde_json::Map<String, serde_json::Value>);
    impl field::Visit for Fields {
        fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
            let key = if field.name() == "message" {
                "msg"
            } else {
                field.name()
            };
            self.0.insert(key.into(), format!("{value:?}").into());
        }
        fn record_bool(&mut self, field: &field::Field, value: bool) {
            self.0.insert(field.name().into(), value.into());
        }
        fn record_str(&mut self, field: &field::Field, value: &str) {
            self.0.insert(field.name().into(), value.into());
        }
        fn record_u64(&mut self, field: &field::Field, value: u64) {
            self.0.insert(field.name().into(), value.into());
        }
        fn record_i64(&mut self, field: &field::Field, value: i64) {
            self.0.insert(field.name().into(), value.into());
        }
    }
    impl Subscriber for Capture {
        fn enabled(&self, _: &Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
            span::Id::from_u64(1)
        }
        fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
        fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
        fn enter(&self, _: &span::Id) {}
        fn exit(&self, _: &span::Id) {}
        fn event(&self, event: &Event<'_>) {
            let mut fields = Fields(serde_json::Map::new());
            event.record(&mut fields);
            fields
                .0
                .insert("level".into(), event.metadata().level().to_string().into());
            (self.0)(fields.0.into());
        }
    }
    tracing::subscriber::with_default(Capture(Box::new(observe)), action);
}

pub(crate) fn capture_log(action: impl FnOnce()) -> serde_json::Value {
    let mut events = capture_logs(action);
    assert_eq!(events.len(), 1);
    events.pop().unwrap()
}
