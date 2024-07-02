/* good libs
FOR MORE LINKS => https://github.com/jetli/awesome-yew


FIGATA => https://github.com/patternfly-yew/patternfly-yew
charts!!! => https://github.com/titanclass/yew-chart


material components => https://github.com/RustVis/zu
components => https://github.com/isosphere/yew-bootstrap
material components => https://github.com/angular-rust/yew-components

https://github.com/yewprint/yewprint
 */

use app::App;

mod app;
mod components;
mod hook;
mod pages;
use browser_panic_hook::{CustomBody, IntoPanicHook};
fn main() {
    console_log::init().unwrap();
    // add panic hook (if panics, it triggers this page)
    yew::set_custom_panic_hook(
        CustomBody(Box::new(|details| {
            format!(
                r#"
<div class="pf-v5-l-bullseye">
  <div class="pf-v5-l-bullseye__item">
    <div class="pf-v5-c-alert pf-m-danger" aria-label="Application panicked">
      <div class="pf-v5-c-alert__icon">
        <i class="fas fa-fw fa-exclamation-circle" aria-hidden="true"></i>
      </div>
      <p class="pf-v5-c-alert__title">
        <span class="pf-v5-screen-reader">Panick alert:</span>
        Application panicked
      </p>
      <div class="pf-v5-c-alert__description">
        <p>The application failed critically and cannot recover.</p>
        <p>Reason: <pre>{message}</pre></p>
      </div>
    </div>
  </div>
</div>
"#,
                message = details.message()
            )
        }))
        .into_panic_hook(),
    );
    yew::Renderer::<App>::new().render();
}
