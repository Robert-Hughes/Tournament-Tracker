use log::{error};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlTableElement, HtmlTableRowElement, HtmlElement, HtmlTableSectionElement, HtmlButtonElement, window};

use crate::{dom::{create_element, create_html_element}, model::tournament::{StageId, TournamentId, TeamId, Stage, Team, StageKind}, model::Model, ui::{create_callback, UiElementId, UiElement, Event, EventList}};


//TODO: show total games played too
//TODO: show position (1st, 2nd etc), including ties
//TODO: show 'trend' e.g. last 5 game score, or full list of 'WLLWLLWL'. Maybe show which teams against too?

pub struct RoundRobinStandings {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_table: HtmlTableElement,
    head_row: HtmlTableRowElement,
    body: HtmlTableSectionElement,

    closures: Vec<Closure::<dyn FnMut()>>,
}

impl RoundRobinStandings {
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

    pub fn new(id: UiElementId, model: &Model, linked_outline_id: UiElementId) -> RoundRobinStandings {
        let dom_table = create_element::<HtmlTableElement>("table");

        let head: HtmlTableSectionElement = dom_table.create_t_head().dyn_into().expect("Cast failed");
        let head_row: HtmlTableRowElement = head.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("Team:");
        let cell = head_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("Score:");

        let body: HtmlTableSectionElement = dom_table.create_t_body().dyn_into().expect("Cast failed");

        let foot: HtmlTableSectionElement = dom_table.create_t_foot().dyn_into().expect("Cast failed");
        let foot_row: HtmlTableRowElement = foot.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");
        let cell = foot_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text("");
        let cell = foot_row.insert_cell().expect("Failed to insert cell");

        let add_team_button: HtmlElement = create_html_element("button");
        add_team_button.set_inner_text("Add team");
        cell.append_child(&add_team_button).expect("Failed to append child");

        let mut result = RoundRobinStandings { id, tournament_id: None, stage_id: None, linked_outline_id, dom_table, head_row, body, closures: vec![] };

        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::RoundRobinStandings(this)) = ui.get_element(id) {
                this.on_add_team_button_click(model);
            }
        });
        add_team_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));

        result.closures.push(click_closure); // Needs to be kept alive

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        let mut show = false;
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(s) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::RoundRobin { .. } = s.kind {
                    show = true;
                }
            }
        }
        self.dom_table.style().set_property("display",
            if show { "block" } else { "none" }).expect("Failed to set style");

        while self.body.rows().length() > 0 {
            self.body.delete_row(0).expect("Failed to delete row");
            //TODO: also delete delete button click closures?
        }
        while self.head_row.cells().length() > 1 {
            self.head_row.delete_cell(1).expect("Failed to delete cell");
        }

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                // Sort by win/loss score
                let mut sorted_teams : Vec<&Team> = stage.teams.values().collect();
                sorted_teams.sort_by_cached_key(|t| {
                    let w = stage.matches.values().filter(|m| m.get_winner() == Some(t.id)).count();
                    let l = stage.matches.values().filter(|m| m.get_loser() == Some(t.id)).count();
                    (w, l)
                });

                for team in sorted_teams.iter().rev() {
                    self.add_team_elements(team.id, &team.name, stage);
                }

            }
        }
    }

    fn add_team_elements(&mut self, team_id: TeamId, team_name: &str, stage: &Stage) {
        // Add row at the end
        let new_row: HtmlTableRowElement = self.body.insert_row().expect("Failed to insert row").dyn_into().expect("Cast failed");

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        cell.set_inner_text(&team_name);

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let w = stage.matches.values().filter(|m| m.get_winner() == Some(team_id)).count();
        let l = stage.matches.values().filter(|m| m.get_loser() == Some(team_id)).count();
        cell.set_inner_text(&format!("{w} - {l}"));

        let cell = new_row.insert_cell().expect("Failed to insert cell");
        let delete_button: HtmlButtonElement = create_element("button");
        delete_button.set_inner_text("X");
        cell.append_child(&delete_button).expect("Failed to append button");
        let id = self.id;
        let team_name2 = team_name.to_string();
        let click_closure = create_callback(move |model, ui| {
            if let Some(UiElement::RoundRobinStandings(this)) = ui.get_element(id) {
                this.on_delete_team_button_click(model, team_id, &team_name2);
            }
        });
        delete_button.set_onclick(Some(click_closure.as_ref().unchecked_ref()));
        self.closures.push(click_closure); // Needs to be kept alive
    }

    fn on_add_team_button_click(&self, model: &mut Model) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Ok(Some(name)) = window().unwrap().prompt_with_message("Enter name for new team:") {
                model.add_team(tournament_id, stage_id, name);
            }
        }
    }

    fn on_delete_team_button_click(&self, model: &mut Model, team_id: TeamId, team_name: &str) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete team '{team_name}'? All data for this team will be lost!!")) == Ok(true) {
                if let Err(_) = model.delete_team(tournament_id, stage_id, team_id) {
                    error!("Failed to delete team");
                }
            }
        }
    }
}
