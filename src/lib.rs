use std::error::Error;
use scraper::Html;

#[derive(Debug)]
pub struct Character {
    pub dustloop: String,
    pub moves: Vec<Move>
}

impl Character {
    fn new(dustloop: &str) -> Character {
        Character { dustloop: String::from(dustloop), moves: vec![] }
    }
}

#[derive(Debug)]
pub struct Move {
    pub name: String,
    pub guard: String,
    pub damage: String,
    pub startup: String,
    pub active: String,
    pub recovery: String,
    pub onblock: String,
    pub invuln: String,
}

pub async fn load() -> Result<Vec<Character>, Box<dyn Error>> {
    let mut characters: Vec<Character> = vec![
        Character::new("https://www.dustloop.com/w/GGST/Testament"),
        Character::new("https://www.dustloop.com/w/GGST/Jack-O"),
        Character::new("https://www.dustloop.com/w/GGST/Nagoriyuki"),
        Character::new("https://www.dustloop.com/w/GGST/Millia_Rage"),
        Character::new("https://www.dustloop.com/w/GGST/Chipp"),
        Character::new("https://www.dustloop.com/w/GGST/Sol_Badguy"),
        Character::new("https://www.dustloop.com/w/GGST/Ky_Kiske"),
        Character::new("https://www.dustloop.com/w/GGST/May"),
        Character::new("https://www.dustloop.com/w/GGST/Zato-1"),
        Character::new("https://www.dustloop.com/w/GGST/I-No"),
        Character::new("https://www.dustloop.com/w/GGST/Happy_Chaos"),
        Character::new("https://www.dustloop.com/w/GGST/Sin_Kiske"),
        Character::new("https://www.dustloop.com/w/GGST/Baiken"),
        Character::new("https://www.dustloop.com/w/GGST/Anji_Mito"),
        Character::new("https://www.dustloop.com/w/GGST/Leo_Whitefang"),
        Character::new("https://www.dustloop.com/w/GGST/Faust"),
        Character::new("https://www.dustloop.com/w/GGST/Axl_low"),
        Character::new("https://www.dustloop.com/w/GGST/Potemkin"),
        Character::new("https://www.dustloop.com/w/GGST/Ramlethal_Valentine"),
        Character::new("https://www.dustloop.com/w/GGST/Giovanna"),
        Character::new("https://www.dustloop.com/w/GGST/Goldlewis_Dickinson"),
        Character::new("https://www.dustloop.com/w/GGST/Bridget"),
    ];

    for character in characters.iter_mut() {
        let res = reqwest::get(character.dustloop.as_str()).await?.text().await?;
        let document = scraper::Html::parse_document(&res);
        append_normals(character, &document)?;
        append_specials(character, &document)?;
        append_overdrives(character, &document)?;
    }

    Ok(characters)
}

fn append_normals(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let move_selector = scraper::Selector::parse("#section-collapsible-2 > h3 > span.mw-headline > big > span")?;
    let data_selector = scraper::Selector::parse("#section-collapsible-2 > div.attack-container > div.attack-info > table.moveTable > tbody > tr:nth-child(2)")?;
    let data_val_selector = scraper::Selector::parse("td")?;

    let move_iter = document.select(&move_selector);
    let data_select = document.select(&data_selector);
    let zipped = move_iter.zip(data_select);

    let mut moves: Vec<Move> = vec![];
    for (move_ele, data_ele) in zipped {
        let mut v = data_ele.select(&data_val_selector);
        let damage = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let guard = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let startup = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let active = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let recovery = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let onblock = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let invuln = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let name = move_ele.inner_html();
        let m = Move {
            name,
            guard,
            damage,
            startup,
            active,
            recovery,
            onblock,
            invuln,
        };
        moves.push(m);
    }
    character.moves.append(&mut moves);

    Ok(())
}

fn append_specials(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let move_selector = scraper::Selector::parse("#section-collapsible-4 > h3 > span.mw-headline > big")?;
    let data_selector = scraper::Selector::parse("#section-collapsible-4 > div.attack-container > div.attack-info > table.moveTable > tbody > tr:nth-child(2)")?;
    let data_val_selector = scraper::Selector::parse("td")?;

    let move_iter = document.select(&move_selector);
    let data_select = document.select(&data_selector);
    let zipped = move_iter.zip(data_select);

    let mut moves: Vec<Move> = vec![];
    for (move_ele, data_ele) in zipped {
        let mut v = data_ele.select(&data_val_selector);
        let damage = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let guard = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let startup = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let active = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let recovery = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let onblock = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let invuln = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let name = move_ele.inner_html();
        let m = Move {
            name,
            guard,
            damage,
            startup,
            active,
            recovery,
            onblock,
            invuln,
        };
        moves.push(m);
    }
    character.moves.append(&mut moves);
    Ok(())
}

fn append_overdrives(character: &mut Character, document: &Html) -> Result<(), Box<dyn Error>> {
    let move_selector = scraper::Selector::parse("#section-collapsible-5 > h3 > span.mw-headline > big")?;
    let data_selector = scraper::Selector::parse("#section-collapsible-5 > div.attack-container > div.attack-info > table.moveTable > tbody > tr:nth-child(2)")?;
    let data_val_selector = scraper::Selector::parse("td")?;

    let move_iter = document.select(&move_selector);
    let data_select = document.select(&data_selector);
    let zipped = move_iter.zip(data_select);

    let mut moves: Vec<Move> = vec![];
    for (move_ele, data_ele) in zipped {
        let mut v = data_ele.select(&data_val_selector);
        let damage = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let guard = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let startup = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let active = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let recovery = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let onblock = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let invuln = v.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let name = move_ele.inner_html();
        let m = Move {
            name,
            guard,
            damage,
            startup,
            active,
            recovery,
            onblock,
            invuln,
        };
        moves.push(m);
    }
    character.moves.append(&mut moves);
    Ok(())
}