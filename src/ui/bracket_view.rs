use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlTableSectionElement, HtmlElement};

use crate::{dom::{create_element}, model::tournament::{StageId, TournamentId, Team, Match, TeamId, StageKind}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

pub struct BracketView {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_root: HtmlElement,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl BracketView {
    pub fn get_id(&self) -> UiElementId {
        self.id
    }

    pub fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        if Some(tournament_id) == self.tournament_id {
            self.refresh(model);
        }
    }

    pub fn process_events(&mut self, events: &EventList, model: &Model) {
        for e in events.get_events() {
            match e {
                Event::SelectedTournamentAndStageChanged { source, new_tournament_id, new_stage_id } if *source == self.linked_outline_id => {
                    self.tournament_id = *new_tournament_id;
                    self.stage_id = *new_stage_id;
                    self.refresh(model);
                }
                _ => (),
            }
        }
    }

    pub fn get_dom_root(&self) -> &HtmlElement {
        &self.dom_root
    }

    pub fn new(id: UiElementId, model: &Model, linked_outline_id: UiElementId) -> BracketView {
        let dom_root = create_element::<HtmlElement>("div");
        dom_root.set_inner_text("Bracket view!");

        let mut result = BracketView { id, tournament_id: None, stage_id: None, linked_outline_id, dom_root, closures: vec![] };

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        let mut show = false;
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(s) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { .. } = s.kind {
                    show = true;
                }
            }
        }
        self.dom_root.style().set_property("display",
            if show { "block" } else { "none" }).expect("Failed to set style");

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
            }
        }
    }
}
