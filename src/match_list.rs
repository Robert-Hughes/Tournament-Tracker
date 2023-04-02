use log::error;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlInputElement, HtmlElement, HtmlTableSectionElement, HtmlButtonElement, window};

use crate::{dom::{create_element, create_html_element}, tournament::{StageId, TournamentId, TeamId, Stage, Match, MatchId}, model::Model, ui::{create_callback, UiElementId, UiElement}};

pub struct MatchList {
    id: UiElementId,
    tournament_id: TournamentId,
    stage_id: StageId,

    dom_table: HtmlTableElement,
    head_row: HtmlTableRowElement,
    body: HtmlTableSectionElement,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl UiElement for MatchList {
    fn get_id(&self) -> UiElementId {
        self.id
    }

    fn as_match_list(&self) -> Option<&MatchList> { Some(self) }

    fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        if tournament_id == self.tournament_id {
            self.refresh(model);
        }
    }
}

impl MatchList {
    pub fn get_dom_table(&self) -> &HtmlTableElement {
        &self.dom_table
    }

    pub fn new(id: UiElementId, model: &Model, tournament_id: TournamentId, stage_id: StageId) -> MatchList {
        let dom_table = create_element::<HtmlTableElement>("table");

        let head: HtmlTableSectionElement = dom_table.create_t_head().dyn_into().expect("Cast failed");
        let head_row: HtmlTableRowElement = head.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");

        let body: HtmlTableSectionElement = dom_table.create_t_body().dyn_into().expect("Cast failed");

        let mut result = MatchList { id, tournament_id, stage_id, dom_table, head_row, body, closures: vec![] };

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        while self.body.rows().length() > 0 {
            self.body.delete_row(0).expect("Failed to delete row");
            //TODO: remove delete closures etc?
        }

        if let Some(stage) = model.get_stage(self.tournament_id, self.stage_id) {
            for (idx, (match_id, m)) in stage.matches.iter().enumerate() {
                self.add_match_elements(m, idx, stage);
            }
        }
    }

    fn add_match_elements(&mut self, m: &Match, idx: usize, stage: &Stage) {
        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let delete_button: HtmlButtonElement = create_element("button");
        delete_button.set_inner_text("X");
        cell.append_child(&delete_button).expect("Failed to append button");
        let id = self.id;
        let m_id = m.id;
        let click_closure = create_callback(move |model, ui| {
            if let Some(this) = ui.get_element(id).and_then(|u| u.as_match_list()) {
                this.on_delete_match_button_click(model, m_id);
            }
        });
        delete_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        self.closures.push(click_closure); // Needs to be kept alive

        //TODO: drag and drop
        if idx > 0 {
            let cell = new_row.insert_cell().expect("Failed to insert cell");
            let move_up_button: HtmlButtonElement = create_element("button");
            move_up_button.set_inner_text("^");
            cell.append_child(&move_up_button).expect("Failed to append button");
            let id = self.id;
            let m_id = m.id;
            let click_closure = create_callback(move |model, ui| {
                if let Some(this) = ui.get_element(id).and_then(|u| u.as_match_list()) {
                    this.on_reorder_match_button_click(model, m_id, idx - 1);
                }
            });
            move_up_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
            self.closures.push(click_closure); // Needs to be kept alive
        }

        if idx < stage.matches.len() - 1 {
            let cell = new_row.insert_cell().expect("Failed to insert cell");
            let move_down_button: HtmlButtonElement = create_element("button");
            move_down_button.set_inner_text("v");
            cell.append_child(&move_down_button).expect("Failed to append button");
            let id = self.id;
            let m_id = m.id;
            let click_closure = create_callback(move |model, ui| {
                if let Some(this) = ui.get_element(id).and_then(|u| u.as_match_list()) {
                    this.on_reorder_match_button_click(model, m_id, idx + 1);
                }
            });
            move_down_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
            self.closures.push(click_closure); // Needs to be kept alive
        }

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let team_a = stage.teams.get(&m.teams[0]).map(|t| &t.name[..]).unwrap_or("?");
        let team_a_score = if m.winner == m.teams[0] { 1 } else { 0 };
        let team_b = stage.teams.get(&m.teams[1]).map(|t| &t.name[..]).unwrap_or("?");
        let team_b_score = if m.winner == m.teams[1] { 1 } else { 0 };
        cell.set_inner_text(&format!("{team_a} {team_a_score} - {team_b_score} {team_b}"));
    }

    fn on_delete_match_button_click(&self, model: &mut Model, match_id: MatchId) {
        if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete this match?")) == Ok(true) {
            if let Err(_) = model.delete_match(self.tournament_id, self.stage_id, match_id) {
                error!("Failed to delete match");
            }
        }
    }

    fn on_reorder_match_button_click(&self, model: &mut Model, match_id: MatchId, new_idx: usize) {
        if let Err(_) = model.reorder_match(self.tournament_id, self.stage_id, match_id, new_idx) {
            error!("Failed to reorder match");
        }
    }
}