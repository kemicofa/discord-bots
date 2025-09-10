use crate::{ error::GameError, gamble_game_manager::GGMResponse, types::PlayerId };

fn fmt_discord_name(player_id: &String) -> String {
    format!("<@{}>", player_id)
}

pub fn build_matched_roll_message(roll_type: String, player_ids: &Vec<PlayerId>) -> String {
    player_ids
        .iter()
        .map(|player_id| {
            format!("<@{}>, you matched the {} roll. Please reroll.", player_id, roll_type)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn map_game_error_to_discord_message(player_id: &PlayerId, error: GameError) -> String {
    let player = fmt_discord_name(player_id);

    match error {
        GameError::PlayerCannotRollOnAnInexistentGame =>
            format!("{}, __you absolute dipshit__, what are you rolling for? (i.e.: `g!create`)", player),
        GameError::PlayerCannotRequestInfoOnInexistentGame =>
            format!("{}, __you're a lost cause__, you gotta create a game first before requesting info. (i.e.: `g!create`)", player),
        GameError::NoWinnersFound =>
            ":thinking: wtf, no winners were found.. but the game is done? Holy fuck.".into(),
        GameError::PlayerCannotPlayOnInexistentGame =>
            format!("{}, __you absolute mongoloid__, you gotta create a game first before playing. (i.e.: `g!create`)", player),
        GameError::PlayerCannotJoinAnInexistentGame =>
            format!("{}, __you absolute donut__, you gotta create a game first before joining one. (i.e.: `g!create`)", player),
        GameError::CannotInitGame =>
            format!(":man_facepalming: {}, bro there is already an ongoing game. (i.e.: `g!info`)", player),
        GameError::NotEnoughPlayers(minimum_number_of_players) =>
            format!(
                ":upside_down: {}, there needs to be at least {} players.",
                player,
                minimum_number_of_players
            ),
        GameError::GoldAmountTooSmall(minimum_gold_amount) =>
            format!(
                ":pinched_fingers: {}, what are you broke? Gamble at least {} gold.",
                player,
                fmt_amount(minimum_gold_amount)
            ),
        GameError::PlayerCannotJoinOngoingGame =>
            format!(":weary: {}, let the game end first and then join the next one.", player),
        GameError::PlayerAlreadyPartOfGame =>
            format!(":zany_face: {}, you're already part of the game dipshit.", player),
        GameError::PlayerCannotRoll =>
            format!(":unamused: {}, it's not the right time to roll.", player),
        GameError::PlayerAlreadyRolled =>
            format!(":expresionless: {}, you think rolling twice is going to help your cause?", player),
        GameError::PlayersMatchedLowestRoll(items) =>
            build_matched_roll_message("lowest".into(), &items),
        GameError::PlayersMatchedHighestRoll(items) =>
            build_matched_roll_message("highest".into(), &items),
        GameError::GameAlreadyExists =>
            format!("{}, a game already exists in this channel.. try finishing it first?", player),
        GameError::UnknownCommand =>
            format!("{}, is this your first time? (i.e.: `g!help`)", player),
    }
}

pub fn map_ggm_response_to_discord_message(
    player_id: &PlayerId,
    response: GGMResponse
) -> Option<String> {
    let player = fmt_discord_name(player_id);

    match response {
        GGMResponse::Started => Some("Game started :rocket:! Type `g!roll`!".into()),
        GGMResponse::Empty => None,
        GGMResponse::ShowJoinInfo => Some("Type `g!join` to join the game!".into()),
        GGMResponse::Done((winner_id, loser_id, amount)) => {
            let winner = fmt_discord_name(&winner_id);
            let loser = fmt_discord_name(&loser_id);

            Some(
                format!(
                    "__A winner has emerged!__\n:coin: {} owes {} **{}** gold.",
                    loser,
                    winner,
                    fmt_amount(amount)
                )
            )
        }
        GGMResponse::PlayerRolled(roll_value) =>
            Some(format!("{} rolled a {}!", player, fmt_amount(roll_value))),
        GGMResponse::ShowGeneralInfo(info) => Some(info),
        GGMResponse::Message(message) => Some(message),
    }
}

pub fn fmt_amount(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, ch) in chars.iter().enumerate() {
        result.push(*ch);
        if (len - i - 1) % 3 == 0 && i != len - 1 {
            result.push(' ');
        }
    }
    result
}
