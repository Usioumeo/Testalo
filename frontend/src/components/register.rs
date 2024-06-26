use std::collections::HashMap;

use ehttp::multipart::MultipartBuilder;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component]
pub fn RegisterComponent() -> Html {
    let username = use_state_eq(String::new);
    let password = use_state_eq(String::new);
    let toaster = use_toaster();

    let onchangeusername = {
        let username = username.clone();
        Callback::from(move |value| {
            username.set(value);
        })
    };

    let onchangepassword = {
        let password = password.clone();
        Callback::from(move |value| {
            password.set(value);
        })
    };

    let onsubmit = {
        let toaster = toaster.clone();
        let username = username.clone();
        let password = password.clone();
        Callback::from(move |_| {
            let mut params = HashMap::new();
            params.insert("username".to_string(), username.to_string());
            params.insert("password".to_string(), password.to_string());

            let form = MultipartBuilder::new()
                .add_text("username", &username)
                .add_text("password", &password);
            let res = ehttp::Request::multipart("/register", form);

            ehttp::fetch(res, move |result: ehttp::Result<ehttp::Response>| {
                println!("Status code: {:?}", result.unwrap().status);
            });

            if let Some(toaster) = &toaster {
                toaster.toast(format!(
                    "Register - Username: {}, Password: {}",
                    &*username, &*password
                ));
            }
        })
    };
    let title = html_nested! {<Title size={Size::XXLarge}>{"Register a new account"}</Title>};
    /*let navigator = use_navigator().unwrap();
    let onclick = Callback::from(move |_| navigator.push(&AppRoute::Login));*/
    html! {
        <>
            <ToastViewer>
                <Background/>
                <Login>
                    <LoginMain>
                        <LoginMainHeader
                            {title}
                            description="Enter the credentials to your account right here."
                        />
                        <LoginMainBody>
                            <Form {onsubmit} method="dialog">
                                <FormGroup label="Username">
                                    <TextInput required=true name="username" onchange={onchangeusername} value={(*username).clone()} />
                                </FormGroup>
                                <FormGroup label="Password">
                                    <TextInput required=true name="password" r#type={TextInputType::Password} onchange={onchangepassword} value={(*password).clone()} />
                                </FormGroup>
                                <ActionGroup>
                                    <Button label="Register" r#type={ButtonType::Submit} variant={ButtonVariant::Primary}/>
                                </ActionGroup>
                                //<LoginMainFooterLink {onclick} target="_blank">{"Or login if you already have an account"}</LoginMainFooterLink>

                                </Form>
                        </LoginMainBody>
                    </LoginMain>
                </Login>
            </ToastViewer>
        </>
    }
    // <Link<AppRoute> callback={AppRoute::Login}>{"test"} </Link<AppRoute>>
    //
}
