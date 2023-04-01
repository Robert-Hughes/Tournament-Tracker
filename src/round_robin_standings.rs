use log::error;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlInputElement, HtmlElement, HtmlTableSectionElement, HtmlButtonElement, window};

use crate::{dom::{create_element, create_html_element}, tournament::{StageId, TournamentId, TeamId, Stage}, model::Model, ui::{create_callback, UiElementId, UiElement}};

pub struct RoundRobinStandings {
    id: UiElementId,
    tournament_id: TournamentId,
    stage_id: StageId,

    dom_table: HtmlTableElement,
    head_row: HtmlTableRowElement,
    body: HtmlTableSectionElement,
    new_team_name_input: HtmlInputElement,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl UiElement for RoundRobinStandings {
    fn get_id(&self) -> UiElementId {
        self.id
    }

    fn as_round_robin_standings(&self) -> Option<&RoundRobinStandings> { Some(self) }

    fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        if tournament_id == self.tournament_id {
            self.refresh(model);
        }
    }
}

impl RoundRobinStandings {
    pub fn get_dom_table(&self) -> &HtmlTableElement {
        &self.dom_table
    }

    pub fn new(id: UiElementId, model: &Model, tournament_id: TournamentId, stage_id: StageId) -> RoundRobinStandings {
        let dom_table = create_element::<HtmlTableElement>("table");

        let head: HtmlTableSectionElement = dom_table.create_t_head().dyn_into().expect("Cast failed");
        let head_row: HtmlTableRowElement = head.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("Team:");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(":");

        let body: HtmlTableSectionElement = dom_table.create_t_body().dyn_into().expect("Cast failed");

        let foot: HtmlTableSectionElement = dom_table.create_t_foot().dyn_into().expect("Cast failed");
        let foot_row: HtmlTableRowElement = foot.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = foot_row.insert_cell().expect("Failed to insert cell");

        let new_team_name_input: HtmlInputElement = create_element::<HtmlInputElement>("input");
        new_team_name_input.set_placeholder("New team name");
        cell.append_child(&new_team_name_input).expect("Failed to append child");

        let add_team_button: HtmlElement = create_html_element("button");
        add_team_button.set_inner_text("Add team");
        cell.append_child(&add_team_button).expect("Failed to append child");

        let mut result = RoundRobinStandings { id, tournament_id, stage_id, dom_table, head_row, body, new_team_name_input, closures: vec![] };

        let click_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_round_robin_standings()) {
                this.on_add_team_button_click(model);
            }
        });
        add_team_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));

        result.closures.push(click_closure); // Needs to be kept alive

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        while self.body.rows().length() > 0 {
            self.body.delete_row(0).expect("Failed to delete row");
            //TODO: also delete delete button click closures?
        }
        while self.head_row.cells().length() > 1 {
            self.head_row.delete_cell(1).expect("Failed to delete cell");
        }

        if let Some(stage) = model.get_stage(self.tournament_id, self.stage_id) {
            for (team_id, team) in &stage.teams {
                self.add_team_elements(*team_id, &team.name, stage);
            }
        }
    }

    fn add_team_elements(&mut self, team_id: TeamId, team_name: &str, stage: &Stage) {
        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let delete_button: HtmlButtonElement = create_element("button");
        delete_button.set_inner_text("X");
        cell.append_child(&delete_button).expect("Failed to append button");
        let id = self.id;
        let team_name2 = team_name.to_string();
        let click_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_round_robin_standings()) {
                this.on_delete_team_button_click(model, team_id, &team_name2);
            }
        });
        delete_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        self.closures.push(click_closure); // Needs to be kept alive


        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&team_name);

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let w = stage.matches.values().filter(|m| m.winner == team_id).count();
        let l = stage.matches.values().filter(|m| m.loser == team_id).count();
        cell.set_inner_text(&format!("{w} - {l}"));
    }

    fn on_add_team_button_click(&self, model: &mut Model) {
        model.add_team(self.tournament_id, self.stage_id, self.new_team_name_input.value());
       // self.add_team_elements(&self.new_team_name_input.value());
    }

    fn on_delete_team_button_click(&self, model: &mut Model, team_id: TeamId, team_name: &str) {
        if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete team '{team_name}'? All data for this team will be lost!!")) == Ok(true) {
            if let Err(_) = model.delete_team(self.tournament_id, self.stage_id, team_id) {
                error!("Failed to delete team");
            }
        }
    }

}
