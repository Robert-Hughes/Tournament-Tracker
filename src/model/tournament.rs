use indexmap::IndexMap;
use indexmap::indexmap;
use serde::Deserialize;
use serde::Serialize;

use super::Model;

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
    #[serde(default = "default_stage_kind_for_deserialization")] // This field was added, so give it a default value so that we can deserialize old data
    pub kind: StageKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StageKind {
    RoundRobin {

    },
    Bracket {
        fixtures: IndexMap<FixtureId, Fixture>,
    }
}

pub type TournamentId = usize;
pub type StageId = usize;
pub type TeamId = usize;
pub type MatchId = usize;
pub type FixtureId = usize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
}

/// A match is something that we already have the results for. See also Fixture.
#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    pub id: MatchId,
    pub team_a: TeamId,
    pub team_b: TeamId,
    pub team_a_score: u32,
    pub team_b_score: u32,
}

/// A fixture is a match that might not yet have been played, used to describe an elimination bracket.
/// See also Match.
#[derive(Serialize, Deserialize, Debug)]
pub struct Fixture {
    pub id: FixtureId,
    /// The position of this fixture on the bracket view.
    pub layout: (i32, i32),
    /// If this fixture has already been played, this links to the match results.
    pub match_id: Option<MatchId>,
    /// If the team(s) playing in this fixture are determined by the results of a previous fixture,
    /// that is recorded here. E.g. in an elimination bracket the winner will advance to the next fixture.
    pub team_a: FixtureTeam,
    pub team_b: FixtureTeam,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FixtureTeam {
    /// The team playing in this fixture is fixed, i.e. pre-determined and not based on the result of another fixture.
    Fixed(TeamId),
    /// The team playing in this fixture is the winner of a previous fixture in the stage.
    Winner(FixtureId),
    /// The team playing in this fixture is the loser of a previous fixture in the stage.
    Loser(FixtureId),
}

impl Tournament {
    pub fn new(id: TournamentId, name: String) -> Tournament {
        Tournament { id, name, stages: indexmap![] }
    }
}

impl Stage {
    pub fn new_round_robin(id: StageId, tournament_id: TournamentId, name: String) -> Stage {
        Stage { id, tournament_id, name, teams: indexmap![], matches: indexmap![], kind: StageKind::RoundRobin {  } }
    }

    pub fn new_bracket(id: StageId, tournament_id: TournamentId, name: String) -> Stage {
        Stage { id, tournament_id, name, teams: indexmap![], matches: indexmap![], kind: StageKind::Bracket { fixtures: indexmap![] } }
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

impl FixtureTeam {
    pub fn to_pretty_desc(&self, stage: &Stage) -> String {
        match self {
            FixtureTeam::Fixed(t) => stage.teams.get(t).and_then(|t| Some(t.name.clone())).unwrap_or("???".to_string()),
            FixtureTeam::Winner(f) => format!("Winner of fixture {f}"),
            FixtureTeam::Loser(f) => format!("Loser of fixture {f}"),
        }
    }
}

// This field was added, so give it a default value so that we can deserialize old data
fn default_stage_kind_for_deserialization() -> StageKind {
    StageKind::RoundRobin {  }
}