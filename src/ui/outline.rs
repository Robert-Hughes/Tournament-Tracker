use log::{error, debug};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlElement, HtmlSelectElement, HtmlDivElement, HtmlOptionElement, window};

use crate::{dom::{create_element, create_html_element}, model::tournament::{StageId, TournamentId}, model::Model, ui::{create_callback, UiElementId, UiElement, EventList, Event}};

//TODO: reorder tournaments and stages

pub struct Outline {
    id: UiElementId,

    div: HtmlDivElement,
    select: HtmlSelectElement,

    selected_tournament_id: Option<TournamentId>,
    selected_stage_id: Option<StageId>,
    selection_change_event_pending: bool,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl Outline {
    pub fn get_id(&self) -> UiElementId {
        self.id
    }

    pub fn tournament_changed(&mut self, model: &Model, _tournament_id: TournamentId) {
        self.refresh(model);
    }

    pub fn get_div(&self) -> &HtmlDivElement {
        &self.div
    }

    pub fn new(id: UiElementId, model: &Model) -> Outline {
        let div = create_element::<HtmlDivElement>("div");

        let select = create_element::<HtmlSelectElement>("select");
        select.set_size(10);
        div.append_child(&select).expect("Failed to append child");

        let add_tournament_button: HtmlElement = create_html_element("button");
        add_tournament_button.set_inner_text("Add tournament");
        div.append_child(&add_tournament_button).expect("Failed to append child");

        let add_stage_round_robin_button: HtmlElement = create_html_element("button");
        add_stage_round_robin_button.set_inner_text("Add stage (round robin)");
        div.append_child(&add_stage_round_robin_button).expect("Failed to append child");

        let add_stage_bracket_button: HtmlElement = create_html_element("button");
        add_stage_bracket_button.set_inner_text("Add stage (bracket)");
        div.append_child(&add_stage_bracket_button).expect("Failed to append child");

        let delete_button: HtmlElement = create_html_element("button");
        delete_button.set_inner_text("Delete");
        div.append_child(&delete_button).expect("Failed to append child");

        let rename_button: HtmlElement = create_html_element("button");
        rename_button.set_inner_text("Rename");
        div.append_child(&rename_button).expect("Failed to append child");

        let mut result = Outline { id, div, select,
            selected_tournament_id: None, selected_stage_id: None, selection_change_event_pending: false, closures: vec![] };

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element(id) {
                this.on_add_tournament_button_click(model);
            }
        });
        add_tournament_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element(id) {
                this.on_add_stage_round_robin_button_click(model);
            }
        });
        add_stage_round_robin_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element(id) {
                this.on_add_stage_bracket_button_click(model);
            }
        });
        add_stage_bracket_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element(id) {
                this.on_delete_button_click(model);
            }
        });
        delete_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element(id) {
                this.on_rename_button_click(model);
            }
        });
        rename_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let change_closure = create_callback(move |_model, ui| {
            if let Some(UiElement::Outline(this)) = ui.get_element_mut(id) {
                this.on_select_change();
            }
        });
        result.select.set_onclick(Some(change_closure.as_ref().unchecked_ref()));
        result.select.set_onkeydown(Some(change_closure.as_ref().unchecked_ref())); // onclick doesn't work for keyboard events!

        result.closures.push(change_closure); // Needs to be kept alive


        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        while self.select.child_element_count() > 0 {
            self.select.first_element_child().expect("Child element missing").remove();
            //TODO: also delete delete button click closures?
        }

        for (tournament_id, tournament) in model.get_tournaments() {
            // Note we don't use optgroups for tournaments, so that they are still selectable
            let option = create_element::<HtmlOptionElement>("option");
            option.set_value(&tournament_id.to_string());
            option.set_text(&tournament.name);
            option.dataset().set("tournament_id", &tournament_id.to_string()).expect("Failed to set dataset");
            self.select.add_with_html_option_element(&option).expect("Failed to append option");

            for (stage_id, stage) in &tournament.stages {
                let option = create_element::<HtmlOptionElement>("option");
                option.set_value(&stage_id.to_string());
                option.set_text(&format!("--{}", stage.name));
                option.dataset().set("tournament_id", &tournament_id.to_string()).expect("Failed to set dataset");
                option.dataset().set("stage_id", &stage_id.to_string()).expect("Failed to set dataset");
                self.select.add_with_html_option_element(&option).expect("Failed to append option");
            }
        }
    }

    fn on_add_tournament_button_click(&self, model: &mut Model) {
        if let Ok(Some(name)) = window().unwrap().prompt_with_message("Enter name for new tournament:") {
            model.add_tournament(name);
        }
    }

    fn on_add_stage_round_robin_button_click(&self, model: &mut Model) {
        if let Some(t) = self.selected_tournament_id {
            if let Ok(Some(name)) = window().unwrap().prompt_with_message("Enter name for new stage:") {
                model.add_stage_round_robin(t, name);
            }
        }
    }

    fn on_add_stage_bracket_button_click(&self, model: &mut Model) {
        if let Some(t) = self.selected_tournament_id {
            if let Ok(Some(name)) = window().unwrap().prompt_with_message("Enter name for new stage:") {
                model.add_stage_bracket(t, name);
            }
        }
    }

    //TODO: this should fire when deleting something from the list, resulting in a refresh. However that would then break things
    // whenever we modify a tournament that is currently selected - need to retain selection through a refresh!
    fn on_select_change(&mut self) {
        match self.select.selected_options() {
            x if x.length() == 0 => {
                self.selected_tournament_id = None;
                self.selected_stage_id = None;
            },
            x if x.length() == 1 => {
                let o = x.item(0).expect("Just checked the length");
                let o: HtmlElement = o.dyn_into().expect("Should be an HTML option");
                self.selected_tournament_id = o.dataset().get("tournament_id").and_then(|t| t.parse().ok());
                self.selected_stage_id = o.dataset().get("stage_id").and_then(|t| t.parse().ok());
            }
            _ => {
                error!("Multi-select should not be possible!");
            }
        }
        debug!("{:?} {:?}", self.selected_tournament_id, self.selected_stage_id);
        self.selection_change_event_pending = true;
    }

    fn on_delete_button_click(&self, model: &mut Model) {
        match (self.selected_tournament_id, self.selected_stage_id) {
            (Some(t), Some(s)) => {
                let stage_name = model.get_stage(t, s).map(|s| s.name.clone()).unwrap_or("".to_string());
                if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete stage '{stage_name}'? All data for this stage will be lost!!")) == Ok(true) {
                    if let Err(_) = model.delete_stage(t, s) {
                        error!("Failed to delete stage");
                    }
                }
            }
            (Some(t), None) => {
                let tournament_name = model.get_tournament(t).map(|t| t.name.clone()).unwrap_or("".to_string());
                if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete tournament '{tournament_name}'? All data for this tournament will be lost!!")) == Ok(true) {
                    if let Err(_) = model.delete_tournament(t) {
                        error!("Failed to delete tournament");
                    }
                }
            }
            _ => (),
        }
    }

    fn on_rename_button_click(&self, model: &mut Model) {
        match (self.selected_tournament_id, self.selected_stage_id) {
            (Some(t), Some(s)) => {
                let stage_name = model.get_stage(t, s).map(|s| s.name.clone()).unwrap_or("".to_string());
                if let Ok(Some(new_name)) = window().unwrap().prompt_with_message_and_default(&format!("Enter new name for stage '{stage_name}':"), &stage_name) {
                    if let Err(_) = model.rename_stage(t, s, &new_name) {
                        error!("Failed to rename tournament");
                    }
                }
            }
            (Some(t), None) => {
                let tournament_name = model.get_tournament(t).map(|t| t.name.clone()).unwrap_or("".to_string());
                if let Ok(Some(new_name)) = window().unwrap().prompt_with_message_and_default(&format!("Enter new name for tournament '{tournament_name}':"), &tournament_name) {
                    if let Err(_) = model.rename_tournament(t, &new_name) {
                        error!("Failed to rename tournament");
                    }
                }
            }
            _ => (),
        }
    }

    pub fn get_events(&mut self) -> EventList {
        if self.selection_change_event_pending {
            EventList::single(Event::SelectedTournamentAndStageChanged { source: self.id, new_tournament_id: self.selected_tournament_id, new_stage_id: self.selected_stage_id })
        } else {
            EventList::new()
        }
    }
}
