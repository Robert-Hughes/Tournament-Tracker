use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlInputElement, HtmlElement, HtmlTableSectionElement};

use crate::{dom::{create_element, create_html_element}, tournament::{StageId, TournamentId}, model::Model, ui::{create_callback, UiElementId, UiElement}};

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

    fn tournament_changed(&self, model: &Model, tournament_id: TournamentId) {
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

    fn refresh(&self, model: &Model) {
        while self.body.rows().length() > 0 {
            self.body.delete_row(0).expect("Failed to delete row");
        }
        while self.head_row.cells().length() > 1 {
            self.head_row.delete_cell(1).expect("Failed to delete cell");
        }

        if let Some(stage) = model.get_stage(self.tournament_id, self.stage_id) {
            for (_team_id, team) in &stage.teams {
                self.add_team_elements(&team.name);
            }
        }
    }

    fn add_team_elements(&self, name: &str) {
        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&name);

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("0 - 0");
    }

    fn on_add_team_button_click(&self, model: &mut Model) {
        model.add_team(self.tournament_id, self.stage_id, self.new_team_name_input.value());
       // self.add_team_elements(&self.new_team_name_input.value());
    }
}
