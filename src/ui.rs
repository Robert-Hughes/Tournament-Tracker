use indexmap::IndexMap;
use indexmap::indexmap;
use wasm_bindgen::prelude::Closure;

use crate::match_list::MatchList;
use crate::model::Model;
use crate::outline::Outline;
use crate::round_robin_standings::RoundRobinStandings;
use crate::tournament::StageId;
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

pub enum Event {
    SelectedTournamentAndStageChanged {
        source: UiElementId,
        new_tournament_id: Option<TournamentId>,
        new_stage_id: Option<StageId>,
    }
}

pub struct EventList {
    events: Vec<Event>,
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

    /// Events are deferred until an explicit "pass" where we process them, to avoid passing
    /// around too many mutable references.
    pub fn process_events(&mut self, model: &Model) {
        let mut all_events = EventList::new();
        for (_id, e) in &mut self.elements {
            all_events.combine(e.get_events());
        }
        for (_id, e) in &mut self.elements {
            e.process_events(&all_events, model);
        }
    }
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

    /// Events are deferred until an explicit "pass" where we process them, to avoid passing
    /// around too many mutable references.
    fn get_events(&mut self) -> EventList {
        match self {
            UiElement::Outline(x) => x.get_events(),
            _ => EventList::new()
        }
    }

    fn process_events(&mut self, events: &EventList, model: &Model) {
        match self {
            UiElement::RoundRobinTable(x) => x.process_events(events, model),
            UiElement::RoundRobinStandings(x) => x.process_events(events, model),
            UiElement::MatchList(x) => x.process_events(events, model),
            _ => ()
        }
    }
}

impl EventList {
    pub fn new() -> EventList {
        EventList { events: vec![] }
    }

    pub fn single(e: Event) -> EventList {
        EventList { events: vec![e] }
    }

    fn combine(&mut self, mut new_events: EventList) {
        // We could do something more fancy here like merge duplicate events
        self.events.append(&mut new_events.events);
    }

    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
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
            u.process_events(m);
        });
    })
}

