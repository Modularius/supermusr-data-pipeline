use leptos::{IntoView, component, prelude::*, view};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};
use strum::IntoEnumIterator;

/// Component which binds an `input` tag to a non-`String` signal.
#[component]
pub(crate) fn ValidatedInput<T: 'static>(
    name: &'static str,
    id: &'static str,
    datatype: &'static str,
    bind_to_signal: RwSignal<T>,
) -> impl IntoView
where
    RwSignal<T>: Get + Set,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Get>::Value: Clone + Display,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value: FromStr,
    <<leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value as FromStr>::Err: Debug,
{
    view! {
        <input name = name id = id type = datatype
            value = {move ||bind_to_signal.get().to_string()}
            on:change = {move |ev|bind_to_signal.set(event_target_value(&ev).parse().expect("Data should parse, this should never fail."))}
        />
    }
}

#[component]
pub(crate) fn ValidatedSelect<T>(
    name: &'static str,
    id: &'static str,
    signal: RwSignal<T>,
) -> impl IntoView
where
    RwSignal<T>: Get + Set,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Get>::Value:
        Clone + Display + PartialEq + Into<T>,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value: FromStr,
    <<leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value as FromStr>::Err: Debug,
    T: Clone + Send + PartialEq + Eq + Hash + Display + IntoEnumIterator + 'static,
    <T as IntoEnumIterator>::Iterator: Send,
{
    view! {
        <select name = name id = id
            on:change = move |ev|
                signal.set(
                    event_target_value(&ev)
                        .parse()
                        .expect("SearchMode value should parse, this should never fail.")
                    )>
            <For each = T::iter
                key = ToOwned::to_owned
                let(mode)
            >
                <option selected={signal.get().into() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
            </For>
        </select>
    }
}
