use std::{cell::RefCell, ops::DerefMut};

use dom::create_html_element;
use match_list::MatchList;
use model::Model;
use outline::Outline;
use round_robin_standings::RoundRobinStandings;
use round_robin_table::RoundRobinTable;
use ui::{Ui, UiElement};
use web_sys::{window};

mod tournament;
mod dom;
mod round_robin_table;
mod round_robin_standings;
mod match_list;
mod outline;
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

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logging");

    with_globals(|model, ui| {
        // let tournament_id = match model.get_tournaments().iter().find(|t| t.1.name == "LCS") {
        //     Some((tid, _)) => *tid,
        //     None => model.add_tournament("LCS".to_string()),
        // };
        // let stage_id = match model.get_tournament(tournament_id).unwrap().stages.iter().find(|t| t.1.name == "Group Stage") {
        //     Some((sid, _)) => *sid,
        //     None => model.add_stage(tournament_id, "Group Stage".to_string()).unwrap(),
        // };

        let outline = Outline::new(ui.get_next_id(), model);
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&outline.get_div()).expect("Failed to add div");
        let outline_id = outline.get_id();
        ui.add_element(UiElement::Outline(outline));

        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&create_html_element("hr")).expect("Failed to add element");

        let standings = RoundRobinStandings::new(ui.get_next_id(), model, outline_id);
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&standings.get_dom_table()).expect("Failed to insert table");
        ui.add_element(UiElement::RoundRobinStandings(standings));

        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&create_html_element("hr")).expect("Failed to add element");

        let table = RoundRobinTable::new(ui.get_next_id(), model, outline_id);
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&table.get_dom_table()).expect("Failed to insert table");
        ui.add_element(UiElement::RoundRobinTable(table));

        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&create_html_element("hr")).expect("Failed to add element");

        let match_list = MatchList::new(ui.get_next_id(), model, outline_id);
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&match_list.get_dom_table()).expect("Failed to insert table");
        ui.add_element(UiElement::MatchList(match_list));
    });
}
