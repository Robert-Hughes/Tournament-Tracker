use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

pub fn create_html_element(tag_name: &str) -> HtmlElement {
    create_element::<HtmlElement>(tag_name)
}

pub fn create_element<T: JsCast>(tag_name: &str) -> T {
    web_sys::window().expect("Missing window")
        .document().expect("Missing document")
        .create_element(tag_name).expect("Failed to create element")
        .dyn_into().expect("Failed to cast to requested type")
}