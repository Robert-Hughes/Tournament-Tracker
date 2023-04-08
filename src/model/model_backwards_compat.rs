// Definitions for old versions of the model, used to load old serialized data and upgrade it to new versions.

// V1 (which was unversioned) had a different representation of Matches
pub(crate) mod v1 {
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
                let mut new_stage = crate::model::Stage { id: old_stage.id, name: old_stage.name, tournament_id, teams: indexmap!{}, matches: indexmap!{} };

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