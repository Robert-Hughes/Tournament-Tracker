use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlTableSectionElement};

use crate::{dom::{create_element}, tournament::{StageId, TournamentId, Team, Match, TeamId}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

//TODO: highlight column and row on mouse over? Or altnerate shading to make rows/cols easier to follow
//TODO: sort by score?
//TODO: support multiple games between the same teams (e.g. double round robin)
//TODO: what about tie breaker games after the normal round robins?
//TODO: make cells uniform width

pub struct RoundRobinTable {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_table: HtmlTableElement,
    head_row: HtmlTableRowElement,
    body: HtmlTableSectionElement,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl RoundRobinTable {
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

    pub fn get_dom_table(&self) -> &HtmlTableElement {
        &self.dom_table
    }

    pub fn new(id: UiElementId, model: &Model, linked_outline_id: UiElementId) -> RoundRobinTable {
        let dom_table = create_element::<HtmlTableElement>("table");

        let head: HtmlTableSectionElement = dom_table.create_t_head().dyn_into().expect("Cast failed");
        let head_row: HtmlTableRowElement = head.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("");

        let body: HtmlTableSectionElement = dom_table.create_t_body().dyn_into().expect("Cast failed");

        let mut result = RoundRobinTable { id, tournament_id: None, stage_id: None, linked_outline_id, dom_table, head_row, body, closures: vec![] };

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        while self.body.rows().length() > 0 {
            self.body.delete_row(0).expect("Failed to delete row");
            //TODO: remove closures for result cells?
        }
        while self.head_row.cells().length() > 1 {
            self.head_row.delete_cell(1).expect("Failed to delete cell");
        }

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                // Column headings
                for (_team_id, team) in &stage.teams {
                    let cell = self.head_row.insert_cell().expect("Failed to insert cell");
                    cell.set_inner_text(&team.name);
                }

                // One row per team
                for (_team_id, team) in &stage.teams {
                    self.add_row(team, &stage.teams, &stage.matches);
                }
            }
        }
    }

    fn add_row(&mut self, team: &Team, teams: &IndexMap<usize, Team>, matches: &IndexMap<usize, Match>) {
        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&team.name);

        for (&other_team_id, _other_team) in teams {
            let cell = new_row.insert_cell().expect("Failed to insert cell");

            // Check if these teams have played
            if let Some((_, m)) = matches.iter().find(|(_, m)| m.is_between(team.id, other_team_id)) {
                cell.set_inner_text(if m.get_winner() == Some(team.id) { "W" } else { "L" });
            } else if team.id == other_team_id {
                cell.set_inner_text("+"); // make the diagonal distinguished from other matches not yet played (as these can never be played!)
            } else {
                cell.set_inner_text("-");
            }

            let id = self.id;
            let team_id = team.id;
            let other_team_id = other_team_id;
            let click_closure = create_callback(move |model, ui| {
                if let Some(UiElement::RoundRobinTable(this)) = ui.get_element(id) {
                    this.on_result_click(model, team_id, other_team_id);
                }
            });
            cell.set_onclick(Some(click_closure.as_ref().unchecked_ref()));

            self.closures.push(click_closure); // Needs to be kept alive
        }
    }

    fn on_result_click(&self, model: &mut Model, team_id: TeamId, other_team_id: TeamId) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                //TODO: check for confirmation before changing/removing match results?

                // Check if these teams have played
                if let Some((match_id, m)) = stage.matches.iter().find(|(_, m)| m.is_between(team_id, other_team_id)) {
                    if m.get_winner() == Some(team_id) {
                        if let Err(_) = model.delete_match(tournament_id, stage_id, *match_id) {
                            error!("Failed to delete match");
                        }
                        model.add_match(tournament_id, stage_id, team_id, other_team_id, 0, 1);
                    } else {
                        if let Err(_) = model.delete_match(tournament_id, stage_id, *match_id) {
                            error!("Failed to delete match");
                        }
                    }
                } else {
                    model.add_match(tournament_id, stage_id, team_id, other_team_id, 1, 0);
                }
            }
        }
    }
}
