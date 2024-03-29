use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use scraper::{Selector, ElementRef, element_ref::Select};

use crate::{Move, CharacterId, Character};

fn get_image_url_matcher() -> &'static Regex {
    static IMAGE_URL_MATCHER: OnceLock<Regex> = OnceLock::new();
    IMAGE_URL_MATCHER.get_or_init(|| Regex::new(r"(?i)src=&quot;(\S*(hitbox|HB)\S*\.png)").unwrap())
}

fn get_row_selector() -> &'static Selector {
    static ROW_SELECTOR: OnceLock<Selector> = OnceLock::new();
    ROW_SELECTOR.get_or_init(|| Selector::parse("tbody > tr").unwrap())
}

fn get_element_selector() -> &'static Selector {
    static ELEMENT_SELECTOR: OnceLock<Selector> = OnceLock::new();
    ELEMENT_SELECTOR.get_or_init(|| Selector::parse("td").unwrap())
}

const SECTIONS: [&str; 3] = ["#section-collapsible-3 > table", "#section-collapsible-4 > table", "#section-collapsible-5 > table"];
pub async fn get_moves(character: &Character) -> Vec<Move> {
    let mut moves: Vec<Move> = vec![];

    let res = reqwest::get(character.frame_data_url.as_str()).await;
    let Ok(res) = res else {
        println!("Error making request for {:?}", character.id);
        return moves;
    };
    let Ok(res) = res.text().await else {
        println!("Error making request for {:?}", character.id);
        return moves;
    };

    let document = scraper::Html::parse_document(&res);
    for (ind, ele) in SECTIONS.iter().enumerate() {
        let parse = Selector::parse(ele);
        let Ok(section_selector) = parse else {
            println!("Error making selector for {:?}: {}", character.id, parse.unwrap_err());
            continue;
        };
        let select = document.select(&section_selector).next();
        let Some(section_element) = select else {
            println!("Could not select section {} for {:?}", ele, character.id);
            continue;
        };
        let mut moves_found = load_section(character.id, section_element, ind != 0);
        moves.append(&mut moves_found);
    }
    moves
}

fn load_section(character: CharacterId, section: ElementRef, named: bool) -> Vec<Move> {
    let select = section.select(get_row_selector());
    let mut moves: Vec<Move> = vec![];
    for row_raw in select {
        // the hitbox image urls are in the html element itself (hidden details control)
        let element_html = row_raw.html();
        let mut hitboxes = vec![];
        // get all hitboxes for this move
        for (_, [url, _]) in get_image_url_matcher().captures_iter(&element_html).map(|c| c.extract()) {
            // the first (0th) capture is always the entire match, I just want the first group as designed in the regex
            hitboxes.push(format!("https://www.dustloop.com{}", url));
        }
        let row_elements = row_raw.select(get_element_selector());
        let mut move_found = parse_row(row_elements, &character, named);
        move_found.hitboxes = hitboxes;
        moves.push(move_found);
    }
    moves
}

// an unfortunate special boolean I have to include because the rows are not the same for Normals, Specials etc
fn parse_row(row: Select, character_id: &CharacterId, named: bool) -> Move {
    let mut row = row.map(|v| v.inner_html());
    let _ = row.next(); // skip one, first is details control on table
    let input = row.next().unwrap_or(String::from("")).trim().to_string();
    let name = if named {
        row.next().unwrap_or(String::from("")).trim().to_string()
    } else {
        input.clone()
    };
    let damage = row.next().unwrap_or(String::from("")).trim().to_string();
    let guard = row.next().unwrap_or(String::from("")).trim().to_string();
    let startup = row.next().unwrap_or(String::from("")).trim().to_string();
    let active = row.next().unwrap_or(String::from("")).trim().to_string();
    let recovery = row.next().unwrap_or(String::from("")).trim().to_string();
    let onblock = row.next().unwrap_or(String::from("")).trim().to_string();
    let onhit = row.next().unwrap_or(String::from("")).trim().to_string();
    let level = row.next().unwrap_or(String::from("")).trim().to_string();
    let counterhit_type = row.next().unwrap_or(String::from("")).trim().to_string();
    let invuln = row.next().unwrap_or(String::from("")).trim().to_string();
    let proration = row.next().unwrap_or(String::from("")).trim().to_string();
    let risc_gain = row.next().unwrap_or(String::from("")).trim().to_string();
    let risc_loss = row.next().unwrap_or(String::from("")).trim().to_string();
    let regex = get_regex_binding(character_id, input.clone(), name.clone())
        .unwrap_or(default_normal_resolver(input.clone()));
    Move {
        regex,
        input,
        name,
        damage,
        guard,
        startup,
        active,
        recovery,
        onblock,
        onhit,
        level,
        counterhit_type,
        invuln,
        proration,
        risc_gain,
        risc_loss,
        hitboxes: vec![],
    }
}

