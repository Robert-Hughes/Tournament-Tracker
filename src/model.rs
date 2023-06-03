use indexmap::IndexMap;
use indexmap::indexmap;
use log::debug;
use log::error;
use serde::Deserialize;
use serde::Serialize;
use web_sys::Storage;
use web_sys::window;

use crate::model::tournament::Match;
use crate::model::tournament::MatchId;
use crate::model::tournament::Team;
use crate::ui::Ui;
use crate::model::{tournament::{TournamentId, Tournament, StageId, Stage, TeamId}};

use self::tournament::Fixture;
use self::tournament::FixtureId;
use self::tournament::FixtureInput;
use self::tournament::FixtureTeam;
use self::tournament::StageKind;

mod model_backwards_compat;
pub mod tournament;

// {"tournaments":{"0":{"id":0,"name":"LCS","stages":{"1":{"id":1,"tournament_id":0,"name":"Group Stage","teams":{"2":{"id":2,"name":"FNC"},"75":{"id":75,"name":"TH"},"76":{"id":76,"name":"KOI"},"77":{"id":77,"name":"XL"},"78":{"id":78,"name":"SK"},"79":{"id":79,"name":"VIT"},"80":{"id":80,"name":"BDS"},"91":{"id":91,"name":"MAD"},"92":{"id":92,"name":"G2"},"93":{"id":93,"name":"AST"}},"matches":{"81":{"id":81,"teams":[2,75],"winner":2,"loser":75},"82":{"id":82,"teams":[2,76],"winner":2,"loser":76},"84":{"id":84,"teams":[2,77],"winner":77,"loser":2},"86":{"id":86,"teams":[2,78],"winner":78,"loser":2},"88":{"id":88,"teams":[2,79],"winner":79,"loser":2},"90":{"id":90,"teams":[2,80],"winner":80,"loser":2},"95":{"id":95,"teams":[77,91],"winner":77,"loser":91},"98":{"id":98,"teams":[77,79],"winner":79,"loser":77},"100":{"id":100,"teams":[77,92],"winner":92,"loser":77},"102":{"id":102,"teams":[77,75],"winner":75,"loser":77},"104":{"id":104,"teams":[77,78],"winner":78,"loser":77},"111":{"id":111,"teams":[75,76],"winner":76,"loser":75},"113":{"id":113,"teams":[75,80],"winner":80,"loser":75},"114":{"id":114,"teams":[75,91],"winner":75,"loser":91},"116":{"id":116,"teams":[75,92],"winner":92,"loser":75},"117":{"id":117,"teams":[91,76],"winner":91,"loser":76},"119":{"id":119,"teams":[91,79],"winner":79,"loser":91},"121":{"id":121,"teams":[91,80],"winner":80,"loser":91},"122":{"id":122,"teams":[91,93],"winner":91,"loser":93},"124":{"id":124,"teams":[80,78],"winner":78,"loser":80},"126":{"id":126,"teams":[80,92],"winner":92,"loser":80},"127":{"id":127,"teams":[80,93],"winner":80,"loser":93},"129":{"id":129,"teams":[78,76],"winner":76,"loser":78},"131":{"id":131,"teams":[78,79],"winner":79,"loser":78},"132":{"id":132,"teams":[78,93],"winner":78,"loser":93},"133":{"id":133,"teams":[92,79],"winner":92,"loser":79},"135":{"id":135,"teams":[92,93],"winner":93,"loser":92},"137":{"id":137,"teams":[92,76],"winner":76,"loser":92},"139":{"id":139,"teams":[76,93],"winner":93,"loser":76},"140":{"id":140,"teams":[93,79],"winner":93,"loser":79},"141":{"id":141,"teams":[93,75],"winner":93,"loser":75}}}}}},"next_id":142}
// {"tournaments":{"0":{"id":0,"name":"LCS","stages":{"1":{"id":1,"tournament_id":0,"name":"Group Stage","teams":{"2":{"id":2,"name":"FNC"},"75":{"id":75,"name":"TH"},"76":{"id":76,"name":"KOI"},"77":{"id":77,"name":"XL"},"78":{"id":78,"name":"SK"},"79":{"id":79,"name":"VIT"},"80":{"id":80,"name":"BDS"},"91":{"id":91,"name":"MAD"},"92":{"id":92,"name":"G2"},"93":{"id":93,"name":"AST"}},"matches":{"81":{"id":81,"teams":[2,75],"winner":2,"loser":75},"82":{"id":82,"teams":[2,76],"winner":2,"loser":76},"84":{"id":84,"teams":[2,77],"winner":77,"loser":2},"86":{"id":86,"teams":[2,78],"winner":78,"loser":2},"88":{"id":88,"teams":[2,79],"winner":79,"loser":2},"90":{"id":90,"teams":[2,80],"winner":80,"loser":2},"95":{"id":95,"teams":[77,91],"winner":77,"loser":91},"98":{"id":98,"teams":[77,79],"winner":79,"loser":77},"100":{"id":100,"teams":[77,92],"winner":92,"loser":77},"102":{"id":102,"teams":[77,75],"winner":75,"loser":77},"104":{"id":104,"teams":[77,78],"winner":78,"loser":77},"111":{"id":111,"teams":[75,76],"winner":76,"loser":75},"113":{"id":113,"teams":[75,80],"winner":80,"loser":75},"114":{"id":114,"teams":[75,91],"winner":75,"loser":91},"116":{"id":116,"teams":[75,92],"winner":92,"loser":75},"117":{"id":117,"teams":[91,76],"winner":91,"loser":76},"119":{"id":119,"teams":[91,79],"winner":79,"loser":91},"121":{"id":121,"teams":[91,80],"winner":80,"loser":91},"122":{"id":122,"teams":[91,93],"winner":91,"loser":93},"124":{"id":124,"teams":[80,78],"winner":78,"loser":80},"126":{"id":126,"teams":[80,92],"winner":92,"loser":80},"127":{"id":127,"teams":[80,93],"winner":80,"loser":93},"129":{"id":129,"teams":[78,76],"winner":76,"loser":78},"131":{"id":131,"teams":[78,79],"winner":79,"loser":78},"132":{"id":132,"teams":[78,93],"winner":78,"loser":93},"133":{"id":133,"teams":[92,79],"winner":92,"loser":79},"135":{"id":135,"teams":[92,93],"winner":93,"loser":92},"137":{"id":137,"teams":[92,76],"winner":76,"loser":92},"139":{"id":139,"teams":[76,93],"winner":93,"loser":76},"140":{"id":140,"teams":[93,79],"winner":93,"loser":79},"141":{"id":141,"teams":[93,75],"winner":93,"loser":75},"142":{"id":142,"teams":[76,77],"winner":76,"loser":77},"144":{"id":144,"teams":[80,79],"winner":80,"loser":79},"145":{"id":145,"teams":[2,91],"winner":2,"loser":91},"146":{"id":146,"teams":[92,78],"winner":92,"loser":78},"147":{"id":147,"teams":[93,77],"winner":93,"loser":77},"148":{"id":148,"teams":[91,78],"winner":91,"loser":78},"149":{"id":149,"teams":[80,76],"winner":80,"loser":76},"150":{"id":150,"teams":[79,75],"winner":79,"loser":75},"151":{"id":151,"teams":[2,92],"winner":2,"loser":92},"152":{"id":152,"teams":[80,77],"winner":80,"loser":77},"153":{"id":153,"teams":[93,2],"winner":93,"loser":2},"154":{"id":154,"teams":[75,78],"winner":75,"loser":78}}}}},"143":{"id":143,"name":"test","stages":{}}},"next_id":155}
// {"tournaments":{"0":{"id":0,"name":"LCS","stages":{"1":{"id":1,"tournament_id":0,"name":"Group Stage","teams":{"2":{"id":2,"name":"FNC"},"75":{"id":75,"name":"TH"},"76":{"id":76,"name":"KOI"},"77":{"id":77,"name":"XL"},"78":{"id":78,"name":"SK"},"79":{"id":79,"name":"VIT"},"80":{"id":80,"name":"BDS"},"91":{"id":91,"name":"MAD"},"92":{"id":92,"name":"G2"},"93":{"id":93,"name":"AST"}},"matches":{"81":{"id":81,"teams":[2,75],"winner":2,"loser":75},"82":{"id":82,"teams":[2,76],"winner":2,"loser":76},"84":{"id":84,"teams":[2,77],"winner":77,"loser":2},"86":{"id":86,"teams":[2,78],"winner":78,"loser":2},"88":{"id":88,"teams":[2,79],"winner":79,"loser":2},"90":{"id":90,"teams":[2,80],"winner":80,"loser":2},"95":{"id":95,"teams":[77,91],"winner":77,"loser":91},"98":{"id":98,"teams":[77,79],"winner":79,"loser":77},"100":{"id":100,"teams":[77,92],"winner":92,"loser":77},"102":{"id":102,"teams":[77,75],"winner":75,"loser":77},"104":{"id":104,"teams":[77,78],"winner":78,"loser":77},"111":{"id":111,"teams":[75,76],"winner":76,"loser":75},"113":{"id":113,"teams":[75,80],"winner":80,"loser":75},"114":{"id":114,"teams":[75,91],"winner":75,"loser":91},"116":{"id":116,"teams":[75,92],"winner":92,"loser":75},"117":{"id":117,"teams":[91,76],"winner":91,"loser":76},"119":{"id":119,"teams":[91,79],"winner":79,"loser":91},"121":{"id":121,"teams":[91,80],"winner":80,"loser":91},"122":{"id":122,"teams":[91,93],"winner":91,"loser":93},"124":{"id":124,"teams":[80,78],"winner":78,"loser":80},"126":{"id":126,"teams":[80,92],"winner":92,"loser":80},"127":{"id":127,"teams":[80,93],"winner":80,"loser":93},"129":{"id":129,"teams":[78,76],"winner":76,"loser":78},"131":{"id":131,"teams":[78,79],"winner":79,"loser":78},"132":{"id":132,"teams":[78,93],"winner":78,"loser":93},"133":{"id":133,"teams":[92,79],"winner":92,"loser":79},"135":{"id":135,"teams":[92,93],"winner":93,"loser":92},"137":{"id":137,"teams":[92,76],"winner":76,"loser":92},"139":{"id":139,"teams":[76,93],"winner":93,"loser":76},"140":{"id":140,"teams":[93,79],"winner":93,"loser":79},"141":{"id":141,"teams":[93,75],"winner":93,"loser":75},"142":{"id":142,"teams":[76,77],"winner":76,"loser":77},"144":{"id":144,"teams":[80,79],"winner":80,"loser":79},"145":{"id":145,"teams":[2,91],"winner":2,"loser":91},"146":{"id":146,"teams":[92,78],"winner":92,"loser":78},"148":{"id":148,"teams":[91,78],"winner":91,"loser":78},"149":{"id":149,"teams":[80,76],"winner":80,"loser":76},"150":{"id":150,"teams":[79,75],"winner":79,"loser":75},"151":{"id":151,"teams":[2,92],"winner":2,"loser":92},"152":{"id":152,"teams":[80,77],"winner":80,"loser":77},"153":{"id":153,"teams":[93,2],"winner":93,"loser":2},"154":{"id":154,"teams":[75,78],"winner":75,"loser":78},"171":{"id":171,"teams":[77,93],"winner":93,"loser":77},"172":{"id":172,"teams":[92,91],"winner":92,"loser":91},"174":{"id":174,"teams":[76,79],"winner":79,"loser":76}}}}},"143":{"id":143,"name":"test","stages":{"155":{"id":155,"tournament_id":143,"name":"test","teams":{"156":{"id":156,"name":"TST"},"157":{"id":157,"name":"BST"},"158":{"id":158,"name":"WAJ"}},"matches":{"161":{"id":161,"teams":[157,158],"winner":157,"loser":158},"163":{"id":163,"teams":[156,158],"winner":158,"loser":156},"167":{"id":167,"teams":[157,156],"winner":157,"loser":156}}}}}},"next_id":175}
// {"tournaments":{"0":{"id":0,"name":"LCS","stages":{"1":{"id":1,"tournament_id":0,"name":"Group Stage","teams":{"2":{"id":2,"name":"FNC"},"75":{"id":75,"name":"TH"},"76":{"id":76,"name":"KOI"},"77":{"id":77,"name":"XL"},"78":{"id":78,"name":"SK"},"79":{"id":79,"name":"VIT"},"80":{"id":80,"name":"BDS"},"91":{"id":91,"name":"MAD"},"92":{"id":92,"name":"G2"},"93":{"id":93,"name":"AST"}},"matches":{"81":{"id":81,"teams":[2,75],"winner":2,"loser":75},"82":{"id":82,"teams":[2,76],"winner":2,"loser":76},"84":{"id":84,"teams":[2,77],"winner":77,"loser":2},"86":{"id":86,"teams":[2,78],"winner":78,"loser":2},"88":{"id":88,"teams":[2,79],"winner":79,"loser":2},"90":{"id":90,"teams":[2,80],"winner":80,"loser":2},"95":{"id":95,"teams":[77,91],"winner":77,"loser":91},"98":{"id":98,"teams":[77,79],"winner":79,"loser":77},"100":{"id":100,"teams":[77,92],"winner":92,"loser":77},"102":{"id":102,"teams":[77,75],"winner":75,"loser":77},"104":{"id":104,"teams":[77,78],"winner":78,"loser":77},"111":{"id":111,"teams":[75,76],"winner":76,"loser":75},"113":{"id":113,"teams":[75,80],"winner":80,"loser":75},"114":{"id":114,"teams":[75,91],"winner":75,"loser":91},"116":{"id":116,"teams":[75,92],"winner":92,"loser":75},"117":{"id":117,"teams":[91,76],"winner":91,"loser":76},"119":{"id":119,"teams":[91,79],"winner":79,"loser":91},"121":{"id":121,"teams":[91,80],"winner":80,"loser":91},"122":{"id":122,"teams":[91,93],"winner":91,"loser":93},"124":{"id":124,"teams":[80,78],"winner":78,"loser":80},"126":{"id":126,"teams":[80,92],"winner":92,"loser":80},"127":{"id":127,"teams":[80,93],"winner":80,"loser":93},"129":{"id":129,"teams":[78,76],"winner":76,"loser":78},"131":{"id":131,"teams":[78,79],"winner":79,"loser":78},"132":{"id":132,"teams":[78,93],"winner":78,"loser":93},"133":{"id":133,"teams":[92,79],"winner":92,"loser":79},"135":{"id":135,"teams":[92,93],"winner":93,"loser":92},"137":{"id":137,"teams":[92,76],"winner":76,"loser":92},"139":{"id":139,"teams":[76,93],"winner":93,"loser":76},"140":{"id":140,"teams":[93,79],"winner":93,"loser":79},"141":{"id":141,"teams":[93,75],"winner":93,"loser":75},"142":{"id":142,"teams":[76,77],"winner":76,"loser":77},"144":{"id":144,"teams":[80,79],"winner":80,"loser":79},"145":{"id":145,"teams":[2,91],"winner":2,"loser":91},"146":{"id":146,"teams":[92,78],"winner":92,"loser":78},"148":{"id":148,"teams":[91,78],"winner":91,"loser":78},"149":{"id":149,"teams":[80,76],"winner":80,"loser":76},"150":{"id":150,"teams":[79,75],"winner":79,"loser":75},"151":{"id":151,"teams":[2,92],"winner":2,"loser":92},"152":{"id":152,"teams":[80,77],"winner":80,"loser":77},"153":{"id":153,"teams":[93,2],"winner":93,"loser":2},"154":{"id":154,"teams":[75,78],"winner":75,"loser":78},"171":{"id":171,"teams":[77,93],"winner":93,"loser":77},"172":{"id":172,"teams":[92,91],"winner":92,"loser":91},"174":{"id":174,"teams":[76,79],"winner":79,"loser":76}}}}},"143":{"id":143,"name":"test","stages":{"155":{"id":155,"tournament_id":143,"name":"test","teams":{"156":{"id":156,"name":"TST"},"157":{"id":157,"name":"BST"},"158":{"id":158,"name":"WAJ"}},"matches":{"161":{"id":161,"teams":[157,158],"winner":157,"loser":158},"163":{"id":163,"teams":[156,158],"winner":158,"loser":156}}}}}},"next_id":175}

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
    // The version is stored separately, so that we don't need to deserialize the model in order to check the version.
    // Beware if this key name is changed, we won't be able to load old data!!
    const LOCAL_STORAGE_VERSION_KEY: &str = "tournament-tracker-version";
    const VERSION: i32 = 3;
    const LOCAL_STORAGE_MODEL_KEY: &str = "tournament-tracker-model";

    pub fn new() -> Model {
        Model { tournaments: indexmap!{}, next_id: 0, changed_tournaments: vec![] }
    }

    pub fn load() -> Model {
        debug!("Loading!");

        let storage = window().expect("Missing window").local_storage().expect("Error getting localStorage").expect("Missing localStorage");

        // Check the version, as we may need to deserialize differently for backwards compatibility
        let res = match storage.get_item(Model::LOCAL_STORAGE_VERSION_KEY) {
            Ok(Some(x)) if x == Model::VERSION.to_string() => {
                Self::load_current_version(storage)
            },
            Ok(Some(x)) if &x == "2" => {
                model_backwards_compat::v2::load_and_upgrade(storage)
            },
            _ => {
                // Assume is old version before we added versioning
                model_backwards_compat::v1::load_and_upgrade(storage)
            }
        };
        match res {
            Ok(r) => r,
            Err(e) => {
                if window().unwrap().confirm_with_message(&format!("Failed to load saved data: {:?}! If this is expected then click OK and it will be reset. Otherwise check what's going on.",
                    e)) == Ok(true) {
                    Model::new()
                } else {
                    panic!("No data!");
                }
            }
        }
    }

    fn load_current_version(storage: Storage) -> Result<Model, String> {
        match storage.get_item(Model::LOCAL_STORAGE_MODEL_KEY) {
            Ok(Some(s)) => {
                match serde_json::from_str::<Model>(&s) {
                    Ok(m) => Ok(m),
                    Err(e) => Err(format!("Failed to deserialize data: {:?}", e)),
                }
            }
            e => Err(format!("Failed to load from local storage: {:?}", e)),
        }
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

    pub fn delete_tournament(&mut self, tournament_id: TournamentId) -> Result<(), ()> {
        if self.tournaments.shift_remove(&tournament_id).is_some() {
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn rename_tournament(&mut self, tournament_id: TournamentId, new_name: &str) -> Result<(), ()> {
        if let Some(t) = self.tournaments.get_mut(&tournament_id) {
            t.name = new_name.to_string();
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_stage_round_robin(&mut self, tournament_id: TournamentId, name: String) -> Option<StageId> {
        let id = self.get_next_id();
        if let Some(t) = self.tournaments.get_mut(&tournament_id) {
            t.stages.insert(id, Stage::new_round_robin(id, t.id, name));
            let tid = t.id;
            self.changed_tournaments.push(tid);
            Some(id)
        } else {
            None
        }
    }

    pub fn add_stage_bracket(&mut self, tournament_id: TournamentId, name: String) -> Option<StageId> {
        let id = self.get_next_id();
        if let Some(t) = self.tournaments.get_mut(&tournament_id) {
            t.stages.insert(id, Stage::new_bracket(id, t.id, name));
            let tid = t.id;
            self.changed_tournaments.push(tid);
            Some(id)
        } else {
            None
        }
    }

    pub fn delete_stage(&mut self, tournament_id: TournamentId, stage_id: StageId) -> Result<(), ()> {
        if self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.shift_remove(&stage_id)).is_some() {
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn rename_stage(&mut self, tournament_id: TournamentId, stage_id: StageId, new_name: &str) -> Result<(), ()> {
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            s.name = new_name.to_string();
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
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
            s.matches.retain(|_, m| !m.contains(team_id));

            if s.teams.shift_remove(&team_id).is_none() {
                return Err(());
            }
            self.changed_tournaments.push(tournament_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_match(&mut self, tournament_id: TournamentId, stage_id: StageId, team_a: TeamId, team_b: TeamId, team_a_score: u32, team_b_score: u32) -> Option<MatchId> {
        let id = self.get_next_id();
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            if !s.teams.contains_key(&team_a) || !s.teams.contains_key(&team_b) {
                return None;
            }
            if team_a == team_b {
                return None;
            }

            s.matches.insert(id, Match { id, team_a, team_b, team_a_score, team_b_score });
            self.changed_tournaments.push(tournament_id);
            return Some(id)
        }
        return None;
    }

    pub fn delete_match(&mut self, tournament_id: TournamentId, stage_id: StageId, match_id: MatchId) -> Result<(), ()> {
        if let Some(s) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)) {
            //TODO: what about any fixtures that were linked to this match?
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

    pub fn add_fixture(&mut self, tournament_id: TournamentId, stage_id: StageId, layout: (i32, i32), team_a: FixtureTeam, team_b: FixtureTeam) -> Option<FixtureId> {
        let id = self.get_next_id();
        if let Some(StageKind::Bracket { fixtures }) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)).and_then(|s| Some(&mut s.kind)) {
            fixtures.insert(id, Fixture { id, layout, match_id: None, team_a, team_b });
            self.changed_tournaments.push(tournament_id);
            return Some(id)
        }
        return None;
    }

    pub fn set_fixture_layout(&mut self, tournament_id: TournamentId, stage_id: StageId, fixture_id: FixtureId, layout: (i32, i32)) -> Result<(), ()> {
        if let Some(StageKind::Bracket { fixtures }) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)).and_then(|s| Some(&mut s.kind)) {
            if let Some(f) = fixtures.get_mut(&fixture_id) {
                f.layout = layout;
                self.changed_tournaments.push(tournament_id);
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn set_fixture_input(&mut self, tournament_id: TournamentId, stage_id: StageId, fixture_id: FixtureId, input: FixtureInput, team: FixtureTeam) -> Result<(), ()> {
        //TODO: if a match already exists for the fixture, does it need updating at all?
        if let Some(StageKind::Bracket { fixtures }) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)).and_then(|s| Some(&mut s.kind)) {
            if let Some(f) = fixtures.get_mut(&fixture_id) {
                match input {
                    FixtureInput::TeamA => f.team_a = team,
                    FixtureInput::TeamB => f.team_b = team,
                }
                self.changed_tournaments.push(tournament_id);
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn delete_fixture(&mut self, tournament_id: TournamentId, stage_id: StageId, fixture_id: FixtureId) -> Result<(), ()> {
        if let Some(StageKind::Bracket { fixtures }) = self.tournaments.get_mut(&tournament_id).and_then(|t| t.stages.get_mut(&stage_id)).and_then(|s| Some(&mut s.kind)) {
            //TODO: what about other fixtures that were linked to this one?
            if fixtures.shift_remove(&fixture_id).is_none() {
                return Err(());
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

        let storage = window().expect("Missing window").local_storage().expect("Error getting localStorage").expect("Missing localStorage");

        // Store version
        storage.set_item(Model::LOCAL_STORAGE_VERSION_KEY, &Model::VERSION.to_string()).expect("Failed to save");

        match serde_json::to_string(&self) {
            Ok(s) => storage.set_item(Model::LOCAL_STORAGE_MODEL_KEY, &s).expect("Failed to save"),
            Err(e) => error!("Error saving: {e}"),
        };
    }
}
