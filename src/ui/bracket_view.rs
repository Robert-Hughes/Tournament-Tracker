use indexmap::IndexMap;
use log::{error, debug};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlTableSectionElement, HtmlElement, HtmlDivElement, MouseEvent, HtmlButtonElement};

use crate::{dom::{create_element}, model::tournament::{StageId, TournamentId, Team, Match, TeamId, StageKind, FixtureId}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

use super::create_callback_with_arg;

//TODO: drag fixture around
//TODO: delete fixture

pub struct BracketView {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_root: HtmlElement,

    closures: Vec<Box<dyn Drop>>,
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
        dom_root.set_class_name("bracket-view");
        dom_root.set_inner_text("Bracket view!");
        dom_root.style().set_property("position", "relative").expect("Failed to set property"); // For children to be absolutely positioned relative to this.

        let dblclick_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                this.on_background_dblclick(model, e);
            }
        }));
        dom_root.set_ondblclick(Some(dblclick_closure.as_ref().as_ref().unchecked_ref()));

        let mut closures: Vec<Box<dyn Drop>> = vec![];
        closures.push(dblclick_closure); // Needs to be kept alive

        let mut result = BracketView { id, tournament_id: None, stage_id: None, linked_outline_id, dom_root, closures };

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

        self.dom_root.set_inner_html("");
        //TODO: delete closures?

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { fixtures } = &stage.kind {
                    for (fid, f) in fixtures {
                        let new_div: HtmlDivElement = create_element::<HtmlDivElement>("div");
                        new_div.set_inner_text(&format!("Fixture {}", fid));
                        new_div.style().set_property("position", "absolute").expect("Failed to set property");
                        new_div.style().set_property("left", &f.layout.0.to_string()).expect("Failed to set property");
                        new_div.style().set_property("top", &f.layout.1.to_string()).expect("Failed to set property");

                        let delete_button: HtmlButtonElement = create_element("button");
                        delete_button.set_inner_text("X");
                        new_div.append_child(&delete_button).expect("Failed to append button");
                        let id = self.id;
                        let fid = *fid;
                        let click_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                                this.on_delete_fixture_button_click(model, fid);
                            }
                        }));
                        delete_button.set_onclick(Some(click_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(click_closure); // Needs to be kept alive


                        self.dom_root.append_child(&new_div).expect("Failed to append child");
                    }
                }
            }
        }
    }

    fn on_background_dblclick(&self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let None = model.add_fixture(tournament_id, stage_id, (e.offset_x(), e.offset_y())) {
                error!("Failed to add fixture");
            }
        }
    }

    fn on_delete_fixture_button_click(&self, model: &mut Model, fixture_id: FixtureId) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Err(_) = model.delete_fixture(tournament_id, stage_id, fixture_id) {
                error!("Failed to delete fixture");
            }
        }
    }
}
