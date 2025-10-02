use crate::app::{
    TopLevelContext,
    components::Section,
    sections::broker_poll::{broker_control::BrokerPoller, broker_info::DisplayBrokerInfo},
    server_functions::PollBroker,
};
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn BrokerSection() -> impl IntoView {
    let default_data = use_context::<TopLevelContext>()
        .expect("ClientSideData should be provided, this should never fail.")
        .client_side_data
        .default_data;

    let poll_broker_timeout_ms = RwSignal::new(default_data.poll_broker_timeout_ms);

    let poll_broker_action = ServerAction::<PollBroker>::new();
    view! {
        <Section text = "Broker" id = "broker">
            <BrokerPoller poll_broker_timeout_ms/>
            <DisplayBrokerInfo poll_broker_action />

            <input type = "button" class = "poll-broker-button" value = "Poll Broker" on:click = move |_| {
                poll_broker_action.dispatch(PollBroker { poll_broker_timeout_ms: poll_broker_timeout_ms.get() });
            }/>
        </Section>
    }
}
