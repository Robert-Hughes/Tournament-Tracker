use std::{cell::RefCell, ops::DerefMut};

use ui::bracket_view::BracketView;
use ui::match_list::MatchList;
use model::Model;
use ui::outline::Outline;
use ui::round_robin_standings::RoundRobinStandings;
use ui::round_robin_table::RoundRobinTable;
use ui::{Ui, UiElement};
use web_sys::{window};

mod dom;
mod model;
mod ui;

//TODO: maybe have a read-only lock toggle, for entering match results vs. viewing stats etc. It's too easy to accidentally click and lose data!
//TODO: "what-if" mode where can enter potential future results to see what happens,
//  e.g. percentages of playoffs without permanently changing the already-completed matches
//TODO: round-robin diagram with arrows (like I draw on paint), useful for smaller groups e.g. 4
//TODO: import data from lolesports or lol wiki?
//TODO: highlight teams on mouse hover (synced across all the different UI elements)
//TODO: shortcut buttons for "preset" brackets, like single-elimination, double-elimination so that you don't have
//      to manually create all the fixures and lay them out

// We use some global state as callbacks from the Javascript world (e.g. click event handlers)
// will need the Model (for example), but we can't easily store a reference in the callback closure
// due to Rust's borrowing rules, as the closures are long-lived and so we would need to use something
// like reference counting, which makes the rest of the code less ergonomic and more prone to runtime panics.
thread_local! {
    static GLOBAL_MODEL: RefCell<Model> = RefCell::new(Model::load());
    static GLOBAL_UI: RefCell<Ui> = RefCell::new(Ui::new());
}

/// Gets a reference to the global state. This should be called only in "top-level" functions,
/// like main() or Javascript callback handlers. Calling it in other places may lead to panics as
/// we try to get a second mutable reference.
pub fn with_globals<F: FnOnce(&mut Model, &mut Ui) -> ()>(f: F) {
    GLOBAL_MODEL.with(|m| {
        GLOBAL_UI.with(|u| {
            let mut m = m.borrow_mut();
            let mut u = u.borrow_mut();
            f(m.deref_mut(), u.deref_mut());
        });
    });
}

fn add_ui_element(ui: &mut Ui, ui_element: UiElement, insertion_selector: &str) {
    window().expect("Missing window")
        .document().expect("Missing document")
        .query_selector(insertion_selector).expect("Selector failed").expect("Selector failed")
        .append_child(ui_element.get_root_html_element()).expect("Failed to add child");
    ui.add_element(ui_element);
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");

    with_globals(|model, ui| {
        let outline = Outline::new(ui.get_next_id(), model);
        let outline_id = outline.get_id();
        add_ui_element(ui, UiElement::Outline(outline), "#left-pane");

        let standings = RoundRobinStandings::new(ui.get_next_id(), model, outline_id);
        add_ui_element(ui, UiElement::RoundRobinStandings(standings), "#right-pane");

        let table = RoundRobinTable::new(ui.get_next_id(), model, outline_id);
        add_ui_element(ui, UiElement::RoundRobinTable(table), "#right-pane");

        let match_list = MatchList::new(ui.get_next_id(), model, outline_id);
        add_ui_element(ui, UiElement::MatchList(match_list), "#right-pane");

        let match_list = BracketView::new(ui.get_next_id(), model, outline_id);
        add_ui_element(ui, UiElement::BracketView(match_list), "#right-pane");
    });
}