fn default_normal_resolver(original: impl Into<String>) -> Regex {
    let original = regex::escape(original.into().as_str());
    let original = original.replace('.', ".?"); // the dot is already there, so it is escaped in previous line
    let input = format!(r"(?i)^({})$", original);
    Regex::new(&input).unwrap()
}

fn get_loaded_move_bindings() -> &'static HashMap<CharacterId, Vec<(Regex, String)>> {
    static BINDINGS: OnceLock<HashMap<CharacterId, Vec<(Regex, String)>>> = OnceLock::new();
    BINDINGS.get_or_init(get_all_bindings)
}

fn get_regex_binding(character_id: &CharacterId, input: String, name: String) -> Option<Regex> {
    get_loaded_move_bindings().get(character_id).and_then(|v| {
        for ele in v {
            let regex = &ele.0;
            let bind = &ele.1;
            if bind.eq_ignore_ascii_case(input.as_str()) || bind.eq_ignore_ascii_case(name.as_str()) {
                return Some(regex.clone());
            }
        }
        None
    })
}

fn get_all_bindings() -> HashMap<CharacterId, Vec<(Regex, String)>> {
    let mut total: HashMap<CharacterId, Vec<(Regex, String)>> = HashMap::default();
    for char_id in CharacterId::ALL {
        let mut moves: Vec<(Regex, String)> = vec![];
        for (k, v) in get_bindings(char_id) {
            moves.push((Regex::new(k.as_str()).unwrap(), v));
        }
        total.insert(char_id, moves);
    }
    total
}

