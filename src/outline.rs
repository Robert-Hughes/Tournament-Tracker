use log::error;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlInputElement, HtmlElement, HtmlTableSectionElement, HtmlButtonElement, window, HtmlSelectElement, HtmlDivElement, HtmlOptGroupElement, HtmlOptionElement};

use crate::{dom::{create_element, create_html_element}, tournament::{StageId, TournamentId, TeamId, Stage}, model::Model, ui::{create_callback, UiElementId, UiElement}};

//TODO: reorder tournaments and stages
//TODO: rename tournaments and stages

pub struct Outline {
    id: UiElementId,

    div: HtmlDivElement,
    select: HtmlSelectElement,

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

        let mut result = Outline { id, div, select, closures: vec![] };

        // let click_closure = create_callback(move |model, ui| {
        //     if let Some(this) = ui.get_element(id).and_then(|u| u.as_round_robin_standings()) {
        //         this.on_add_team_button_click(model);
        //     }
        // });
        // add_team_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));

        // result.closures.push(click_closure); // Needs to be kept alive

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        while self.select.options().length() > 0 {
            self.select.options().remove(0).expect("Failed to delete option");
            //TODO: also delete delete button click closures?
        }

        for (tournament_id, tournament) in model.get_tournaments() {
            let opt_group = create_element::<HtmlOptGroupElement>("optgroup");
            opt_group.set_label(&tournament.name);

            for (stage_id, stage) in &tournament.stages {
                let option = create_element::<HtmlOptionElement>("option");
                option.set_value(&stage_id.to_string());
                option.set_text(&stage.name);
                opt_group.append_child(&option).expect("Failed to append child");
            }
            self.select.add_with_html_opt_group_element(&opt_group).expect("Failed to append optgroup");
        }
    }
}
