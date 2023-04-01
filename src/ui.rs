use std::any::Any;

use indexmap::IndexMap;
use indexmap::indexmap;
use wasm_bindgen::prelude::Closure;

use crate::model::Model;
use crate::round_robin_standings::RoundRobinStandings;
use crate::with_globals;
use crate::{tournament::{TournamentId}, round_robin_table::RoundRobinTable};

/// Contains all the UI elements.
pub struct Ui {
    elements: IndexMap<UiElementId, Box<dyn UiElement>>,
    next_id: UiElementId,
}

pub type UiElementId = usize;

pub trait UiElement : Any {
    fn get_id(&self) -> UiElementId;

    fn as_round_robin_table(&self) -> Option<&RoundRobinTable> { None }
    fn as_round_robin_standings(&self) -> Option<&RoundRobinStandings> { None }

    fn tournament_changed(&self, model: &Model, tournament_id: TournamentId);
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

    pub fn get_elements(&self) -> &IndexMap<UiElementId, Box<dyn UiElement>> {
        &self.elements
    }

    pub fn get_element(&self, id: UiElementId) -> Option<&Box<dyn UiElement>> {
        self.elements.get(&id)
    }

    pub fn add_element(&mut self, element: Box<dyn UiElement>) {
        self.elements.insert(element.get_id(), element);
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