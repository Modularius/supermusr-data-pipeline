use crate::app::components::ValidatedInput;
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub fn BrokerPoller(poll_broker_timeout_ms: RwSignal<u64>) -> impl IntoView {
    view! {
        <div class = "division">
            <div class = "control-grid">
                <div class = "control">
                    <label for = "poll_broker_timeout_ms"> "Poll Broker Timeout (ms):" </label>
                    <ValidatedInput name = "poll_broker_timeout_ms" id = "poll_broker_timeout_ms" datatype = "number" bind_to_signal = poll_broker_timeout_ms />
                </div>
            </div>
        </div>
    }
}
