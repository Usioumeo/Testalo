use crate::components::exercise_input::RustInput;
use crate::components::github::Github;
use crate::components::DarkModeSwitch;
use crate::pages::login::LoginPage;
use crate::pages::register::RegisterPage;
use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_nested_router::prelude::Switch as RouterSwitch;
use yew_nested_router::{Router, Target};
#[derive(Debug, Default, Clone, PartialEq, Eq, Target)]

pub enum AppRoute {
    //Index,

    Exercise,
    #[default]
    Login,
    Register,
}

impl AppRoute {
    fn switch(self) -> Html {
        match self {
           // AppRoute::Index => html! {<AppPage><Counter/></AppPage>},

            AppRoute::Exercise => html!(<AppPage><RustInput/></AppPage>),
            AppRoute::Login => html!(<AppPage><LoginPage/></AppPage>),
            AppRoute::Register => html!(<AppPage><RegisterPage/></AppPage>),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct PageProps {
    pub children: Children,
}

#[function_component(AppPage)]
fn page(props: &PageProps) -> Html {
    let sidebar = html_nested! {
        <PageSidebar>
            <Nav>
                <NavList>
                    <NavRouterItem<AppRoute> to={AppRoute::Exercise}>{"Submit"}</NavRouterItem<AppRoute>>
                    <NavExpandable title="Logins">
                        //<NavRouterItem<AppRoute> to={AppRoute::Index}>{"Index"}</NavRouterItem<AppRoute>>
                        
                        <NavRouterItem<AppRoute> to={AppRoute::Login}>{"Login"}</NavRouterItem<AppRoute>>
                        <NavRouterItem<AppRoute> to={AppRoute::Register}>{"Register"}</NavRouterItem<AppRoute>>
                    </NavExpandable>

                </NavList>
            </Nav>
        </PageSidebar>
    };

    let tools = html!(
        <Toolbar full_height=true>
            <ToolbarContent>
                <ToolbarGroup
                    modifiers={ToolbarElementModifier::Right.all()}
                    variant={GroupVariant::IconButton}
                >
                    <ToolbarItem>
                        <DarkModeSwitch/>
                    </ToolbarItem>
                    <ToolbarItem>
                        <Github/>
                    </ToolbarItem>
                    <ToolbarItem>
                        <Dropdown
                            position={Position::Right}
                            icon={Icon::QuestionCircle}
                            variant={MenuToggleVariant::Plain}
                        >
                            //<MenuAction onclick={onabout}>{"About"}</MenuAction>
                        </Dropdown>
                    </ToolbarItem>
                </ToolbarGroup>
            </ToolbarContent>
        </Toolbar>
    );
    html! (
        <Page {sidebar} {tools}>
            { for props.children.iter() }
        </Page>
    )
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BackdropViewer>
            <ToastViewer>
                <Router<AppRoute> default={AppRoute::Login}>
                    <RouterSwitch<AppRoute> render={AppRoute::switch} />
                </Router<AppRoute>>
            </ToastViewer>
        </BackdropViewer>
    }
}
