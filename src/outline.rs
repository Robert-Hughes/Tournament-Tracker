use log::{error, debug};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlInputElement, HtmlElement, HtmlTableSectionElement, HtmlButtonElement, window, HtmlSelectElement, HtmlDivElement, HtmlOptGroupElement, HtmlOptionElement};

use crate::{dom::{create_element, create_html_element}, tournament::{StageId, TournamentId, TeamId, Stage}, model::Model, ui::{create_callback, UiElementId, UiElement}};

//TODO: reorder tournaments and stages
//TODO: rename tournaments and stages
//TODO: delete tournaments and stages

pub struct Outline {
    id: UiElementId,

    div: HtmlDivElement,
    select: HtmlSelectElement,
    new_tournament_name_input: HtmlInputElement,
    new_stage_name_input: HtmlInputElement,

    selected_tournament_id: Option<TournamentId>,
    selected_stage_id: Option<StageId>,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl UiElement for Outline {
    fn get_id(&self) -> UiElementId {
        self.id
    }

    fn as_outline(&self) -> Option<&Outline> { Some(self) }

    fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        self.refresh(model);
    }
}

impl Outline {
    pub fn get_div(&self) -> &HtmlDivElement {
        &self.div
    }

    pub fn new(id: UiElementId, model: &Model) -> Outline {
        let div = create_element::<HtmlDivElement>("div");

        let select = create_element::<HtmlSelectElement>("select");
        select.set_size(10);
        div.append_child(&select).expect("Failed to append child");

        let new_tournament_name_input: HtmlInputElement = create_element::<HtmlInputElement>("input");
        new_tournament_name_input.set_placeholder("New tournament name");
        div.append_child(&new_tournament_name_input).expect("Failed to append child");

        let add_tournament_button: HtmlElement = create_html_element("button");
        add_tournament_button.set_inner_text("Add tournament");
        div.append_child(&add_tournament_button).expect("Failed to append child");

        let new_stage_name_input: HtmlInputElement = create_element::<HtmlInputElement>("input");
        new_stage_name_input.set_placeholder("New stage name");
        div.append_child(&new_stage_name_input).expect("Failed to append child");

        let add_stage_button: HtmlElement = create_html_element("button");
        add_stage_button.set_inner_text("Add stage");
        div.append_child(&add_stage_button).expect("Failed to append child");

        let mut result = Outline { id, div, select, new_tournament_name_input, new_stage_name_input, selected_tournament_id: None, selected_stage_id: None, closures: vec![] };

        let click_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_outline()) {
                this.on_add_tournament_button_click(model);
            }
        });
        add_tournament_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let click_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_outline()) {
                this.on_add_stage_button_click(model);
            }
        });
        add_stage_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        result.closures.push(click_closure); // Needs to be kept alive

        let change_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_outline()) {
                this.on_select_change(model);
            }
        });
        result.select.set_onclick(Some(change_closure.as_ref().unchecked_ref()));
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
            self.select.add_with_html_option_element(&option).expect("Failed to append option");

            for (stage_id, stage) in &tournament.stages {
                let option = create_element::<HtmlOptionElement>("option");
                option.set_value(&stage_id.to_string());
                option.set_text(&format!("--{}", stage.name));
                self.select.add_with_html_option_element(&option).expect("Failed to append option");
            }
        }
    }

    fn on_add_tournament_button_click(&self, model: &mut Model) {
        model.add_tournament(self.new_tournament_name_input.value());
    }

    fn on_add_stage_button_click(&self, model: &mut Model) {
        if let Some(t) = self.selected_tournament_id {
            model.add_stage(t, self.new_stage_name_input.value());
        }
    }

    fn on_select_change(&mut self, model: &mut Model) {
        self.selected_tournament_id = x;
        self.selected_stage_id = y;
        debug!("{:?}", self.select.selected_options());
    }
}
