use wasm_bindgen::{JsCast};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlTableSectionElement};

use crate::{dom::{create_element}, tournament::{StageId, TournamentId}, model::Model, ui::{UiElement, UiElementId}};

pub struct RoundRobinTable {
    id: UiElementId,
    tournament_id: TournamentId,
    stage_id: StageId,

    dom_table: HtmlTableElement,
    head_row: HtmlTableRowElement,
    body: HtmlTableSectionElement,
}

impl UiElement for RoundRobinTable {
    fn get_id(&self) -> UiElementId {
        self.id
    }

    fn as_round_robin_table(&self) -> Option<&RoundRobinTable> { Some(self) }

    fn tournament_changed(&self, model: &Model, tournament_id: TournamentId) {
        if tournament_id == self.tournament_id {
            self.refresh(model);
        }
    }
}

impl RoundRobinTable {
    pub fn get_dom_table(&self) -> &HtmlTableElement {
        &self.dom_table
    }

    pub fn new(id: UiElementId, model: &Model, tournament_id: TournamentId, stage_id: StageId) -> RoundRobinTable {
        let dom_table = create_element::<HtmlTableElement>("table");

        let head: HtmlTableSectionElement = dom_table.create_t_head().dyn_into().expect("Cast failed");
        let head_row: HtmlTableRowElement = head.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("Team:");

        let body: HtmlTableSectionElement = dom_table.create_t_body().dyn_into().expect("Cast failed");

        let result = RoundRobinTable { id, tournament_id, stage_id, dom_table, head_row, body };

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
        // New column heading for the new team
        let cell = self.head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&name);

        // Add a new cell to the rows for existing teams
        let num_rows = self.body.rows().length();
        for row_idx in 0..num_rows {
            let row: HtmlTableRowElement = self.body.rows().get_with_index(row_idx).expect("Indexing failed").dyn_into().expect("Cast failed");
            let cell = row.insert_cell().expect("Failed to insert cell");
            cell.set_inner_text("?");
        }

        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&name);

        for _col_idx in 0..num_rows+1 {
            let cell = new_row.insert_cell().expect("Failed to insert cell");
            cell.set_inner_text("?");
        }
    }
}
