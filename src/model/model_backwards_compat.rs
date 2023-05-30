// Definitions for old versions of the model, used to load old serialized data and upgrade it to new versions.

// V1 (which was unversioned) had a different representation of Matches
pub(crate) mod v1 {
    use indexmap::IndexMap;
    use indexmap::indexmap;
    use log::debug;
    use serde::{Serialize, Deserialize};
    use web_sys::Storage;

    use crate::model::tournament::StageKind;

    #[derive(Serialize, Deserialize, Debug)]
    struct Model {
        tournaments: IndexMap<TournamentId, Tournament>,
        next_id: usize,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Tournament {
        id: TournamentId,
        name: String,
        stages: IndexMap<StageId, Stage>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Stage  {
        id: StageId,
        tournament_id: TournamentId,

        name: String,
        teams: IndexMap<TeamId, Team>,
        matches: IndexMap<MatchId, Match>,
    }

    type TournamentId = usize;
    type StageId = usize;
    type TeamId = usize;
    type MatchId = usize;

    #[derive(Serialize, Deserialize, Debug)]
    struct Team {
        id: TeamId,
        name: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Match {
        id: MatchId,
        teams: [TeamId; 2],
        winner: TeamId,
        loser: TeamId,
    }

    pub fn load_and_upgrade(storage: Storage) -> Result<crate::model::Model, String> {
        debug!("Loading and upgrading model from v1");

        let old_model = match storage.get_item("tournament-tracker-model") {
            Ok(Some(s)) => {
                match serde_json::from_str::<Model>(&s) {
                    Ok(m) => Ok(m),
                    Err(e) => Err(format!("Failed to deserialize data: {:?}", e)),
                }
            }
            e => Err(format!("Failed to load from local storage: {:?}", e)),
        }?;

        let mut new_model = crate::model::Model::new();
        new_model.next_id = old_model.next_id;
        for (tournament_id, old_tournament) in old_model.tournaments {
            let mut new_tournament = crate::model::Tournament { id: old_tournament.id, name: old_tournament.name, stages: indexmap!{} };

            for (stage_id, old_stage) in old_tournament.stages {
                let mut new_stage = crate::model::Stage { id: old_stage.id, name: old_stage.name, tournament_id, teams: indexmap!{}, matches: indexmap!{}, kind: StageKind::RoundRobin {  } };

                for (team_id, old_team) in old_stage.teams {
                    let new_team = crate::model::Team { id: old_team.id, name: old_team.name };

                    new_stage.teams.insert(team_id, new_team);
                }

                for (match_id, old_match) in old_stage.matches {
                    // This is where the actual conversion happens from v1 -> v2
                    let new_match = crate::model::Match { id: old_match.id, team_a: old_match.teams[0],
                        team_b: old_match.teams[1], team_a_score: if old_match.winner == old_match.teams[0] { 1 } else { 0 },
                        team_b_score: if old_match.winner == old_match.teams[1] { 1 } else { 0 },
                    };

                    new_stage.matches.insert(match_id, new_match);
                }

                new_tournament.stages.insert(stage_id, new_stage);
            }

            new_model.tournaments.insert(tournament_id, new_tournament);
        }

        Ok(new_model)
    }
}

// V2 had fixture teams as separate enum variants
pub(crate) mod v2 {
    use indexmap::IndexMap;
    use indexmap::indexmap;
    use log::debug;
    use serde::{Serialize, Deserialize};
    use web_sys::Storage;

    #[derive(Serialize, Deserialize, Debug)]
    struct Model {
        tournaments: IndexMap<TournamentId, Tournament>,
        next_id: usize,
    }

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

    // This field was added, so give it a default value so that we can deserialize old data
    fn default_stage_kind_for_deserialization() -> StageKind {
        StageKind::RoundRobin {  }
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
        /// The team playing in this fixture is the winner/loser of a previous fixture in the stage.
        Winner(FixtureId),
        Loser(FixtureId)
    }

    pub fn load_and_upgrade(storage: Storage) -> Result<crate::model::Model, String> {
        debug!("Loading and upgrading model from v2");

        let old_model = match storage.get_item("tournament-tracker-model") {
            Ok(Some(s)) => {
                match serde_json::from_str::<Model>(&s) {
                    Ok(m) => Ok(m),
                    Err(e) => Err(format!("Failed to deserialize data: {:?}", e)),
                }
            }
            e => Err(format!("Failed to load from local storage: {:?}", e)),
        }?;

        let mut new_model = crate::model::Model::new();
        new_model.next_id = old_model.next_id;
        for (tournament_id, old_tournament) in old_model.tournaments {
            let mut new_tournament = crate::model::Tournament { id: old_tournament.id, name: old_tournament.name, stages: indexmap!{} };

            for (stage_id, old_stage) in old_tournament.stages {
                let new_kind = match old_stage.kind {
                    StageKind::RoundRobin {  } => crate::model::StageKind::RoundRobin {  },
                    StageKind::Bracket { fixtures: old_fixtures } => {
                        let mut new_fixtures = indexmap!{};
                        for (fixture_id, old_fixture) in old_fixtures {
                            let upgrade_fixture_team = |ft: FixtureTeam| {
                                match ft {
                                    FixtureTeam::Fixed(t) => crate::model::FixtureTeam::Fixed(t),
                                    FixtureTeam::Winner(f) => crate::model::FixtureTeam::Linked { fixture_id: f, outcome: crate::model::tournament::Outcome::Winner },
                                    FixtureTeam::Loser(f) => crate::model::FixtureTeam::Linked { fixture_id: f, outcome: crate::model::tournament::Outcome::Loser },
                                }
                            };

                            let new_fixture = crate::model::Fixture { id: old_fixture.id, layout: old_fixture.layout, match_id: old_fixture.match_id,
                                team_a: upgrade_fixture_team(old_fixture.team_a), team_b: upgrade_fixture_team(old_fixture.team_b) };

                            new_fixtures.insert(fixture_id, new_fixture);
                        }

                        crate::model::StageKind::Bracket { fixtures: new_fixtures }
                    }
                };

                let mut new_stage = crate::model::Stage { id: old_stage.id, name: old_stage.name, tournament_id, teams: indexmap!{}, matches: indexmap!{}, kind: new_kind };

                for (team_id, old_team) in old_stage.teams {
                    let new_team = crate::model::Team { id: old_team.id, name: old_team.name };

                    new_stage.teams.insert(team_id, new_team);
                }

                for (match_id, old_match) in old_stage.matches {
                    let new_match = crate::model::Match { id: old_match.id, team_a: old_match.team_a,
                        team_b: old_match.team_b, team_a_score: old_match.team_a_score, team_b_score: old_match.team_b_score,
                    };

                    new_stage.matches.insert(match_id, new_match);
                }

                new_tournament.stages.insert(stage_id, new_stage);
            }

            new_model.tournaments.insert(tournament_id, new_tournament);
        }

        Ok(new_model)
    }
}