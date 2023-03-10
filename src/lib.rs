use std::{error::Error, fmt::Display};

use regex::Regex;

mod resolver;

#[derive(Debug)]
pub enum GGSTDLError {
    UnknownCharacter, UnknownMove
}

impl Display for GGSTDLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GGSTDLError::UnknownCharacter => write!(f, "Unknown character"),
            GGSTDLError::UnknownMove => write!(f, "Unknown move"),
        }
    }
}

impl Error for GGSTDLError {}

#[derive(Debug)]
pub struct GGSTDLData {
    characters: Vec<Character>
}

impl GGSTDLData {
    pub fn find_character(&self, char_query: &str) -> Result<&Character, GGSTDLError> {
        self.characters.iter().find(|c| c.regex.is_match(char_query))
            .ok_or(GGSTDLError::UnknownCharacter)
    }

    pub fn find_move(&self, char_query: &str, move_query: &str) -> Result<&Move, GGSTDLError> {
        let character = self.find_character(char_query)?;
        character.moves.iter().find(|m| m.regex.is_match(move_query))
            .ok_or(GGSTDLError::UnknownMove)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterId {
    TESTAMENT, JACKO, NAGORIYUKI, MILLIA, CHIPP, SOL, KY, MAY, ZATO, INO, HAPPYCHAOS, 
    SIN, BAIKEN, ANJI, LEO, FAUST, AXL, POTEMKIN, RAMLETHAL, GIO, GOLDLEWIS, BRIDGET,
}

impl CharacterId {
    pub const ALL: [CharacterId; 22] = [
        CharacterId::TESTAMENT, CharacterId::JACKO, CharacterId::NAGORIYUKI, CharacterId::MILLIA, CharacterId::CHIPP, 
        CharacterId::SOL, CharacterId::KY, CharacterId::MAY, CharacterId::ZATO, CharacterId::INO, CharacterId::HAPPYCHAOS, 
        CharacterId::SIN, CharacterId::BAIKEN, CharacterId::ANJI, CharacterId::LEO, CharacterId::FAUST, CharacterId::AXL, 
        CharacterId::POTEMKIN, CharacterId::RAMLETHAL, CharacterId::GIO, CharacterId::GOLDLEWIS, CharacterId::BRIDGET
    ];
}

#[derive(Debug)]
pub struct Character {
    pub id: CharacterId,
    pub regex: Regex,
    pub frame_data_url: String,
    pub moves: Vec<Move>
}

impl Character {
    pub fn new(id: CharacterId, regex: &str, frame_data_url: &str) -> Character {
        Character {
            id, 
            regex: Regex::new(regex).unwrap(), 
            frame_data_url: String::from(frame_data_url),
             moves: vec![] 
        }
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    pub regex: Regex,
    pub input: String,
    pub name: String,
    pub damage: String,
    pub guard: String,
    pub startup: String, 
    pub active: String,
    pub recovery: String,
    pub onblock: String,
    pub onhit: String,
    pub level: String,
    pub counterhit_type: String,
    pub invuln: String,
    pub proration: String,
    pub risc_gain: String,
    pub risc_loss: String,
    pub image: String,
    pub hitboxes: String
}

pub async fn load() -> Result<GGSTDLData, Box<dyn Error>> {

    let mut characters: Vec<Character> = vec![
        Character::new(CharacterId::TESTAMENT, r"(?i)(test)", "https://www.dustloop.com/wiki/index.php?title=GGST/Testament/Frame_Data"),
        Character::new(CharacterId::JACKO, r"(?i)(jack)", "https://www.dustloop.com/wiki/index.php?title=GGST/Jack-O/Frame_Data"),
        Character::new(CharacterId::NAGORIYUKI, r"(?i)(nago)", "https://www.dustloop.com/wiki/index.php?title=GGST/Nagoriyuki/Frame_Data"),
        Character::new(CharacterId::MILLIA, r"(?i)(millia|milia)", "https://www.dustloop.com/wiki/index.php?title=GGST/Millia_Rage/Frame_Data"),
        Character::new(CharacterId::CHIPP, r"(?i)(chip)", "https://www.dustloop.com/wiki/index.php?title=GGST/Chipp_Zanuff/Frame_Data"),
        Character::new(CharacterId::SOL, r"(?i)(sol)", "https://www.dustloop.com/wiki/index.php?title=GGST/Sol_Badguy/Frame_Data"),
        Character::new(CharacterId::KY, r"(?i)(ky)", "https://www.dustloop.com/wiki/index.php?title=GGST/Ky_Kiske/Frame_Data"),
        Character::new(CharacterId::MAY, r"(?i)(may)", "https://www.dustloop.com/wiki/index.php?title=GGST/May/Frame_Data"),
        Character::new(CharacterId::ZATO, r"(?i)(zato)", "https://www.dustloop.com/wiki/index.php?title=GGST/Zato-1/Frame_Data"),
        Character::new(CharacterId::INO, r"(?i)(ino|i-no)", "https://www.dustloop.com/wiki/index.php?title=GGST/I-No/Frame_Data"),
        Character::new(CharacterId::HAPPYCHAOS, r"(?i)(hc|chaos|happy)", "https://www.dustloop.com/wiki/index.php?title=GGST/Happy_Chaos/Frame_Data"),
        Character::new(CharacterId::SIN, r"(?i)(sin)", "https://www.dustloop.com/wiki/index.php?title=GGST/Sin_Kiske/Frame_Data"),
        Character::new(CharacterId::BAIKEN, r"(?i)(baiken)", "https://www.dustloop.com/wiki/index.php?title=GGST/Baiken/Frame_Data"),
        Character::new(CharacterId::ANJI, r"(?i)(anji)", "https://www.dustloop.com/wiki/index.php?title=GGST/Anji_Mito/Frame_Data"),
        Character::new(CharacterId::LEO, r"(?i)(leo)", "https://www.dustloop.com/wiki/index.php?title=GGST/Leo_Whitefang/Frame_Data"),
        Character::new(CharacterId::FAUST, r"(?i)(faust)", "https://www.dustloop.com/wiki/index.php?title=GGST/Faust/Frame_Data"),
        Character::new(CharacterId::AXL, r"(?i)(axl)", "https://www.dustloop.com/wiki/index.php?title=GGST/Axl_Low/Frame_Data"),
        Character::new(CharacterId::POTEMKIN, r"(?i)(pot)", "https://www.dustloop.com/wiki/index.php?title=GGST/Potemkin/Frame_Data"),
        Character::new(CharacterId::RAMLETHAL, r"(?i)(ram)", "https://www.dustloop.com/wiki/index.php?title=GGST/Ramlethal_Valentine/Frame_Data"),
        Character::new(CharacterId::GIO, r"(?i)(gio)", "https://www.dustloop.com/wiki/index.php?title=GGST/Giovanna/Frame_Data"),
        Character::new(CharacterId::GOLDLEWIS, r"(?i)(lewis|gold|goldlewis|gl|dick)", "https://www.dustloop.com/wiki/index.php?title=GGST/Goldlewis_Dickinson/Frame_Data"),
        Character::new(CharacterId::BRIDGET, r"(?i)(bridget)", "https://www.dustloop.com/wiki/index.php?title=GGST/Bridget/Frame_Data"),
    ];

    for character in characters.iter_mut() {
        resolver::import_moves(character).await?;
        println!("Loaded moves for {:?} : {}", character.id, character.moves.len());
    }

    Ok(GGSTDLData { characters })

}