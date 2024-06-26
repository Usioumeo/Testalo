use patternfly_yew::prelude::*;
use reqwest::Url;
use syn::parse_file;
use wasm_bindgen_futures::JsFuture;
use yew::{platform::spawn_local, prelude::*};
use yew_hooks::prelude::*;
use yew_more_hooks::hooks::use_async_with_cloned_deps;
use crate::components::problem_selector::ProblemSelector;
#[derive(Clone, Debug, PartialEq)]

enum DropContent {
    None,
    Text(String),
}

impl From<String> for DropContent {
    fn from(value: String) -> Self {
        if value.is_empty() {
            Self::None
        } else {
            Self::Text(value)
        }
    }
}

impl std::fmt::Display for DropContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Text(_) => f.write_str("User Input"),
            Self::None => Ok(()),
        }
    }
}


#[function_component]
pub fn RustInput() -> Html {
    let node = use_node_ref();

    let drop_content: UseStateHandle<String> = use_state(|| "fn main(){}".to_string());
    let selected_problem:  UseStateHandle<Option<String>>  = use_state(|| None);
    let drop = use_drop_with_options(
        node.clone(),
        UseDropOptions {
            onfiles: {
                let drop_content = drop_content.clone();
                Some(Box::new(move |files, _data_transfer| {
                    if files.len() == 1 {
                        drop_content.set(files[0].to_string().into());
                    }
                }))
            },
            ontext: {
                let drop_content = drop_content.clone();
                Some(Box::new(move |text, _data_transfer| {
                    drop_content.set(text);
                }))
            },
            ..Default::default()
        },
    );

    let error: UseStateHandle<Option<String>> = use_state(|| None);

    let onclear = use_callback(drop_content.clone(), |_, drop_content| {
        drop_content.set(String::new())
    });
    let submission: UseStateHandle<Option<Vec<(String, String)>>> = use_state(|| None);
    let _ = {
        let backdrop = use_backdrop();
        let submission = submission.clone();
        //let subm = (*submission).clone();
        use_async_with_cloned_deps(|(submission, backdrop)|async move {
            let mut sub = (*submission).clone();
            submission.set(None);
            let Some(s) = sub.take() else{return Err("".to_string())};
            let local = web_sys::window().unwrap().location().origin().unwrap();
            let local = Url::parse(&local).unwrap();

            let client = reqwest::Client::new();
            let request = client.post(local.join("/submit").unwrap()).form(&s).build().unwrap();
            let t = client.execute(request).await.map_err(|x| x.to_string())?;
            let result = t.text().await.map_err(|x| x.to_string())?;
            backdrop.as_ref().unwrap().open(html!({
                html!(
                    <Bullseye plain=true>
                        <Modal
                            title="Submittion Result"
                            variant={ModalVariant::Large}
                        >
                            <CodeBlock>
                                <CodeBlockCode>
                                    {result }
                                </CodeBlockCode>
                            </CodeBlock>
                        </Modal>
                    </Bullseye>
                )
            }));
            Ok(()) 
        }, (submission, backdrop))
    };
    let onsubmit = {
        let drop_content = drop_content.clone();
        let error: UseStateHandle<Option<String>> = error.clone();
        let selected_problem = selected_problem.clone();
        //let processing = processing.clone();
        Callback::from(move |_| {
            if drop_content.is_empty(){
                error.set(Some("Empty file".to_string()));
                return;
            }
            match parse_file(&drop_content) {
            Ok(x) => {
                // ok so we can send to server

                // nice formatting
                let source = prettyplease::unparse(&x);
                let Some(selected_problem) = (*selected_problem).clone() else {
                    error.set(Some("Problem not selected".to_string()));
                    return;
                };
                submission.set(Some(vec![("problem".to_string(), selected_problem.clone()), ("source".to_string(), source.clone())]));

                //send it
                /*let form = MultipartBuilder::new()
                .add_text("problem", &selected_problem)
                .add_text("source", &source);
                let res = ehttp::Request::multipart("/submit", form);
                
                ehttp::fetch(res, move |result: ehttp::Result<ehttp::Response>| {
                    //let backdrop = use_backdrop();
                    info!("Status code: {:?}", result.unwrap().status);
                   
                    //error.set(None);
                }*/
                
            }
            Err(e) => {
                let span = e.span().start();
                let e = format!("{}::{}: {}", span.line, span.column, e);
                error.set(Some(e));
            }
        }})

        /*if let Some((data, backdrop)) = processing.data().zip(backdrop.as_ref()) {

        }*/
    };

    let file_input_ref = use_node_ref();
    let onopen = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(ele) = file_input_ref.cast::<web_sys::HtmlElement>() {
                ele.click();
            }
        })
    };
    let onchange_open = {
        let file_input_ref = file_input_ref.clone();
        let drop_content = drop_content.clone();
        Callback::from(move |_| {
            if let Some(ele) = file_input_ref.cast::<web_sys::HtmlInputElement>() {
                let files = ele
                    .files()
                    .map(|files| {
                        let mut r =
                            Vec::with_capacity(files.length().try_into().unwrap_or_default());
                        for i in 0..files.length() {
                            Extend::extend(&mut r, files.get(i));
                        }
                        r
                    })
                    .unwrap_or_default();

                if files.len() == 1 {
                    let drop_content = drop_content.clone();
                    spawn_local(async move {
                        if let Ok(x) = JsFuture::from(files[0].text()).await {
                            let val = x.as_string().unwrap_or_default();
                            drop_content.set(val);
                        }
                    });
                }
            }
        })
    };

    let oninput_text = use_callback(drop_content.clone(), |new_text, drop_content| {
        drop_content.set(new_text)
    });
    
    let choose_problem = {
        let selector = selected_problem.clone();
        let onselect: Callback<String> =  {
            let selected = selector.clone();
            Callback::from( 
            move |v: String| { 
                selected.set(Some(v));
            })
        };
        html!(
            <ProblemSelector {onselect} selected={selector.as_ref().map(|x| x.clone())}/>

        )
    };
    


    // generate html
    html!(<>
        
        <div ref={node.clone()}>
        <Form>
            <FormGroup>
                <FileUpload
                    drag_over={*drop.over}
                >
                    //can upload inside  
                    <FileUploadSelect>
                        <InputGroup>
                            {choose_problem}
                            <TextInput readonly=true value={""}/>
                            <input ref={file_input_ref.clone()} style="display: none;" type="file" onchange={onchange_open} />
                            <Button
                                variant={ButtonVariant::Control}
                                onclick={onopen}
                            >
                                {"Open"}
                            </Button>
                            <Button
                                variant={ButtonVariant::Control}
                                onclick={onsubmit}
                            >
                                {"Submit"}
                            </Button>
                            <Button
                                variant={ButtonVariant::Control}
                                onclick={onclear}
                                >
                                {"Clear"}
                            </Button>
                        </InputGroup>
                    </FileUploadSelect>
                    <FileUploadDetails
                        invalid={error.is_some()}
                    >
                        <TextArea
                            value={(drop_content.to_string()).clone()}
                            resize={ResizeOrientation::Vertical}
                            onchange={oninput_text}
                            state={ if error.is_some() {InputState::Error} else {InputState::Default} }
                        />
                    </FileUploadDetails>
                </FileUpload>

                {error.as_ref().unwrap_or(&String::new()).clone()}
            </FormGroup>
        </Form>
        </div>
        </>
    )
}
