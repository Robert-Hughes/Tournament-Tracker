use indexmap::IndexMap;
use indexmap::indexmap;
use log::debug;
use log::error;
use serde::Deserialize;
use serde::Serialize;
use web_sys::window;

use crate::tournament::Match;
use crate::tournament::MatchId;
use crate::tournament::Team;
use crate::ui::Ui;
use crate::{tournament::{TournamentId, Tournament, StageId, Stage, TeamId}};

/// The data model, holding all the tournaments, stages, teams, etc.
/// The details of the held records are publicly exposed, but only as non-mutable references.
/// Any changes to the data are made through methods on the top level Model, allowing us to track
/// changes.
#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    tournaments: IndexMap<TournamentId, Tournament>,
    next_id: usize,
    #[serde(skip)]
    changed_tournaments: Vec<TournamentId>,
}

impl Model {
    const LOCAL_STORAGE_KEY: &str = "tournament-tracker-model";

    pub fn load() -> Model {
        debug!("Loading!");

        if let Ok(Some(s)) = window().expect("Missing window").local_storage().expect("Error getting localStorage").expect("Missing localStorage").get_item(Model::LOCAL_STORAGE_KEY) {
            if let Ok(m) = serde_json::from_str::<Model>(&s) {
                return m;
            }
        }

        if window().unwrap().confirm_with_message(&format!("Failed to load data! If this is expected then click OK and it will be reset. Otherwise check what's going on.")) == Ok(true) {
            return Model { tournaments: indexmap!{}, next_id: 0, changed_tournaments: vec![] }
        }

        panic!("No data!");
    }

    pub fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get_tournaments(&self) -> &IndexMap<TournamentId, Tournament> {
        &self.tournaments
    }

    pub fn get_tournament(&self, id: TournamentId) -> Option<&Tournament> {
        self.tournaments.get(&id)
    }

    pub fn get_stage(&self, tournament_id: TournamentId, stage_id: StageId) -> Option<&Stage> {
        self.tournaments.get(&tournament_id).and_then(|t| t.stages.get(&stage_id))
    }

    pub fn add_tournament(&mut self, name: String) -> TournamentId {
        let id = self.get_next_id();
        self.tournaments.insert(id, Tournament::new(id, name));
        self.changed_tournaments.push(id);
        id
    }

    pub fn add_stage(&mut self, tournament_id: TournamentId, name: String) -> Option<StageId> {
        let id = self.get_next_id();
        if let Some(t) = self.tournaments.get_mut(&tournament_id) {
            t.stages.insert(id, Stage::new(id, t.id, name));
            let tid = t.id;
            self.changed_tournaments.push(tid);
            Some(id)
        } else {
            None
        }
    }

    pub fn add_team(&mut self, tournament_id: TournamentId, stage_id: StageId, name: String) -> Option<TeamId> {
        let id = self.get_next_id();
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            s.teams.insert(id, Team::new(id, name));
            self.changed_tournaments.push(tournament_id);
            Some(id)
        } else {
            None
        }
    }

    pub fn delete_team(&mut self, tournament_id: TournamentId, stage_id: StageId, team_id: TeamId) -> Result<(), ()> {
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            // Remove any matches this team was in
            s.matches.retain(|_, m| !m.teams.contains(&team_id));

            if s.teams.shift_remove(&team_id).is_none() {
                return Err(());
            }
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_match(&mut self, tournament_id: TournamentId, stage_id: StageId, team_a: TeamId, team_b: TeamId, winner: TeamId) -> Option<MatchId> {
        let id = self.get_next_id();
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            if !s.teams.contains_key(&team_a) || !s.teams.contains_key(&team_b) {
                return None;
            }

            if let Some(m) = Match::new(id, team_a, team_b, winner) {
                s.matches.insert(id, m);
                self.changed_tournaments.push(tournament_id);
                return Some(id)
            }
        }
        return None;
    }

    pub fn delete_match(&mut self, tournament_id: TournamentId, stage_id: StageId, match_id: MatchId) -> Result<(), ()> {
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            if s.matches.shift_remove(&match_id).is_none() {
                return Err(());
            }
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn reorder_match(&mut self, tournament_id: TournamentId, stage_id: StageId, match_id: MatchId, new_idx: usize) -> Result<(), ()> {
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            if let Some(old_idx) = s.matches.get_index_of(&match_id) {
                let new_idx = std::cmp::min(new_idx, s.matches.len() - 1);
                s.matches.move_index(old_idx, new_idx);
            }
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    /// We can't easily notify subscribers about changes to the model during the change itself,
    /// as that would require passing round lots of mutable references which Rust doesn't like.
    /// Instead we batch them up and handle them all "at the end".
    pub fn process_updates(&mut self, ui: &mut Ui) {
        for t in &self.changed_tournaments {
           ui.tournament_changed(self, *t);
        }
        if !self.changed_tournaments.is_empty() {
            self.save();
        }
        self.changed_tournaments.clear();
    }

    pub fn save(&self) {
        debug!("Saving!");
        match serde_json::to_string(&self) {
            Ok(s) => window().expect("Missing window").local_storage().expect("Error getting localStorage").expect("Missing localStorage").set_item(Model::LOCAL_STORAGE_KEY, &s).expect("Failed to save"),
            Err(e) => error!("Error saving: {e}"),
        };
    }
}
