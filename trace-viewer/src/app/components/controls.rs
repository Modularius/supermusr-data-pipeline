use leptos::{IntoView, component, prelude::*, view};
use std::str::FromStr;
use std::fmt::{Debug, Display};

/// Component which binds an `input` tag to a non-`String` signal.
#[component]
pub(crate) fn ValidatedInput<T: 'static>(name: &'static str, id: &'static str, datatype: &'static str, bind_to_signal: RwSignal<T>) -> impl IntoView
 where RwSignal<T>: Get + Set,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Get>::Value: Clone + Display,
    <leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value: FromStr,
    <<leptos::prelude::RwSignal<T> as leptos::prelude::Set>::Value as FromStr>::Err : Debug
{
    view!{
        <input name = name id = id type = datatype
            value = {move ||bind_to_signal.get().to_string()}
            on:change = {move |ev|bind_to_signal.set(event_target_value(&ev).parse().expect("Data should parse, this should never fail."))}
        />
    }
}