use std::any::Any;

use indexmap::IndexMap;
use indexmap::indexmap;
use wasm_bindgen::prelude::Closure;

use crate::match_list::MatchList;
use crate::model::Model;
use crate::outline::Outline;
use crate::round_robin_standings::RoundRobinStandings;
use crate::with_globals;
use crate::{tournament::{TournamentId}, round_robin_table::RoundRobinTable};

/// Contains all the UI elements.
pub struct Ui {
    elements: IndexMap<UiElementId, UiElement>,
    next_id: UiElementId,
}

pub type UiElementId = usize;

pub enum UiElement {
    RoundRobinTable(RoundRobinTable),
    RoundRobinStandings(RoundRobinStandings),
    MatchList(MatchList),
    Outline(Outline),
}

impl UiElement {
    fn get_id(&self) -> UiElementId {
        match self {
            UiElement::RoundRobinTable(x) => x.get_id(),
            UiElement::RoundRobinStandings(x) => x.get_id(),
            UiElement::MatchList(x) => x.get_id(),
            UiElement::Outline(x) => x.get_id(),
        }
    }

    fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        match self {
            UiElement::RoundRobinTable(x) => x.tournament_changed(model, tournament_id),
            UiElement::RoundRobinStandings(x) => x.tournament_changed(model, tournament_id),
            UiElement::MatchList(x) => x.tournament_changed(model, tournament_id),
            UiElement::Outline(x) => x.tournament_changed(model, tournament_id),
        }
    }
}

impl Ui {
    pub fn new() -> Ui {
        Ui { elements: indexmap!{}, next_id: 0 }
    }

    pub fn get_next_id(&mut self) -> UiElementId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get_elements(&self) -> &IndexMap<UiElementId, UiElement> {
        &self.elements
    }

    pub fn get_element(&self, id: UiElementId) -> Option<&UiElement> {
        self.elements.get(&id)
    }
    pub fn get_element_mut(&mut self, id: UiElementId) -> Option<&mut UiElement> {
        self.elements.get_mut(&id)
    }


    pub fn add_element(&mut self, element: UiElement) {
        self.elements.insert(element.get_id(), element);
    }

    pub fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        let ids: Vec<usize> = self.elements.keys().map(|k| *k).collect();

        for id in ids {
            self.get_element_mut(id).unwrap().tournament_changed(model, tournament_id);
        }
    }
}

/// Creates a wasm-bindgen Closure which can be called from Javascript, for use in event callbacks
/// e.g. onclick.
/// In the callback, we get access to the global state such as the Model, which we can't
/// easily store a reference to in the closure due to Rust's borrowing rules (as the closure is long-lived).
/// It's also responsible for doing post-change updates.
/// The goal here is to make it easy for UI components to register event callbacks in an ergonomic way,
/// without having to worry about borrowing of global data etc.
pub fn create_callback<F: FnMut(&mut Model, &mut Ui) -> () + 'static>(mut f: F) -> Closure<dyn FnMut()> {
    Closure::<dyn FnMut()>::new(move || {
        with_globals(|m, u| {
            f(m, u);
            m.process_updates(u);
        });
    })
}