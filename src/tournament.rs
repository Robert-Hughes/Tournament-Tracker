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
    pub team_a: TeamId,
    pub team_b: TeamId,
    pub team_a_score: u32,
    pub team_b_score: u32,
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
    pub fn is_between(&self, a: TeamId, b: TeamId) -> bool {
        let t = [self.team_a, self.team_b];
        t == [a, b] ||t == [b, a]
    }

    pub fn contains(&self, t: TeamId) -> bool {
        self.team_a == t || self.team_b == t
    }

    pub fn get_winner(&self) -> Option<TeamId> {
        match self.team_a_score.cmp(&self.team_b_score) {
            std::cmp::Ordering::Less => Some(self.team_b),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(self.team_a),
        }
    }

    pub fn get_loser(&self) -> Option<TeamId> {
        match self.team_a_score.cmp(&self.team_b_score) {
            std::cmp::Ordering::Less => Some(self.team_a),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(self.team_b),
        }
    }
}
