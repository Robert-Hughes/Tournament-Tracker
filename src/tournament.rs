use indexmap::IndexMap;
use indexmap::indexmap;
use serde::Deserialize;
use serde::Serialize;

// These types are all public with public fields, i.e. no encapsulation.
// The Model handles all that. Nobody outside the Model can get a mutable reference
// to the data here so can't modify it incorrectly.

#[derive(Serialize, Deserialize, Debug)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub stages: IndexMap<StageId, Stage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stage  {
    pub id: StageId,
    pub tournament_id: TournamentId,

    pub name: String,
    pub teams: IndexMap<TeamId, Team>,
    pub matches: Vec<Match>,
}

pub type TournamentId = usize;
pub type StageId = usize;
pub type TeamId = usize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    pub teams: [TeamId; 2],
}

impl Tournament {
    pub fn new(id: TournamentId, name: String) -> Tournament {
        Tournament { id, name, stages: indexmap![] }
    }
}

impl Stage {
    pub fn new(id: StageId, tournament_id: TournamentId, name: String) -> Stage {
        Stage { id, tournament_id, name, teams: indexmap![], matches: vec![] }
    }
}

impl Team {
    pub fn new(id: TeamId, name: String) -> Team {
        Team { id, name }
    }
}