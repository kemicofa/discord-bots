use std::collections::{ HashMap, HashSet };

use rand::{ SeedableRng, distr::{ Distribution, Uniform }, rngs::StdRng };

use crate::{
    error::GameError,
    gamble_game::{ GambleGame, GameStatus },
    types::{ PlayerId, RollValue },
};

#[derive(Debug)]
pub struct GambleClassic {
    players_by_roll: HashMap<RollValue, Vec<PlayerId>>,
    players: HashSet<PlayerId>,
    status: GameStatus,
    die: Uniform<RollValue>,
    rng: StdRng,
    winners_to_reroll: HashSet<PlayerId>,
    losers_to_reroll: HashSet<PlayerId>,
    winner: Option<PlayerId>,
    winning_roll: Option<RollValue>,
    loser: Option<PlayerId>,
    losing_roll: Option<RollValue>,
    max_roll: RollValue,
}

const MIN_GOLD_AMOUNT: u64 = 100;
const MIN_AMOUNT_OF_PLAYERS: u64 = 2; // TODO: put this back at 2

impl GambleClassic {
    pub fn new(player_id: String, max_roll_value: u64) -> Result<Self, GameError> {
        if max_roll_value < MIN_GOLD_AMOUNT {
            return Err(GameError::GoldAmountTooSmall(MIN_GOLD_AMOUNT));
        }

        Ok(Self {
            players_by_roll: Default::default(),
            players: HashSet::from_iter([player_id]),
            status: GameStatus::INITIATED,
            die: Uniform::new_inclusive(0, max_roll_value).unwrap(),
            rng: StdRng::from_os_rng(),
            winners_to_reroll: Default::default(),
            losers_to_reroll: Default::default(),
            winner: None,
            loser: None,
            max_roll: max_roll_value,
            losing_roll: Default::default(),
            winning_roll: Default::default(),
        })
    }

    fn players_are_done_rolling(&self) -> bool {
        self.players.len() == 0
    }
}

impl GambleGame for GambleClassic {
    fn add_player(&mut self, player_id: String) -> Result<(), GameError> {
        if self.status != GameStatus::INITIATED {
            return Err(GameError::PlayerCannotJoinOngoingGame);
        }

        if self.players.contains(&player_id) {
            return Err(GameError::PlayerAlreadyPartOfGame);
        }

        self.players.insert(player_id.clone());

        Ok(())
    }

    fn start(&mut self) -> Result<(), GameError> {
        if self.status != GameStatus::INITIATED {
            return Err(GameError::CannotInitGame);
        }

        if self.players.len() < MIN_AMOUNT_OF_PLAYERS.try_into().unwrap() {
            return Err(GameError::NotEnoughPlayers(MIN_AMOUNT_OF_PLAYERS));
        }

        self.status = GameStatus::ONGOING;

        Ok(())
    }

    fn roll(&mut self, player_id: String) -> Result<RollValue, GameError> {
        if self.status != GameStatus::ONGOING || !self.players.contains(&player_id) {
            return Err(GameError::PlayerCannotRoll);
        }

        // Removing the player indicates they've now rolled.
        self.players.remove(&player_id);

        let roll_value = self.die.sample(&mut self.rng);

        if let Some(players) = self.players_by_roll.get_mut(&roll_value) {
            players.push(player_id);
        } else {
            self.players_by_roll.insert(roll_value, vec![player_id]);
        }

        Ok(roll_value)
    }

    fn update(&mut self) -> Result<&GameStatus, GameError> {
        // if not ongoing or players still rolling continue
        if self.status != GameStatus::ONGOING || !self.players_are_done_rolling() {
            return Ok(&self.status);
        }

        let mut rolls = self.players_by_roll.keys().collect::<Vec<&RollValue>>();
        rolls.sort();

        let highest_roll = rolls[rolls.len() - 1];
        let winners = self.players_by_roll.get(highest_roll).unwrap().clone();
        let lowest_roll = rolls[0];
        let losers = self.players_by_roll.get(lowest_roll).unwrap().clone();

        // Done only once to avoid applying rerolls later on.
        if self.winning_roll.is_none() {
            self.winning_roll = Some(*highest_roll);
        }

        if self.losing_roll.is_none() {
            self.losing_roll = Some(*lowest_roll);
        }

        self.winners_to_reroll = HashSet::from_iter(winners.clone());
        self.losers_to_reroll = HashSet::from_iter(losers.clone());
        self.players_by_roll.clear();

        if self.winners_to_reroll.len() > 1 {
            self.players_by_roll.clear();
            self.players = self.winners_to_reroll.clone();
            return Err(GameError::PlayersMatchedHighestRoll(winners));
        } else {
            self.winner = self.winners_to_reroll.iter().last().cloned();
        }

        if self.losers_to_reroll.len() > 1 {
            self.players_by_roll.clear();
            self.players = self.losers_to_reroll.clone();
            return Err(GameError::PlayersMatchedLowestRoll(losers.clone()));
        } else {
            self.loser = self.losers_to_reroll.iter().last().cloned();
        }

        if self.winner.is_some() && self.loser.is_some() {
            self.status = GameStatus::DONE;
        }

        Ok(&self.status)
    }

    /**
     * @deprecated gamble_classic should have no knowledge of discord formatting.
     * @todo make a mapper function for each game style to generate info based on game state.
     */
    fn info(&self) -> String {
        match self.status {
            GameStatus::INITIATED => {
                let joined_players = self.players
                    .iter()
                    .map(|player_id| format!("- <@{}>", player_id))
                    .collect::<Vec<String>>();

                let joined_players_message = format!("*Players who have already joined*\n{}", if
                    joined_players.len() > 0
                {
                    joined_players.join("\n")
                } else {
                    "- No players have joined yet".into()
                });

                return format!(
                    ":moneybag: __Ongoing Game!__\nFor **{}** gold!\n\n{}\n\n*Next steps*\n- `g!join` to join\n- `g!play` to start the game",
                    self.max_roll,
                    joined_players_message
                );
            }
            GameStatus::ONGOING => {
                let players_that_still_need_to_roll: String = self.players
                    .iter()
                    .map(|player_id| {
                        format!("- <@{}> still needs to roll! (i.e.: g!roll)", player_id)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                return format!("Game is ongoing!\n{}", players_that_still_need_to_roll);
            }
            GameStatus::DONE => {
                return format!(
                    "<@{}> owes <@{}> {} gold!",
                    self.loser.clone().unwrap(),
                    self.winner.clone().unwrap(),
                    self.max_roll
                );
            }
        }
    }

    fn wl(&self) -> Option<(PlayerId, PlayerId, RollValue)> {
        if self.winner.is_some() && self.loser.is_some() {
            return Some((
                self.winner.clone().unwrap(),
                self.loser.clone().unwrap(),
                self.winning_roll.unwrap() - self.losing_roll.unwrap(),
            ));
        }

        None
    }
}
