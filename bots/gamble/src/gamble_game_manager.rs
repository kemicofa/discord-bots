use std::{ collections::HashMap };

use crate::{
    error::GameError,
    gamble_classic::GambleClassic,
    gamble_game::{ GambleGame, GameStatus },
    types::{ PlayerId, RollValue },
};

type BoxedGameGame = Box<dyn GambleGame + Send + Sync>;

type GameMap = HashMap<String, BoxedGameGame>;

pub enum GGMResponse {
    Empty,
    ShowJoinInfo,
    Started,
    PlayerRolled(RollValue),
    Done((PlayerId, PlayerId, RollValue)),
    ShowGeneralInfo(String),
    Message(String),
}

pub struct GambleGameManager {
    map: GameMap,
}

impl Default for GambleGameManager {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

const HELP: &str =
    r#"
:moneybag: **Gamble Game!**
*Great way to lose gold in your favorite game.*
- `g!create <GOLD_AMOUNT>`  __Create a unique game in a channel__
- `g!join` __Join a new game__
- `g!play` __Start a new game__
- `g!roll` __Roll__
- `g!help` __List all available commands__
- `g!info` __List information about the current game__
"#;

impl GambleGameManager {
    fn create(
        &mut self,
        game_id: String,
        user_id: String,
        max_roll: u64
    ) -> Result<GGMResponse, GameError> {
        if self.map.contains_key(&game_id) {
            return Err(GameError::GameAlreadyExists);
        }

        let gamble_classic = GambleClassic::new(user_id, max_roll)?;

        let boxed = Box::new(gamble_classic);
        self.map.insert(game_id, boxed);

        return Ok(GGMResponse::ShowJoinInfo);
    }

    fn join(&mut self, game_id: String, player_id: String) -> Result<GGMResponse, GameError> {
        match self.map.get_mut(&game_id) {
            Some(game) => {
                game.add_player(player_id)?;

                return Ok(GGMResponse::Empty);
            }
            None => {
                return Err(GameError::PlayerCannotJoinAnInexistentGame);
            }
        }
    }

    fn play(&mut self, game_id: String) -> Result<GGMResponse, GameError> {
        match self.map.get_mut(&game_id) {
            Some(game) => {
                game.start()?;
                return Ok(GGMResponse::Started);
            }
            None => {
                return Err(GameError::PlayerCannotPlayOnInexistentGame);
            }
        }
    }

    fn roll(&mut self, game_id: String, player_id: String) -> Result<GGMResponse, GameError> {
        match self.map.get_mut(&game_id) {
            Some(game) => {
                let roll_value = game.roll(player_id.clone())?;

                return Ok(GGMResponse::PlayerRolled(roll_value));
            }
            None => {
                return Err(GameError::PlayerCannotRollOnAnInexistentGame);
            }
        }
    }

    fn info(&self, game_id: String) -> Result<GGMResponse, GameError> {
        match self.map.get(&game_id) {
            Some(game) => {
                return Ok(GGMResponse::ShowGeneralInfo(game.info()));
            }
            None => {
                return Err(GameError::PlayerCannotRequestInfoOnInexistentGame);
            }
        }
    }

    pub fn tick(&mut self, channel_id: String) -> Result<GGMResponse, GameError> {
        match self.map.get_mut(&channel_id) {
            Some(game) => {
                let status = game.update()?;

                if *status != GameStatus::DONE {
                    return Ok(GGMResponse::Empty);
                }

                let winner_and_loser = game.wl();

                if winner_and_loser.is_none() {
                    return Err(GameError::NoWinnersFound);
                }

                // Once the game is done, delete it from the map so the players can create a new one.
                self.map.remove(&channel_id);

                return Ok(GGMResponse::Done(winner_and_loser.unwrap()));
            }
            None => {
                return Ok(GGMResponse::Empty);
            }
        }
    }

    pub fn execute(
        &mut self,
        channel_id: String,
        user_id: String,
        command: &str,
        args: Vec<&str>
    ) -> Result<GGMResponse, GameError> {
        match command {
            "g!create" => {
                let max_roll = args.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
                return self.create(channel_id, user_id, max_roll);
            }
            "g!join" => {
                return self.join(channel_id, user_id);
            }
            "g!play" => {
                return self.play(channel_id);
            }
            "g!roll" => {
                return self.roll(channel_id, user_id);
            }
            "g!help" => {
                return Ok(GGMResponse::Message(HELP.to_string()));
            }
            "g!info" => {
                return self.info(channel_id);
            }
            _ => {
                return Err(GameError::UnknownCommand);
            }
        }
    }
}
