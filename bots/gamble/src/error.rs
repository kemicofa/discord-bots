use crate::types::PlayerId;

#[derive(Debug)]
pub enum GameError {
    CannotInitGame,
    GameAlreadyExists,
    NotEnoughPlayers(u64),
    GoldAmountTooSmall(u64),
    PlayerCannotRollOnAnInexistentGame,
    PlayerCannotRequestInfoOnInexistentGame,
    PlayerCannotPlayOnInexistentGame,
    PlayerCannotJoinAnInexistentGame,
    PlayerCannotJoinOngoingGame,
    PlayerAlreadyPartOfGame,
    PlayerCannotRoll,
    PlayerAlreadyRolled,
    PlayersMatchedLowestRoll(Vec<PlayerId>),
    PlayersMatchedHighestRoll(Vec<PlayerId>),
    NoWinnersFound,
    UnknownCommand,
}