fn get_bindings(character_id: CharacterId) -> Vec<(String, String)> {
    match character_id {
        CharacterId::TESTAMENT => {
            vec![
                (r"(?i)(^(j.?)?(236HS?|hs?\s*(grave)?\s*reaper)$)", "236H"),
                (r"(?i)(236\{hs?\}|((med)\s*(j.?)?(236HS?|hs?\s*(grave)?\s*reaper)))", "236{H}"),
                (r"(?i)(236\[hs?\]|((charge|heavy|hard)\s*(j.?)?(236HS?|hs?\s*(grave)?\s*reaper)))", "236[H]"),
                (r"(?i)(^(j.?)?(236S|s\s*(grave)?\s*reaper)$)", "236H"),
                (r"(?i)(236\{s\}|((med)\s*(j.?)?(236S|s\s*(grave)?\s*reaper)))", "236{H}"),
                (r"(?i)(236\[s\]|((charge|heavy|hard)\s*(j.?)?(236S|s\s*(grave)?\s*reaper)))", "236[H]"),
                (r"(?i)((crow|unholy|diver))", "Unholy Diver"),
                (r"(?i)((tele))", "Possession"),
                (r"(?i)((214S|s\s*arbiter))", "214S"),
                (r"(?i)((214H|hs?\s*arbiter))", "214H"),
                (r"(?i)((236P236P|nostrovia|succub))", "Nostrovia"),
                (r"(?i)((236P236K|calamity\s*one|reversal))", "Calamity One"),
            ]
        },
        CharacterId::JACKO => {
            vec![
                (r"(?i)(2D)", "Sweep"),

                (r"(?i)((236K(&|\*)))", "Launched Servant"),
                (r"(?i)((236K|shoot|kick))", "236K"),
                (r"(?i)((236P|summon|pull))", "236P"),
                (r"(?i)((236\[P\]|((pull|summon)\s*hold)))", "236[P]"),
                (r"(?i)((pick|2P))", "2P"),
                (r"(?i)((throw|6(P|K|S|HS?|D)))", "Throw Servant"),
                (r"(?i)((drop|release))", "Release Servant"),
                (r"(?i)((unsummon|recover|recall|214P))", "Recover Servant"),
                (r"(?i)((attack|214K))", "Attack Command"),
                (r"(?i)((defend|block|214S))", "Defend Command"),
                (r"(?i)((countdown|bomb|214HS?))", "Countdown"),
                (r"(?i)((632146P|F.?E.?D|forever\s*elysion\s*driver))", "Forever Elysion Driver"),
                (r"(?i)((s\s*cheer|236236S))", "Cheer Servant On S"),
                (r"(?i)((hs?\s*cheer|236236HS?))", "Cheer Servant On H"),
            ]
        },
        CharacterId::NAGORIYUKI => {
            vec![
                (r"(?i)(^(f.?S(\s*(level|lv|lvl)?\s*1)?)\s?$)", "f.S Level 1"),
                (r"(?i)((f.?S\s*(level|lv|lvl)?\s*2))", "f.S Level 2"),
                (r"(?i)((f.?S\s*(level|lv|lvl)?\s*3))", "f.S Level 3"),
                (r"(?i)((f.?S\s*(level|lv|lvl)?\s*BR))", "f.S Level BR"),

                (r"(?i)(^(f.?SS(\s*(level|lv|lvl)?\s*1)?)\s?$)", "f.SS Level 1"),
                (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*2))", "f.SS Level 2"),
                (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*3))", "f.SS Level 3"),
                (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*BR))", "f.SS Level BR"),

                (r"(?i)(^(f.?SSS(\s*(level|lv|lvl)?\s*1)?)\s?$)", "f.SSS Level 1"),
                (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*2))", "f.SSS Level 2"),
                (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*3))", "f.SSS Level 3"),
                (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*BR))", "f.SSS Level BR"),

                (r"(?i)(^(5?H(\s*(level|lv|lvl)?\s*1)?)\s?$)", "5H Level 1"),
                (r"(?i)((5?H\s*(level|lv|lvl)?\s*2))", "5H Level 2"),
                (r"(?i)((5?H\s*(level|lv|lvl)?\s*3))", "5H Level 3"),
                (r"(?i)((5?H\s*(level|lv|lvl)?\s*BR))", "5H Level BR"),

                (r"(?i)(^(2S(\s*(level|lv|lvl)?\s*1)?)\s?$)", "2S Level 1"),
                (r"(?i)((2S\s*(level|lv|lvl)?\s*2))", "2S Level 2"),
                (r"(?i)((2S\s*(level|lv|lvl)?\s*3))", "2S Level 3"),
                (r"(?i)((2S\s*(level|lv|lvl)?\s*BR))", "2S Level BR"),

                (r"(?i)(^(2H(\s*(level|lv|lvl)?\s*1)?)\s?$)", "2H Level 1"),
                (r"(?i)((2H\s*(level|lv|lvl)?\s*2))", "2H Level 2"),
                (r"(?i)((2H\s*(level|lv|lvl)?\s*3))", "2H Level 3"),
                (r"(?i)((2H\s*(level|lv|lvl)?\s*BR))", "2H Level BR"),

                (r"(?i)(^(6H(\s*(level|lv|lvl)?\s*1)?)\s?$)", "6H Level 1"),
                (r"(?i)((6H\s*(level|lv|lvl)?\s*2))", "6H Level 2"),
                (r"(?i)((6H\s*(level|lv|lvl)?\s*3))", "6H Level 3"),
                (r"(?i)((6H\s*(level|lv|lvl)?\s*BR))", "6H Level BR"),

                (r"(?i)(^(j.?S(\s*(level|lv|lvl)?\s*1)?)\s?$)", "j.S Level 1"),
                (r"(?i)((j.?S\s*(level|lv|lvl)?\s*2))", "j.S Level 2"),
                (r"(?i)((j.?S\s*(level|lv|lvl)?\s*3))", "j.S Level 3"),
                (r"(?i)((j.?S\s*(level|lv|lvl)?\s*BR))", "j.S Level BR"),

                (r"(?i)(^(j.?H(\s*(level|lv|lvl)?\s*1)?)\s?$)", "j.H Level 1"),
                (r"(?i)((j.?H\s*(level|lv|lvl)?\s*2))", "j.H Level 2"),
                (r"(?i)((j.?H\s*(level|lv|lvl)?\s*3))", "j.H Level 3"),
                (r"(?i)((j.?H\s*(level|lv|lvl)?\s*BR))", "j.H Level BR"),

                (r"(?i)(^(j.?D(\s*(level|lv|lvl)?\s*1)?)\s?$)", "j.D Level 1"),
                (r"(?i)((j.?D\s*(level|lv|lvl)?\s*2))", "j.D Level 2"),
                (r"(?i)((j.?D\s*(level|lv|lvl)?\s*3))", "j.D Level 3"),
                (r"(?i)((j.?D\s*(level|lv|lvl)?\s*BR))", "j.D Level BR"),

                (r"(?i)((214K|fukyo\s*back))", "214K"), // Not sure anymore if this needs to come before fukyo forward
                (r"(?i)((236K|(fukyo(\s*forward))|fukyo$))", "236K"),

                (r"(?i)((236S|clone|zarameyuki))", "Zarameyuki"),

                (r"(?i)((214HS?|beyblade|kamuriyuki))", "Kamuriyuki"),

                (r"(?i)(^(623HS?|shizuriyuki\s?|dp\s?)$)", "623H"),
                (r"(?i)((623HS?HS?|((shizuriyuki|dp)\s*(follow|HS?|2))))", "623HH"),

                (r"(?i)((623P|bite|command|blood))", "Bloodsucking Universe"),

                (r"(?i)((632146S|wasureyuki|reversal))", "Wasureyuki"),

                (r"(?i)((632146H|zansetsu|reversal))", "Zansetsu"),
            ]
        },
        CharacterId::MILLIA => {
            vec![
                (r"(?i)((S\s*disk|236S))", "236S"),
                (r"(?i)((HS?\s*disk|236H))", "236H"),
                (r"(?i)((moon|(j.?)?236P))", "Bad Moon"),
                (r"(?i)((214P|hair|car))", "Iron Savior"),
                (r"(?i)((turbo|fall|(j.?)?236K))", "Turbo Fall"),
                (r"(?i)((214K|mirazh))", "Mirazh"),
                (r"(?i)((lust|shaker|214S))", "Lust Shaker"),
                (r"(?i)((kapel|j.?236HS?))", "Kapel"),
                (r"(?i)((632146HS?|winger|reversal))", "Winger"),
                (r"(?i)((236236S|septum))", "Septum Voices"),
            ]
        },
        CharacterId::CHIPP => {
            vec![
                (r"(?i)((236P|p\s*alpha))", "236P"),
                (r"(?i)((j.?236P|((air|j.?)\s*p\s*alpha)))", "j.236P"),
                (r"(?i)((236K|k\s*alpha))", "236K"),
                (r"(?i)((j.?236K|((air|j.?)\s*k\s*alpha)))", "j.236K"),
                (r"(?i)((623S|dp|beta))", "623S"),
                (r"(?i)((j.?(623P|dp|beta)))", "j.623S"),
                (r"(?i)((236HS?|gamma|clone))", "Gamma Blade"),
                (r"(?i)(^(236S|rekka(\s*1)?|resshou)\s?$)", "Resshou"),
                (r"(?i)((rekka\s*2|rokusai))", "Rokusai"),
                (r"(?i)((senshuu?|rekka\s*3))", "Senshuu"),
                (r"(?i)((63214S|command|grab))", "Genrou Zan"),
                (r"(?i)((j.?214P|shuriken))", "Shuriken"),
                (r"(?i)((632146HS?|zansei))", "Zansei Rouga"),
                (r"(?i)((236236P|banki))", "Banki Messai"),
            ]
        },
        CharacterId::SOL => {
            vec![
                (r"(?i)((feint|faint|214P))", "Gun Flame (Feint)"),
                (r"(?i)(^(gun\s?flame|236P)\s?$)", "Gun Flame"),
                (r"(?i)((svv|623S))", "623S"),
                (r"(?i)((hvv|623HS?|dp))", "623H"),
                (r"(?i)(^(j.?\s*(s?vv|623S))$)", "j.633H"),
                (r"(?i)((j.?\s*(hvv|623HS?|dp)))", "j.633H"),
                (r"(?i)((revolver|br|236K)$)", "236K"),
                (r"(?i)((236KK))", "236KK"),
                (r"(?i)((j.?\s*(revolver|br|236K)))", "j.236K"),
                (r"(?i)((j.?\s*(236KK)))", "j.236KK"),
                (r"(?i)((bringer|bb|236K))", "214K"),
                (r"(?i)((j.?\s*(bringer|bb|236K)))", "j.214K"),
                (r"(?i)((623K|wild|throw|grab))", "Wild Throw"),
                (r"(?i)((nrv|214S|vortex))", "Night Raid Vortex"),
                (r"(?i)((fafnir|41236HS?))", "Fafnir"),
                (r"(?i)((632146HS?|tyrant|rave))", "Tyrant Rave"),
                (r"(?i)((hmc|mob|cemetary|214214HS?))", "Heavy Mob Cemetery"),
            ]
        },
        CharacterId::KY => {
            vec![
                (r"(?i)(^(edge|236S)\s?$)", "236S"),
                (r"(?i)(DI\s*(edge|236S))", "DI 236S"),
                (r"(?i)(^(charge|236HS?)\s?$)", "236H"),
                (r"(?i)(DI\s*(charge|236HS?))", "DI 236H"),
                (r"(?i)(j.?\s*(arial|236S))", "j.236S"),
                (r"(?i)(j.?\s*(arial|236HS?))", "DI j.236H"),
                (r"(?i)(^(dip|236K)\s?$)", "236K"),
                (r"(?i)(DI\s*(dip|236K))", "DI 236K"),
                (r"(?i)(^(flip|foudre|214K)\s?$)", "214K"),
                (r"(?i)(DI\s*(flip|foudre|214K))", "DI 214K"),
                (r"(?i)(^(623S)\s?$)", "623S"),
                (r"(?i)(DI\s*(623S))", "DI 623S"),
                (r"(?i)(^(623HS?|dp|vapor|thrust)\s?$)", "623H"),
                (r"(?i)(DI\s*(623HS?|dp|vapor|thrust))", "DI 623H"),
                (r"(?i)(^(dire|eclat|214S)\s?$)", "214S"),
                (r"(?i)(DI\s*(dire|eclat|214S))", "DI 214S"),
                (r"(?i)(^(rtl|ride|lightning|632146HS?)\s?$)", "632146H"),
                (r"(?i)(DI\s*(rtl|ride|lightning|632146HS?))", "DI 632146H"),
                (r"(?i)(^(sacred|236236P)\s?$)", "236236P"),
                (r"(?i)(DI\s*(sacred|236236P))", "DI 236236P"),
                (r"(?i)(^(di|dragon|install|214214HS?)\s?$)", "Dragon Install"),
            ]
        },
        CharacterId::MAY => {
            vec![
                (r"(?i)(^(\[4\]6S|s?\s*dolphin)\s?$)", "[4]6S"),
                (r"(?i)(^(\[4\]6HS?|hs?\s*dolphin)\s?$)", "[4]6H"),
                (r"(?i)((\[2\]8S|(up|vertical)\s*s?\s*dolphin))", "[2]8S"),
                (r"(?i)((\[2\]8HS?|(up|vertical)\s*hs?\s*dolphin))", "[2]8H"),
                (r"(?i)((ok|overhead|kiss|623K|command|grab))", "Overhead Kiss"),
                (r"(?i)((214P))", "214P"),
                (r"(?i)((beach|ball|214K))", "214K"),
                (r"(?i)((yamada|236236S))", "Great Yamada Attack"),
                (r"(?i)((orca|632146HS?))", "The Wonderful and Dynamic Goshogawara"),
            ]
        },
        CharacterId::ZATO => {
            vec![
                (r"(?i)((summon|214HS?))", "Summon Eddie"),
                (r"(?i)((unsummon))", "Unsummon Eddie"),

                (r"(?i)((pierce|236P))", "236P"),
                (r"(?i)((\]P\[|-P-))", "]P["),

                (r"(?i)((that's a lot|drills|236K))", "236K"),
                (r"(?i)((\]K\[|-K-))", "]K["),

                (r"(?i)((leap|frog|236S))", "236S"),
                (r"(?i)((\]S\[|-S-))", "]S["),

                (r"(?i)((oppose|236HS?))", "236H"),
                (r"(?i)((\]HS?\[|-HS?-))", "]H["),

                (r"(?i)((invite|hell|22HS?))", "Invite Hell"),
                (r"(?i)((btl|break|law|214K))", "Break The Law"),
                (r"(?i)((damned|fang|command|grab|623S))", "Damned Fang"),
                (r"(?i)((214S|shade|drunk))", "Drunkard Shade"),
                (r"(?i)((632146HS?|amongus|amor))", "Amorphous"),
                (r"(?i)((sun|void|632146S|sword|excalibur))", "Sun Void"),
            ]
        },
        CharacterId::INO => {
            vec![
                (r"(?i)((note|anti|214P))", "214P"),
                (r"(?i)((j.?\s*(note|anti|214P)))", "j.214P"),
                (r"(?i)((s\s*stroke))", "236S"),
                (r"(?i)((hs?\s*stroke))", "236H"),
                (r"(?i)((j.?\s*236K))", "j.236K"),
                (r"(?i)((j.?\s*236S))", "j.236S"),
                (r"(?i)((j.?\s*236HS?))", "j.236H"),
                (r"(?i)((love|chemical|214K))", "214K"),
                (r"(?i)((j.?\s*(love|chemical|214K)))", "j.214K"),
                (r"(?i)((mega|632146HS?))", "Megalomania"),
                (r"(?i)((ultimate|fort|632146S))", "632146S"),
                (r"(?i)((j.?\s*(ultimate|fort|632146S)))", "j.632146S"),
            ]
        },
        CharacterId::HAPPYCHAOS => {
            vec![
                (r"(?i)(^(h)\s?$)", "H"),
                (r"(?i)(^(\]H\[|fire|shot)\s?$)", "236S H"),
                (r"(?i)((atr|236S|flip))", "236S"),
                (r"(?i)(^(steady|aim|sa|214S|steady\s?aim)\s?$)", "Steady Aim"),
                (r"(?i)((steady|aim|sa|214S|steady\s?aim)\s*(shot|fire|h))", "214S H"),
                (r"(?i)((cancel|2H|stow))", "236S 2H"),
                (r"(?i)(((steady|aim|sa)\s*(cancel|stow)))", "214S 214S"),
                (r"(?i)((reload|22P))", "Reload"),
                (r"(?i)((focus|214P))", "Focus"),
                (r"(?i)((curse|ball|236P))", "Curse"),
                (r"(?i)((clone|236K))", "Scapegoat"),
                (r"(?i)((roll|214K))", "Roll"),
                (r"(?i)((dem|deus|ex|machina|632146S))", "Deus Ex Machina"),
                (r"(?i)((super\s*focus|214214P))", "Super Focus"),
            ]
        },
        CharacterId::SIN => {
            vec![
                (r"(?i)(^(beak|driver|236HS?)\s?$)", "Beak Driver"),
                (r"(?i)(((beak|driver|236HS?)\s*(~?H|follow)))", "Beak Driver Follow-up"),
                (r"(?i)(^(hawk|baker|623S|dp)\s?$)", "Hawk Baker"),
                (r"(?i)(((hawk|baker|623S|dp)\s*(~?S|follow)))", "Hawk Baker Follow-up"),
                (r"(?i)(^(elk|hunt|236K)\s?$)", "236K"),
                (r"(?i)(((elk|hunt|236K)\s*(~?K|follow)))", "236K~K"),
                (r"(?i)(^(hoof|stomp|214S)\s?$)", "214S"),
                (r"(?i)(((hoof|stomp|214S)\s*(~?S|follow)))", "214S~S"),
                (r"(?i)((gazelle|dash|step))", "Gazelle Step"),
                (r"(?i)((food|eat|grow|63214P))", "Still Growing"),
                (r"(?i)(^(rtl|ride|lightning|632146HS?)\s?$)", "632146H"),
                (r"(?i)(((rtl|ride|lightning|632146HS?)\s*(~?HS?|follow)))", "632146HH"),
                (r"(?i)(^(barrel|tyrant|236236P)\s?$)", "236236P"),
                (r"(?i)(((barrel|tyrant|236236P)\s*(~?\[?P\]?|follow)))", "236236P~]P["),
            ]
        },
        CharacterId::BAIKEN => {
            vec![
                (r"(?i)((tatami|mat|gaeshi|236K))", "236K"),
                (r"(?i)((j.?\s*(tatami|mat|gaeshi|236K)))", "j.236K"),
                (r"(?i)((tether|s\s*kabari|41236S))", "41236S"),
                (r"(?i)((hs?\s*kabari|41236HS?))", "41236H"),
                (r"(?i)((^(hs?\s*kabari|41236HS?)\s?$\s*(follow|~?HS?)))", "41236H~H"),
                (r"(?i)((yozansen|youzansen|tk|236S))", "Youzansen"),
                (r"(?i)((parry|Hiiragi|236P))", "Hiiragi"),
                (r"(?i)((236236S|watashi|tsurane|sanzu))", "Tsurane Sanzu-watashi"),
                (r"(?i)((gun|kenjyu|214214P))", "214214P"),
                (r"(?i)((j.?\s*(gun|kenjyu|214214P)))", "j.214214P"),
            ]
        },
        CharacterId::ANJI => {
            vec![
                (r"(?i)((butter|shitsu|fire|236P))", "Shitsu"),
                (r"(?i)((parry|suigetsu|spin|236K))", "Suigetsu No Hakobi"),
                (r"(?i)(^(fuujin|fujin|236HS?)\s?$)", "Fuujin"),
                (r"(?i)(((fuujin|fujin|236HS?)\s*P))", "Shin: Ichishiki"),
                (r"(?i)(((fuujin|fujin|236HS?)\s*K))", "Issokutobi"),
                (r"(?i)(((fuujin|fujin|236HS?)\s*S))", "Nagiha"),
                (r"(?i)(((fuujin|fujin|236HS?)\s*HS?))", "Rin"),
                (r"(?i)((kou|236S))", "Kou"),
                (r"(?i)((issei|ougi|632146HS?))", "Issei Ougi: Sai"),
                (r"(?i)(kach|632146S)", "Kachoufuugetsu Kai"),
            ]
        },
        CharacterId::LEO => {
            vec![
                (r"(?i)((hyper|guard|\[HS?\]S|\[S\]HS?))", "Guard"),
                (r"(?i)((s\s*(fire|ball|grav)))", "[4]6S"),
                (r"(?i)((hs?\s*(fire|ball|grav)))", "[4]6H"),
                (r"(?i)(((s\*(dp|ein))|\[2\]8S))", "[2]8S"),
                (r"(?i)(((dp|ein)|\[2\]8HS?))", "[2]8H"),
                (r"(?i)((236S|erstes))", "Erstes Kaltes Gestöber"),
                (r"(?i)((236HS?|zwe))", "Zweites Kaltes Gestöber"),
                (r"(?i)((214S|turb))", "Turbulenz"),
                (r"(?i)((parry|kahn|schild|sheild|bt\.D))", "Kahn-Schild"),
                (r"(?i)((command|grab|dunkel|214K))", "Glänzendes Dunkel"),
                (r"(?i)((blitz|214HS?))", "Blitzschlag"),
                (r"(?i)((632146S|stahl))", "Stahlwirbel"),
                (r"(?i)((632146HS?|lei))", "Leidenschaft des Dirigenten"),
            ]
        },
        CharacterId::FAUST => {
            vec![
                (r"(?i)(^(scalpel|thrust|41236K)\s?$)", "Thrust"),
                (r"(?i)((pull|back))", "Pull Back"),
                (r"(?i)((hit|golf|club|hole|41236K\s*HS?))", "Hole in One!"),
                (r"(?i)(^(item|toss|236P|what)\s?$)", "What Could This Be?"),
                (r"(?i)((mmm|mix|236S))", "Mix Mix Mix"),
                (r"(?i)((snip|command|grab|236HS?))", "Snip Snip Snip"),
                (r"(?i)(((j.?)?love|j.?236P))", "j.236P"),
                (r"(?i)(((j.?)?love|j.?236P)\s*(afro))", "j.236P (With Afro)"),
                (r"(?i)(((p\s*(scare)?crow)|214P))", "214P"),
                (r"(?i)(((k\s*(scare)?crow)|214K))", "214K"),
                (r"(?i)(((s\s*(scare)?crow)|214S))", "214S"),
                (r"(?i)((bone|wheel|chair|reversal|632146HS?))", "Bone-crushing Excitement"),
                (r"(?i)(^(236236P|item\s*super)\s?$)", "W-W-What Could This Be?"),
                (r"(?i)(^(236236236236P)\s?$)", "W-W-W-W-W-W-W-W-W-What Could This Be?"),
            ]
        },
        CharacterId::AXL => {
            vec![
                (r"(?i)(^(rensen|rensin|\[4\]6S|flash)\s?$)", "Sickle Flash"),
                (r"(?i)(((rensen|rensin|\[4\]6S|flash)\s*(8|up)))", "Soaring Chain Strike"),
                (r"(?i)(((rensen|rensin|\[4\]6S|flash)\s*(2|down)))", "Spinning Chain Strike"),
                (r"(?i)((cherry|((rensen|rensin|\[4\]6S|flash)\s*(s|bomb))))", "Winter Cherry"),
                (r"(?i)((mantis|command|grab|41236HS?))", "Winter Mantis"),
                (r"(?i)((rain|water|216S))", "Rainwater"),
                (r"(?i)((snail|214HS?))", "214H"),
                (r"(?i)((j.?\s*(snail|214HS?)))", "j.214H"),
                (r"(?i)((bomber|j.?\s*236HS?))", "Axl Bomber"),
                (r"(?i)((whistling|tornado|wind|214K))", "Whistling Wind"),
                (r"(?i)((reversal|storm|236236HS?))", "Sickle Storm"),
                (r"(?i)(^(one|vision|time\s*stop|632146P)\s?$)", "632146P"),
                (r"(?i)(((one|vision|time\s*stop|632146P)\s*activ))", "632146P Attack"),
            ]
        },
        CharacterId::POTEMKIN => {
            vec![
                (r"(?i)(^(pb|grab|buster|360P|632146P)$)", "Potemkin Buster"),
                (r"(?i)(^(heat knuckle|knuckle|hk|623HS?)$)", "Heat Knuckle"),
                (r"(?i)(^(fmf|mf|236P|forward|mega(\s+fist)?)\s?$)", "236P"),
                (r"(?i)((^(back|b)\s*(214P|mf|mega(\s+fist)?)\s?$))", "214P"),
                (r"(?i)(((k|kara)\s*(back|b)\s*(mega|fist|214P|mf)))", "2146K~P"),
                (r"(?i)((slide|head|236S))", "Slide Head"),
                (r"(?i)(^(hammer|fall|hammer\s*fall|\[4\]6HS?|hf)\s?$)", "Hammer Fall"),
                (r"(?i)(((hammer|fall|hammer\s*fall|\[4\]6HS?|hf)\s*(break|b)))", "Hammer Fall Break"),
                (r"(?i)(^(flick|f.?d.?b.?)\s?$)", "F.D.B."),
                (r"(?i)(((flick|f.?d.?b.?)\s*charge))", "F.D.B. (Charged)"),
                (r"(?i)(((flick|f.?d.?b.?)\s*(hit|reflect)))", "Reflect Projectile"),
                (r"(?i)((garuda|214HS?))", "Garuda Impact"),
                (r"(?i)(^(heat tackle|tackle|ht|41236HS?)$)", "41236H"),
                (r"(?i)((hpb|236236S|heavenly))", "Heavenly Potemkin Buster"),
                (r"(?i)(^(giganter(\s+kai)?|632146HS?)\s?$)", "Giganter Kai"),
                (r"(?i)((giganter(\s+kai)?|632146HS?)\s*(barrier))", "Giganter Kai (Barrier)"),
            ]
        },
        CharacterId::RAMLETHAL => {
            vec![
                (r"(?i)((623P|dp|dauro))", "Dauro"),
                (r"(?i)(^(rekka|214P|erar)\s?$)", "214P"),
                (r"(?i)(((rekka|214P|erar)\s*2))", "214P 214P"),
                (r"(?i)(((rekka|214P|erar)\s*3))", "214P 214P 214P"),
                (r"(?i)((flip|214K|slido))", "214K"),
                (r"(?i)((j.?\s*(flip|214K|slido)))", "j.214K"),
                (r"(?i)((sword|throw|toss|bajoneto|236S))", "236S"),
                (r"(?i)(((hs?)\s*(sword|throw|toss|bajoneto|236HS?)))", "236H"),
                (r"(?i)((ordono|agress?a|(j.?\s*214S)))", "Agressa Ordono"),
                (r"(?i)((wind|wiper|sab|214HS?))", "Sabrobato"),
                (r"(?i)((ondo|rock|236K))", "Ondo"),
                (r"(?i)((calvados|63214HS?))", "Calvados"),
                (r"(?i)((mortobato|reversal|236236S))", "Mortobato"),
            ]
        },
        CharacterId::GIO => {
            vec![
                (r"(?i)((kick|214K|sep))", "Sepultura"),
                (r"(?i)((drill|dog|236K|tro))", "Trovao"),
                (r"(?i)((623S|dp|nascente))", "Sol Nascente"),
                (r"(?i)((214S|sol|poente))", "214S"),
                (r"(?i)((j.?\s*(214S|sol|poente)))", "j.214S"),
                (r"(?i)((spin|reversal|63214HS?))", "Ventania"),
                (r"(?i)((temp|air|(j.?\s*236236HS?)))", "Tempestade"),
            ]
        },
        CharacterId::GOLDLEWIS => {
            vec![
                (r"(?i)((41?23?6(HS?)?))", "41236H"),
                (r"(?i)((j.?\s*(41?23?6(HS?)?)))", "j.41236H"),

                (r"(?i)((63?21?4(HS?)?))", "63214H"),
                (r"(?i)((j.?\s*(63?21?4(HS?)?)))", "j.63214H"),

                (r"(?i)((23?69?8(HS?)?))", "23698H"),
                (r"(?i)((j.?\s*(23?69?8(HS?)?)))", "j.23698H"),

                (r"(?i)((21?47?8(HS?)?))", "21478H"),
                (r"(?i)((j.?\s*(21?47?8(HS?)?)))", "j.21478H"),

                (r"(?i)((69?87?4(HS?)?))", "69874H"),
                (r"(?i)((j.?\s*(684(HS?)?)))", "j.69874H"),

                (r"(?i)((47?89?6(HS?)?))", "47896H"),
                (r"(?i)((j.?\s*(47?89?6(HS?)?)))", "j.47896H"),

                (r"(?i)((87?41?2(HS?)?))", "87412H"),
                (r"(?i)((j.?\s*(87?41?2(HS?)?)))", "j.87412H"),

                (r"(?i)((89?63?2(HS?)?))", "89632H"),
                (r"(?i)((j.?\s*(89?63?2(HS?)?)))", "j.89632H"),

                (r"(?i)(^(drone|214S)\s?$)", "214S Level 1"),
                (r"(?i)(((drone|214S)\s*(level|lv|lvl)?\s*2))", "214S Level 2"),
                (r"(?i)(((drone|214S)\s*(level|lv|lvl)?\s*3))", "214S Level 3"),

                (r"(?i)(^(gun|mini|skyfish|236S)\s?$)", "236S Level 1"),
                (r"(?i)(((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*2))", "236S Level 2"),
                (r"(?i)(((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*3))", "236S Level 3"),

                (r"(?i)((dwts|system|reversal|360P?|63214P))", "632146P"),
                (r"(?i)((720P?))", "720P"),
                (r"(?i)((1080P?))", "1080P"),

                (r"(?i)(^(beam|burn|236236K)\s?$)", "236236K Level 1"),
                (r"(?i)(((beam|burn|236236K)\s*(level|lv|lvl)?\s*2))", "236236K Level 2"),
                (r"(?i)(((beam|burn|236236K)\s*(level|lv|lvl)?\s*3))", "236236K Level 3"),
            ]
        },
        CharacterId::BRIDGET => {
            vec![
                (r"(?i)(((236(S|(HS?))|yoyo|toss)))", "Stop and Dash (Hit on send)"),
                (r"(?i)((roll|spin|214K))", "Rolling Movement"),
                (r"(?i)((dp|starship|623P))", "Starship"),
                (r"(?i)(^(car|kick|start|heart|236K)$)", "Kick Start My Heart"),
                (r"(?i)((brake|((car|kick|start|heart|236K)\s*P)))", "Brake"),
                (r"(?i)((shoot|((car|kick|start|heart|236K)\s*K)))", "Shoot"),
                (r"(?i)((dive|(j.?\s*236K)))", "Roger Dive"),
                (r"(?i)((command|grab|rock|baby|63214P))", "Rock the Baby"),
                (r"(?i)((loop|632146S))", "Loop the Loop"),
                (r"(?i)((motor|killing|632146HS?|return))", "Return of the Killing Machine"),
            ]
        },
        CharacterId::BEDMAN => {
            vec![
                /* TODO! */
            ]
        },
        CharacterId::ASUKA => {
            vec![
                /* TODO! */
            ]
        },
        CharacterId::JOHNNY => {
            vec![
                /* TODO! */
            ]
        },
        CharacterId::ELPHELT => {
            vec![
                /* TODO! */
            ]
        },
        CharacterId::ABA => {
            vec![
                /* TODO! */
            ]
        }
    }.into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect::<Vec<(String, String)>>()
}