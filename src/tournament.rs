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
    //TODO: any way to re-order matches from GUI?
    pub matches: IndexMap<MatchId, Match>,
}

pub type TournamentId = usize;
pub type StageId = usize;
pub type TeamId = usize;
pub type MatchId = usize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    pub id: MatchId,
    pub teams: [TeamId; 2],
    pub winner: TeamId,
    pub loser: TeamId,
}

impl Tournament {
    pub fn new(id: TournamentId, name: String) -> Tournament {
        Tournament { id, name, stages: indexmap![] }
    }
}

impl Stage {
    pub fn new(id: StageId, tournament_id: TournamentId, name: String) -> Stage {
        Stage { id, tournament_id, name, teams: indexmap![], matches: indexmap![] }
    }
}

impl Team {
    pub fn new(id: TeamId, name: String) -> Team {
        Team { id, name }
    }
}

impl Match {
    pub fn new(id: MatchId, team_a: TeamId, team_b: TeamId, winner: TeamId) -> Option<Match> {
        if winner != team_a && winner != team_b {
            return None;
        }
        if team_a == team_b {
            return None;
        }
        let loser = if winner == team_a { team_b } else { team_a };
        Some(Match { id, teams: [team_a, team_b], winner, loser })
    }

    pub fn is_between(&self, a: TeamId, b: TeamId) -> bool {
        self.teams == [a, b] || self.teams == [b, a]
    }
}