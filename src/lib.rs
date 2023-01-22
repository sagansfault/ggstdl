use std::{error::Error, hash::Hash};
use regex::Regex;
use resolver::move_import::MOVE_IMPORT_RESOLVERS;
use scraper::Html;

pub mod resolver;

#[derive(Debug)]
pub struct Character {
    pub dustloop: String,
    pub id: CharacterId,
    pub regex: Regex,
    pub moves: Vec<Move>
}

impl Character {
    fn new(id: CharacterId, regex: &str, dustloop: &str) -> Character {
        Character {
            dustloop: String::from(dustloop), 
            id,
            regex: Regex::new(regex).unwrap(),
            moves: vec![]
        }
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
pub struct Move {
    pub name: String,
    pub matcher: Regex,
    pub guard: String,
    pub damage: String,
    pub startup: String,
    pub active: String,
    pub recovery: String,
    pub onblock: String,
    pub invuln: String,
}

impl Move {
    pub fn format(&self, verbose: bool) -> String {
        if verbose {
            format!("{}: g=({}) s=({}) a=({}) r=({}) b=({}) i=({})", self.name, self.guard, self.startup, self.active, self.recovery, self.onblock, self.invuln)
        } else {
            format!("{}: s=({}) a=({}) r=({}) b=({})", self.name, self.startup, self.active, self.recovery, self.onblock)
        }
    }
}

pub async fn load() -> Result<Vec<Character>, Box<dyn Error>> {
    let mut characters: Vec<Character> = vec![
        Character::new(CharacterId::TESTAMENT, r"(?i)(test)", "https://www.dustloop.com/w/GGST/Testament"),
        Character::new(CharacterId::JACKO, r"(?i)(jack)", "https://www.dustloop.com/w/GGST/Jack-O"),
        Character::new(CharacterId::NAGORIYUKI, r"(?i)(nago)", "https://www.dustloop.com/w/GGST/Nagoriyuki"),
        Character::new(CharacterId::MILLIA, r"(?i)(millia|milia)", "https://www.dustloop.com/w/GGST/Millia_Rage"),
        Character::new(CharacterId::CHIPP, r"(?i)(chip)", "https://www.dustloop.com/w/GGST/Chipp"),
        Character::new(CharacterId::SOL, r"(?i)(sol)", "https://www.dustloop.com/w/GGST/Sol_Badguy"),
        Character::new(CharacterId::KY, r"(?i)(ky)", "https://www.dustloop.com/w/GGST/Ky_Kiske"),
        Character::new(CharacterId::MAY, r"(?i)(may)", "https://www.dustloop.com/w/GGST/May"),
        Character::new(CharacterId::ZATO, r"(?i)(zato)", "https://www.dustloop.com/w/GGST/Zato-1"),
        Character::new(CharacterId::INO, r"(?i)(ino|i-no)", "https://www.dustloop.com/w/GGST/I-No"),
        Character::new(CharacterId::HAPPYCHAOS, r"(?i)(hc|chaos|happy)", "https://www.dustloop.com/w/GGST/Happy_Chaos"),
        Character::new(CharacterId::SIN, r"(?i)(sin)", "https://www.dustloop.com/w/GGST/Sin_Kiske"),
        Character::new(CharacterId::BAIKEN, r"(?i)(baiken)", "https://www.dustloop.com/w/GGST/Baiken"),
        Character::new(CharacterId::ANJI, r"(?i)(anji)", "https://www.dustloop.com/w/GGST/Anji_Mito"),
        Character::new(CharacterId::LEO, r"(?i)(leo)", "https://www.dustloop.com/w/GGST/Leo_Whitefang"),
        Character::new(CharacterId::FAUST, r"(?i)(faust)", "https://www.dustloop.com/w/GGST/Faust"),
        Character::new(CharacterId::AXL, r"(?i)(axl)", "https://www.dustloop.com/w/GGST/Axl_Low"),
        Character::new(CharacterId::POTEMKIN, r"(?i)(pot)", "https://www.dustloop.com/w/GGST/Potemkin"),
        Character::new(CharacterId::RAMLETHAL, r"(?i)(ram)", "https://www.dustloop.com/w/GGST/Ramlethal_Valentine"),
        Character::new(CharacterId::GIO, r"(?i)(gio)", "https://www.dustloop.com/w/GGST/Giovanna"),
        Character::new(CharacterId::GOLDLEWIS, r"(?i)(lewis|gold|goldlewis|gl|dick)", "https://www.dustloop.com/w/GGST/Goldlewis_Dickinson"),
        Character::new(CharacterId::BRIDGET, r"(?i)(bridget)", "https://www.dustloop.com/w/GGST/Bridget"),
    ];

    for character in characters.iter_mut() {
        let res = reqwest::get(character.dustloop.as_str()).await?.text().await?;
        let document = scraper::Html::parse_document(&res);
        append_normals(character, &document)?;
        append_specials(character, &document)?;
        append_overdrives(character, &document)?;
        println!("Loaded moves for {:?} : {}", character.id, character.moves.len());
    }

    Ok(characters)
}

const NORMAL_MOVE_SELECTOR: &str = "#section-collapsible-2 > h3 > span.mw-headline > big > span";
const NORMAL_DATA_SELECTOR: &str = "#section-collapsible-2 > div.attack-container > div.attack-info > table.moveTable > tbody";
fn append_normals(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let mut moves = select_parse(character, NORMAL_MOVE_SELECTOR, NORMAL_DATA_SELECTOR, document)?;
    character.moves.append(&mut moves);
    Ok(())
}

const SPECIAL_MOVE_SELECTOR: &str = "#section-collapsible-4 > h3 > span.mw-headline > big";
const SPECIAL_DATA_SELECTOR: &str = "#section-collapsible-4 > div.attack-container > div.attack-info > table.moveTable > tbody";
fn append_specials(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let mut moves = select_parse(character, SPECIAL_MOVE_SELECTOR, SPECIAL_DATA_SELECTOR, document)?;
    character.moves.append(&mut moves);
    Ok(())
}

const OVERDRIVE_MOVE_SELECTOR: &str = "#section-collapsible-5 > h3 > span.mw-headline > big";
const OVERDRIVE_DATA_SELECTOR: &str = "#section-collapsible-5 > div.attack-container > div.attack-info > table.moveTable > tbody";
fn append_overdrives(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let mut moves = select_parse(character, OVERDRIVE_MOVE_SELECTOR, OVERDRIVE_DATA_SELECTOR, document)?;
    character.moves.append(&mut moves);
    Ok(())
}

fn select_parse<'a>(character: &Character, move_selector: &'a str, data_selector: &'a str, document: &Html) -> Result<Vec<Move>, Box<dyn Error + 'a>> {
    let move_selector = scraper::Selector::parse(move_selector)?;
    let data_selector = scraper::Selector::parse(data_selector)?;

    let move_iter = document.select(&move_selector);
    let data_select = document.select(&data_selector);
    let zipped = move_iter.zip(data_select);

    let mut moves: Vec<Move> = vec![];
    for (move_ele, data_ele) in zipped {
        let name = move_ele.inner_html();
        let name = name.trim();
        //println!("{}", name);
        for resolver in MOVE_IMPORT_RESOLVERS {
            let res = resolver(character, name, data_ele);
            if let Some(mut moves_res) = res {
                moves.append(&mut moves_res);
                break;
            }
        }
    }
    Ok(moves)
}

#[tokio::test]
async fn test() {
    let _ = load().await;
}