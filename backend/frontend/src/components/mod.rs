use patternfly_yew::prelude::ToolbarItem;
use yew::prelude::*;
pub mod exercise_input;
pub mod github;
pub mod login;
pub mod register;
pub mod problem_selector;
#[function_component]
pub fn DarkModeSwitch() -> Html {
    let darkmode = use_state_eq(|| {
        gloo_utils::window()
            .match_media("(prefers-color-scheme: light)")
            .ok()
            .flatten()
            .map(|m| m.matches())
            .unwrap_or_default()
    });

    // apply dark mode
    use_effect_with(*darkmode, |state| match state {
        true => gloo_utils::document_element().set_class_name("pf-v5-theme-dark"),
        false => gloo_utils::document_element().set_class_name(""),
    });

    // toggle dark mode
    let onthemeswitch = use_callback(darkmode.setter(), |state, setter| setter.set(state));
    html!(
        <ToolbarItem>
            <patternfly_yew::prelude::Switch checked={*darkmode} onchange={onthemeswitch} label="Dark Theme" />
        </ToolbarItem>
    )
}
