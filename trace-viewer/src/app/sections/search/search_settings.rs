use crate::app::{components::ValidatedInput, sections::search::context::SearchLevelContext};
use chrono::TimeDelta;
use leptos::{IntoView, component, either::EitherOf3, prelude::*, view};
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[component]
pub(crate) fn SearchSettings() -> impl IntoView {
    let search_level_context = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    let advance_time = RwSignal::<i64>::new(1000);
    view! {
        <div class = "division">
            <div class = "control-grid">
                <div class = "control">
                    <label for = "date"> "Date:" </label>
                    <ValidatedInput name = "date" id = "date" datatype = "date" bind_to_signal = search_level_context.date/>
                </div>
                <div class = "control">
                    <label for = "time"> "Time:" </label>
                    <ValidatedInput name = "time" id = "time" datatype = "text" bind_to_signal = search_level_context.time />
                </div>
                <div class = "control">
                    <label for = "time"> "Advance Time (ms):" </label>
                    <ValidatedInput name = "advance-time" id = "advance-time" datatype = "number" bind_to_signal = advance_time />
                </div>
            </div>
        </div>

        <div class = "division">
            <div class = "control-grid">
                <SearchMode />
                <div class = "control">
                    <label for = "number"> "Number:" </label>
                    <ValidatedInput name = "time" id = "time" datatype = "number" bind_to_signal = search_level_context.number />
                </div>
                <Show when = move|| matches!(search_level_context.search_mode.get(), SearchMode::Dragnet)>
                    <div class = "control">
                        <label for = "backstep"> "Backstep:" </label>
                        <ValidatedInput name = "backstep" id = "backstep" datatype = "number" bind_to_signal = search_level_context.backstep />
                    </div>
                    <div class = "control">
                        <label for = "forward-dist"> "Forward Distance:" </label>
                        <ValidatedInput name = "forward-dist" id = "forward-dist" datatype = "number" bind_to_signal = search_level_context.forward_distance />
                    </div>
                </Show>
            </div>
        </div>

        <div class = "division">
            <div class = "control-grid">
                <MatchCriteria />
                <MatchBy />
            </div>
        </div>
    }
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub(crate) enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
    #[strum(to_string = "Dragnet Search")]
    Dragnet,
}

#[component]
pub(crate) fn SearchMode() -> impl IntoView {
    let search_level_context = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    let search_mode = search_level_context.search_mode;

    view! {
        <div class = "control">
            <label class = "panel-item" for = "search-mode">
                "Search Mode: "
            </label>
            <select name = "search-mode" id = "search-mode"
                on:change = move |ev|
                    search_mode.set(
                        event_target_value(&ev)
                            .parse()
                            .expect("SearchMode value should parse, this should never fail.")
                        )>
                <For each = SearchMode::iter
                    key = ToOwned::to_owned
                    let(mode)
                >
                    <option selected={search_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                </For>
            </select>
        </div>
    }
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub(crate) enum SearchBy {
    #[default]
    #[strum(to_string = "Match All")]
    All,
    #[strum(to_string = "By Channels")]
    ByChannels,
    #[strum(to_string = "By Digitiser Ids")]
    ByDigitiserIds,
}

#[component]
pub(crate) fn MatchCriteria() -> impl IntoView {
    let search_level_context = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    let search_by = search_level_context.search_by;

    view! {
        <div class = "control">
            <label for = "match-criteria">
                "Match Criteria: "
            </label>
            <select name = "match-criteria" id = "match-criteria" class = "panel-item"
                data-tooltip = "Choose which criteria to match on: by channel's contained, or by digitiser id."
                on:change = move |ev| search_by.set(
                    event_target_value(&ev)
                        .parse()
                        .expect("SearchBy value should parse, this should never fail.")
                )
            >
                <For
                    each = SearchBy::iter
                    key = ToOwned::to_owned
                    let(mode)
                >
                    <option selected={search_by.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                </For>
            </select>
        </div>
    }
}

#[component]
pub(crate) fn MatchBy() -> impl IntoView {
    let search_level_context = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    fn parse_to_list<T: ToString>(list: &[T]) -> String {
        list.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn parse_from_list<T: FromStr>(str: String) -> Vec<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        str.split(",")
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()
            .expect("This should do some validation checking. TODO")
    }

    move || match search_level_context.search_by.get() {
        SearchBy::All => EitherOf3::A(()),
        SearchBy::ByChannels => EitherOf3::B(view! {
            <div class = "control">
                <label for = "channels">
                "Channels:"
                </label>
                <input class = "panel-item" type = "text" id = "channels"
                    value = move ||parse_to_list(&search_level_context.channels.get())
                    on:change = move |ev|search_level_context.channels.set(parse_from_list(event_target_value(&ev).parse().expect("msg")))
                />
            </div>
        }),
        SearchBy::ByDigitiserIds => EitherOf3::C(view! {
            <div class = "control">
                <label for = "digitiser-ids">
                    "Digitiser IDs:"
                </label>
                <input class = "panel-item" type = "text" id = "digitiser-ids"
                    value = move ||parse_to_list(&search_level_context.digitiser_ids.get())
                    on:change = move |ev|search_level_context.digitiser_ids.set(parse_from_list(event_target_value(&ev).parse().expect("msg")))
                />
            </div>
        }),
    }
}
