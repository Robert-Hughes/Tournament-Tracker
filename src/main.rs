use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlInputElement, HtmlElement};

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");

    for _i in 0..5 {
        let p = web_sys::window().unwrap().document().unwrap().create_element("p").unwrap();
        let p : web_sys::HtmlElement = p.dyn_into().unwrap();
        p.set_inner_html("Hellooo");
        let f = Closure::<dyn Fn()>::new(|| { log::info!("Clickety click!"); });
        p.set_onclick(Some(f.as_ref().unchecked_ref()));
        web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&p).unwrap();

        std::mem::forget(f);
    }

    let input: HtmlInputElement = web_sys::window().unwrap().document().unwrap().create_element("input").unwrap().dyn_into().unwrap();
    input.set_value("something");
    web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&input).unwrap();

    let button: HtmlElement = web_sys::window().unwrap().document().unwrap().create_element("button").unwrap().dyn_into().unwrap();
    button.set_inner_text("Click");
    web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&button).unwrap();
    let f = Closure::<dyn Fn()>::new(move || {
        let p = web_sys::window().unwrap().document().unwrap().create_element("p").unwrap();
        let p : web_sys::HtmlElement = p.dyn_into().unwrap();
        p.set_inner_html(&input.value());
        web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&p).unwrap();
    });

    button.set_onclick(Some(f.as_ref().unchecked_ref()));
    std::mem::forget(f);



}
