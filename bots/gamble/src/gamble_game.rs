use core::fmt;

use crate::{ error::GameError, types::{ PlayerId, RollValue } };

#[derive(PartialEq, Clone, Debug)]
pub enum GameStatus {
    INITIATED,
    ONGOING,
    DONE,
}

pub trait GambleGame: fmt::Debug {
    fn add_player(&mut self, player_id: String) -> Result<(), GameError>;
    fn start(&mut self) -> Result<(), GameError>;
    fn roll(&mut self, player_id: String) -> Result<RollValue, GameError>;
    fn update(&mut self) -> Result<&GameStatus, GameError>;
    fn info(&self) -> String;
    fn wl(&self) -> Option<(PlayerId, PlayerId, RollValue)>;
}
