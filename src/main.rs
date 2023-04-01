use std::{cell::RefCell, ops::DerefMut};

use model::Model;
use round_robin_standings::RoundRobinStandings;
use round_robin_table::RoundRobinTable;
use ui::Ui;
use web_sys::{window};

mod tournament;
mod dom;
mod round_robin_table;
mod round_robin_standings;
mod model;
mod ui;

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
        let tournament_id = match model.get_tournaments().iter().find(|t| t.1.name == "LCS") {
            Some((tid, _)) => *tid,
            None => model.add_tournament("LCS".to_string()),
        };
        let stage_id = match model.get_tournament(tournament_id).unwrap().stages.iter().find(|t| t.1.name == "Group Stage") {
            Some((sid, _)) => *sid,
            None => model.add_stage(tournament_id, "Group Stage".to_string()).unwrap(),
        };

        let standings = Box::new(RoundRobinStandings::new(ui.get_next_id(), model, tournament_id, stage_id));
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&standings.get_dom_table()).expect("Failed to insert table");
        ui.add_element(standings);

        let table = Box::new(RoundRobinTable::new(ui.get_next_id(), model, tournament_id, stage_id));
        window().expect("Missing window").document().expect("Missing document").body().expect("Missing body").append_child(&table.get_dom_table()).expect("Failed to insert table");
        ui.add_element(table);
    });
}
