use patternfly_yew::prelude::*;
use yew::prelude::*;

use crate::hook::use_open;

#[function_component]
pub fn Github() -> Html {
    let callback_github: Callback<MouseEvent, _> =
        use_open("https://github.com/Usioumeo/Thesys", "_blank");
    html!(
        <Button variant={ButtonVariant::Plain} icon={Icon::Github} onclick={callback_github}/>
    )
    
    
}