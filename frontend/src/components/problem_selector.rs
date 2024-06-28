use patternfly_yew::prelude::*;
use reqwest::Url;
use yew::prelude::*;
use yew_hooks::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ProblemSelectorProperties
{
    #[prop_or(Callback::from(|_: String|{}))]
    pub onselect: Callback<String>,
    #[prop_or(None)]
    pub selected: Option<String>,
}

#[function_component]
pub fn ProblemSelector(prop: &ProblemSelectorProperties) -> Html {
    
    let get_problems: UseAsyncHandle<Vec<String>, String> = {
        use_async_with_options(async move{
            let local = web_sys::window().unwrap().location().origin().unwrap();
            let local = Url::parse(&local).unwrap();
            
            let client = reqwest::Client::new();
            
            let request = client.get(local.join("/list_problems").unwrap()).build().map_err(|x| x.to_string())?;
            let val = client.execute(request).await.map_err(|x| x.to_string())?;
            let val: Vec<String>  = serde_json::from_str(&val.text().await.map_err(|x| x.to_string())?).map_err(|x| x.to_string())?;
            Ok(val)
            
        }, UseAsyncOptions::enable_auto())
    };
    
    //let t = prop.onselect;
    html!(
        <SimpleSelect<String>
            placeholder="Pick a value"
            selected={prop.selected.clone()}
            entries={get_problems.data.iter().flatten().cloned().collect::<Vec<String>>()}
            onselect={prop.onselect.clone()}
        />
    )
}