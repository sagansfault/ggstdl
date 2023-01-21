pub mod move_import {
    use lazy_static::lazy_static;
    use regex::Regex;
    use scraper::{ElementRef, Selector};
    use crate::{Character, Move, CharacterId};

    use super::move_search::get_binding_regex;

    pub const MOVE_IMPORT_RESOLVERS: [fn(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>>; 7] = [
        chaos_fire_resolver,
        bridget_normal_resolver,
        faust_100T_resolver,
        leo_guard_resolver,
        jacko_cheer_resolver,

        standard_resolver,
        versioned_rows_resolver
    ];

    lazy_static! {
        static ref VAL_SELECTOR: Selector = scraper::selector::Selector::parse("td").expect("Could not parse selector");
        static ref ROW_SELECTOR: Selector = scraper::selector::Selector::parse("tr").expect("Could not parse selector");
        static ref NAME_SELECTOR: Selector = scraper::selector::Selector::parse("th").expect("Could not parse selector");
    }

    // TODO add special resolver for 5D

     fn chaos_fire_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::HAPPYCHAOS {
            if name.contains("Steady Aim / Fire") {
                let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
                let mut iter = rows.iter();
                let row1 = iter.next().unwrap();
                let row2 = iter.next().unwrap();
                let mut moves: Vec<Move> = vec![];
                let (version_name, damage, guard, startup, active, recovery, onblock, invuln) = versioned_row_parser(row1);
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({})/gmi", version_name).as_str()).unwrap());
                moves.push(Move {
                    name: version_name,
                    matcher: regex,
                    guard,
                    damage,
                    startup,
                    active,
                    recovery,
                    onblock,
                    invuln,
                });
                let (_, damage, guard, startup, active, recovery, onblock, invuln) = versioned_row_parser(row2);
                let version_name = String::from("SA Fire");
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({})/gmi", version_name).as_str()).unwrap());
                moves.push(Move {
                    name: version_name,
                    matcher: regex,
                    guard,
                    damage,
                    startup,
                    active,
                    recovery,
                    onblock,
                    invuln,
                });
                return Some(moves);
            } else if name.contains("At the Ready") && name.contains("236S") {
                return standard_resolver(character, "At the Ready 236S", move_table);
            }
        }
        None
     }

     fn jacko_cheer_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::JACKO {
            if name.contains("Cheer Servant On") {
                if name.contains("S") {
                    return standard_resolver(character, "Cheer Servant On S", move_table);
                } else if name.contains("H") {
                    return standard_resolver(character, "Cheer Servant On H", move_table);
                }
            }
        }
        None
     }

     fn bridget_normal_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::BRIDGET {
            if name.contains("f.SS") {
                return standard_resolver(character, "f.SS", move_table)
            } else if name.contains("5HH") {
                return standard_resolver(character, "5HH", move_table)
            }
        }
        None
     }

     fn faust_100T_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::FAUST {
            if name.contains("100T") {
                let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
                if let Some(data_row) = rows.iter().skip(1).next() {
                    let (damage, guard, startup, active, recovery, onblock, invuln) = standard_row_parser(data_row);
                    let regex = get_binding_regex(character.id, name.to_string()).unwrap_or(Regex::new(format!("/({})/gmi", name).as_str()).unwrap());
                    return Some(vec![Move { name: String::from(name), matcher: regex, guard, damage, startup, active, recovery, onblock, invuln }])
                }
            }
        }
        None
     }

     fn leo_guard_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::LEO {
            if name.contains("[H]") {
                return standard_resolver(character, "Guard", move_table);
            }
        }
        None
     }

    fn standard_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
        if rows.len() == 2 {
            if let Some(data_row) = rows.iter().skip(1).next() {
                let (damage, guard, startup, active, recovery, onblock, invuln) = standard_row_parser(data_row);
                let regex = get_binding_regex(character.id, name.to_string()).unwrap_or(Regex::new(format!("/({})/gmi", name).as_str()).unwrap());
                return Some(vec![Move { name: String::from(name), matcher: regex, guard, damage, startup, active, recovery, onblock, invuln }]);
            }
        }
        None
    }

    fn versioned_rows_resolver(character: &Character, _: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
        if rows.len() > 2 {
            let mut iter = rows.iter();
            iter.next();
            let mut moves: Vec<Move> = vec![];
            for data_row in iter {
                let (version_name, damage, guard, startup, active, recovery, onblock, invuln) = versioned_row_parser(data_row);
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({})/gmi", version_name).as_str()).unwrap());
                moves.push(Move { name: version_name, matcher: regex, guard, damage, startup, active, recovery, onblock, invuln });
            }
            return Some(moves);
        }
        None
    }

    fn standard_row_parser(row: &ElementRef) -> (String, String, String, String, String, String, String) {
        let mut vals = row.select(&VAL_SELECTOR);
        let damage = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let guard = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let startup = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let active = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let recovery = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let onblock = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let invuln = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        (damage, guard, startup, active, recovery, onblock, invuln)
    }

    fn versioned_row_parser(row: &ElementRef) -> (String, String, String, String, String, String, String, String) {
        let mut vals = row.select(&VAL_SELECTOR);
        let version_name = row.select(&NAME_SELECTOR).next().unwrap().inner_html();
        let damage = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let guard = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let startup = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let active = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let recovery = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let onblock = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        let invuln = vals.next().map(|v| v.inner_html()).unwrap_or(String::from(""));
        (version_name, damage, guard, startup, active, recovery, onblock, invuln)
    }
}

pub mod move_search {
    use std::collections::HashMap;
    use regex::Regex;
    use crate::CharacterId;

    lazy_static::lazy_static! {
        static ref MOVE_SEARCH_MATCHERS: HashMap<CharacterId, Vec<(Regex, String)>> = get_all();
    }

    fn get_all() -> HashMap<CharacterId, Vec<(Regex, String)>> {
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

    pub fn get_binding_regex(character_id: CharacterId, official_name: String) -> Option<Regex> {
        MOVE_SEARCH_MATCHERS.get(&character_id).map(|v| {
            for ele in v {
                if ele.1 == official_name {
                    return Some(ele.0.clone());
                }
            }
            return None;
        }).flatten()
    }
    
    fn get_bindings(character_id: CharacterId) -> Vec<(String, String)> {
        match character_id {
            CharacterId::TESTAMENT => {
                vec![
                    (r"/(236HS?|((j.?\s*)?(light)?\s*hs?\s*reaper))/gmi", "236H"),
                    (r"/(236\{HS?\}|((j.?\s*)?(med(ium)?)?\s*hs?\s*reaper))/gmi", "236{H}"),
                    (r"/(236\[HS?\]|((j.?\s*)?(hard|charged?)?\s*hs?\s*reaper))/gmi", "236[H]"),
                    (r"/(236S|((j.?\s*)?(light)?\s*s\s*reaper))/gmi", "236S"),
                    (r"/(236\{S\}|((j.?\s*)?(med(ium)?)?\s*s\s*reaper))/gmi", "236{S}"),
                    (r"/(236\[S\]|((j.?\s*)?(hard|charged?)?\s*s\s*reaper))/gmi", "236[S]"),
                    (r"/(crow|unholy|diver)/gmi", "Unholy Diver"),
                    (r"/(tele)/gmi", "Possession"),
                    (r"/(214S|s\s*arbiter)/gmi", "214S"),
                    (r"/(214H|hs?\s*arbiter)/gmi", "214H"),
                    (r"/(236P236P|nostrovia|succub)/gmi", "Nostrovia"),
                    (r"/(236P236K|calamity\s*one|reversal)/gmi", "Calamity One"),
                ]
            },
            CharacterId::JACKO => {
                vec![
                    (r"/(236K(&|\*))/gmi", "Launched Servant"),
                    (r"/(236K|shoot|kick)/gmi", "236K"),
                    (r"/(236P|summon|pull)/gmi", "236P"),
                    (r"/(236\[P\]|((pull|summon)\s*hold))/gmi", "236[P]"),
                    (r"/(pick|2P)/gmi", "2P"),
                    (r"/(throw|6(P|K|S|HS?|D))/gmi", "Throw Servant"),
                    (r"/(drop|release)/gmi", "Release Servant"),
                    (r"/(unsummon|recover|214P)/gmi", "Recover Servant"),
                    (r"/(attack|214K)/gmi", "Attack Command"),
                    (r"/(defend|block|214S)/gmi", "Defend Command"),
                    (r"/(countdown|bomb|214HS?)/gmi", "Countdown"),
                    (r"/(632146P|F.?E.?D|forever\s*elysion\s*driver)/gmi", "Forever Elysion Driver"),
                    (r"/(s\s*cheer|236236S)/gmi", "Cheer"),
                ]
            },
            CharacterId::NAGORIYUKI => {
                vec![
                    (r"/(f.?S(\s*(level|lv|lvl)?\s*1)?)/gmi", "f.S Level 1"),
                    (r"/(f.?S\s*(level|lv|lvl)?\s*2)/gmi", "f.S Level 2"),
                    (r"/(f.?S\s*(level|lv|lvl)?\s*3)/gmi", "f.S Level 3"),
                    (r"/(f.?S\s*(level|lv|lvl)?\s*BR)/gmi", "f.S Level BR"),

                    (r"/(f.?SS(\s*(level|lv|lvl)?\s*1)?)/gmi", "f.SS Level 1"),
                    (r"/(f.?SS\s*(level|lv|lvl)?\s*2)/gmi", "f.SS Level 2"),
                    (r"/(f.?SS\s*(level|lv|lvl)?\s*3)/gmi", "f.SS Level 3"),
                    (r"/(f.?SS\s*(level|lv|lvl)?\s*BR)/gmi", "f.SS Level BR"),

                    (r"/(f.?SSS(\s*(level|lv|lvl)?\s*1)?)/gmi", "f.SSS Level 1"),
                    (r"/(f.?SSS\s*(level|lv|lvl)?\s*2)/gmi", "f.SSS Level 2"),
                    (r"/(f.?SSS\s*(level|lv|lvl)?\s*3)/gmi", "f.SSS Level 3"),
                    (r"/(f.?SSS\s*(level|lv|lvl)?\s*BR)/gmi", "f.SSS Level BR"),

                    (r"/(5?H(\s*(level|lv|lvl)?\s*1)?)/gmi", "5H Level 1"),
                    (r"/(5?H\s*(level|lv|lvl)?\s*2)/gmi", "5H Level 2"),
                    (r"/(5?H\s*(level|lv|lvl)?\s*3)/gmi", "5H Level 3"),
                    (r"/(5?H\s*(level|lv|lvl)?\s*BR)/gmi", "5H Level BR"),

                    (r"/(2S(\s*(level|lv|lvl)?\s*1)?)/gmi", "2S Level 1"),
                    (r"/(2S\s*(level|lv|lvl)?\s*2)/gmi", "2S Level 2"),
                    (r"/(2S\s*(level|lv|lvl)?\s*3)/gmi", "2S Level 3"),
                    (r"/(2S\s*(level|lv|lvl)?\s*BR)/gmi", "2S Level BR"),

                    (r"/(2H(\s*(level|lv|lvl)?\s*1)?)/gmi", "2H Level 1"),
                    (r"/(2H\s*(level|lv|lvl)?\s*2)/gmi", "2H Level 2"),
                    (r"/(2H\s*(level|lv|lvl)?\s*3)/gmi", "2H Level 3"),
                    (r"/(2H\s*(level|lv|lvl)?\s*BR)/gmi", "2H Level BR"),

                    (r"/(6H(\s*(level|lv|lvl)?\s*1)?)/gmi", "6H Level 1"),
                    (r"/(6H\s*(level|lv|lvl)?\s*2)/gmi", "6H Level 2"),
                    (r"/(6H\s*(level|lv|lvl)?\s*3)/gmi", "6H Level 3"),
                    (r"/(6H\s*(level|lv|lvl)?\s*BR)/gmi", "6H Level BR"),

                    (r"/(j.?S(\s*(level|lv|lvl)?\s*1)?)/gmi", "j.S Level 1"),
                    (r"/(j.?S\s*(level|lv|lvl)?\s*2)/gmi", "j.S Level 2"),
                    (r"/(j.?S\s*(level|lv|lvl)?\s*3)/gmi", "j.S Level 3"),
                    (r"/(j.?S\s*(level|lv|lvl)?\s*BR)/gmi", "j.S Level BR"),

                    (r"/(j.?H(\s*(level|lv|lvl)?\s*1)?)/gmi", "j.H Level 1"),
                    (r"/(j.?H\s*(level|lv|lvl)?\s*2)/gmi", "j.H Level 2"),
                    (r"/(j.?H\s*(level|lv|lvl)?\s*3)/gmi", "j.H Level 3"),
                    (r"/(j.?H\s*(level|lv|lvl)?\s*BR)/gmi", "j.H Level BR"),

                    (r"/(j.?D(\s*(level|lv|lvl)?\s*1)?)/gmi", "j.D Level 1"),
                    (r"/(j.?D\s*(level|lv|lvl)?\s*2)/gmi", "j.D Level 2"),
                    (r"/(j.?D\s*(level|lv|lvl)?\s*3)/gmi", "j.D Level 3"),
                    (r"/(j.?D\s*(level|lv|lvl)?\s*BR)/gmi", "j.D Level BR"),

                    (r"/(236K|fukyo(\s*forward)?)/gmi", "236K"),
                    (r"/(214K|fukyo(\s*back)?)/gmi", "213K"),

                    (r"/(236S|clone|zarameyuki)/gmi", "Zarameyuki"),

                    (r"/(214HS?|beyblade|kamuriyuki)/gmi", "Kamuriyuki"),

                    (r"/(623HS?|shizuriyuki|dp)/gmi", "623H"),
                    (r"/(623HS?HS?|((shizuriyuki|dp)\s*follow))/gmi", "623HH"),

                    (r"/(623P|bite|command|blood)/gmi", "Bloodsucking Universe"),

                    (r"/(632146S|wasureyuki|reversal)/gmi", "Wasureyuki"),

                    (r"/(632146H|zansetsu|reversal)/gmi", "Zansetsu"),
                ]
            },
            CharacterId::MILLIA => {
                vec![
                    (r"/(S\s*disk|236S)/gmi", "236S"),
                    (r"/(HS?\s*disk|236H)/gmi", "236H"),
                    (r"/(moon|(j.?)?236P)/gmi", "Bad Moon"),
                    (r"/(214P|hair|car)/gmi", "Iron Savior"),
                    (r"/(turbo|fall|(j.?)?236K)/gmi", "Turbo Fall"),
                    (r"/(214K|mirazh)/gmi", "Mirazh"),
                    (r"/(lust|shaker|214S)/gmi", "Lust Shaker"),
                    (r"/(kapel|j.?236HS?)/gmi", "Kapel"),
                    (r"/(632146HS?|winger|reversal)/gmi", "Winger"),
                    (r"/(236236S|septum)/gmi", "Septum Voices"),
                ]
            },
            CharacterId::CHIPP => {
                vec![
                    (r"/(236P|p\s*alpha)/gmi", "236P"),
                    (r"/(j.?236P|((air|j.?)\s*p\s*alpha))/gmi", "j.236P"),
                    (r"/(236K|k\s*alpha)/gmi", "236K"),
                    (r"/(j.?236K|((air|j.?)\s*k\s*alpha))/gmi", "j.236K"),
                    (r"/(623S|dp|beta)/gmi", "623S"),
                    (r"/(j.?(623P|dp|beta))/gmi", "j.623S"),
                    (r"/(236HS?|gamma|clone)/gmi", "Gamma Blade"),
                    (r"/(236S|rekka(\s*1)?|resshou)/gmi", "Resshou"),
                    (r"/(rekka\s*2|rokusai)/gmi", "Rokusai"),
                    (r"/(senshuu?|rekka\s*3)/gmi", "Senshuu"),
                    (r"/(63214S|command|grab)/gmi", "Genrou Zan"),
                    (r"/(j.?214P|shuriken)/gmi", "Shuriken"),
                    (r"/(632146HS?|zansei)/gmi", "Zansei Rouga"),
                    (r"/(236236P|banki)/gmi", "Banki Messai"),
                ]
            },
            CharacterId::SOL => {
                vec![
                    (r"/(gun|flame|236P)/gmi", "Gun Flame"),
                    (r"/(feint|214P)/gmi", "Gun Flame (Feint)"),
                    (r"/(vv|623S)/gmi", "623S"),
                    (r"/(hvv|623HS?|dp)/gmi", "623H"),
                    (r"/(j.?\s*(vv|623S))/gmi", "j.633H"),
                    (r"/(j.?\s*(hvv|623HS?|dp))/gmi", "j.633H"),
                    (r"/(revolver|br|236K)/gmi", "236K"),
                    (r"/(236KK)/gmi", "236KK"),
                    (r"/(j.?\s*(revolver|br|236K))/gmi", "j.236K"),
                    (r"/(j.?\s*(236KK))/gmi", "2.236KK"),
                    (r"/(bringer|bb|236K)/gmi", "214K"),
                    (r"/(j.?\s*(bringer|bb|236K))/gmi", "j.214K"),
                    (r"/(623K|wild|throw|grab)/gmi", "Wild Throw"),
                    (r"/(nrv|214S|vortex)/gmi", "Night Raid Vortex"),
                    (r"/(fafnir|41236HS?)/gmi", "Fafnir"),
                    (r"/(632146HS?|tyrant|rave)/gmi", "Tyrant Rave"),
                    (r"/(hmc|mob|cemetary|214214HS?)/gmi", "Heavy Mob Cemetery"),
                ]
            },
            CharacterId::KY => {
                vec![
                    (r"/(edge|236S)/gmi", "236S"),
                    (r"/DI\s*(edge|236S)/gmi", "DI 236S"),
                    (r"/(charge|236HS?)/gmi", "236H"),
                    (r"/DI\s*(charge|236HS?)/gmi", "236H"),
                    (r"/j.?\s*(arial|236S)/gmi", "j.236S"),
                    (r"/j.?\s*(arial|236HS?)/gmi", "j.236H"),
                    (r"/(dip|236K)/gmi", "236K"),
                    (r"/DI\s*(dip|236K)/gmi", "DI 236K"),
                    (r"/(flip|foudre|214K)/gmi", "214K"),
                    (r"/DI\s*(flip|foudre|214K)/gmi", "DI 214K"),
                    (r"/(623S)/gmi", "623S"),
                    (r"/DI\s*(623S)/gmi", "DI 623S"),
                    (r"/(623HS?|dp|vapor|thrust)/gmi", "623H"),
                    (r"/DI\s*(623HS?|dp|vapor|thrust)/gmi", "DI 623H"),
                    (r"/(dire|eclat|214S)/gmi", "214S"),
                    (r"/DI\s*(dire|eclat|214S)/gmi", "DI 214S"),
                    (r"/(rtl|ride|lightning|632146HS?)/gmi", "632146H"),
                    (r"/DI\s*(rtl|ride|lightning|632146HS?)/gmi", "DI 632146H"),
                    (r"/(sacred|236236P)/gmi", "236236P"),
                    (r"/DI\s*(sacred|236236P)/gmi", "236236P"),
                    (r"/(di|dragon|install|214214HS?)/gmi", "Dragon Install"),
                ]
            },
            CharacterId::MAY => {
                vec![
                    (r"/(\[4\]6S|s\s*dolphin)/gmi", "[4]6S"),
                    (r"/(\[4\]6HS?|hs?*dolphin)/gmi", "[4]6H"),
                    (r"/(\[2\]8S|up\s*s\s*dolphin)/gmi", "[2]8S"),
                    (r"/(\[2\]8HS?|up\s*hs?\s*dolphin)/gmi", "[2]8H"),
                    (r"/(ok|overhead|kiss|623K|command|grab)/gmi", "Overhead Kiss"),
                    (r"/(214P)/gmi", "214P"),
                    (r"/(beach|ball|214K)/gmi", "214K"),
                    (r"/(yamada|236236S)/gmi", "Great Yamada Attack"),
                    (r"/(orca|632146HS?)/gmi", "The Wonderful and Dynamic Goshogawara"),
                ]
            },
            CharacterId::ZATO => {
                vec![
                    (r"/(summon|214HS?)/gmi", "Summon Eddie"),
                    (r"/(unsummon)/gmi", "Unsummon Eddie"),

                    (r"/(pierce|236P)/gmi", "236P"),
                    (r"/(\]P\[|-P-)/gmi", "]P["),

                    (r"/(that's a lot|drills|236K)/gmi", "236K"),
                    (r"/(\]K\[|-K-)/gmi", "]K["),

                    (r"/(leap|frog|236S)/gmi", "236S"),
                    (r"/(\]S\[|-S-)/gmi", "]S["),

                    (r"/(oppose|236HS?)/gmi", "236H"),
                    (r"/(\]HS?\[|-HS?-)/gmi", "]H["),

                    (r"/(invite|hell|22HS?)/gmi", "Invite Hell"),
                    (r"/(btl|break|law|214K)/gmi", "Break The Law"),
                    (r"/(damned|fang|command|grab|623S)/gmi", "Damned Fang"),
                    (r"/(214S|shade|drunk)/gmi", "Drunkard Shade"),
                    (r"/(632146HS?|amongus|amor)/gmi", "Amorphous"),
                    (r"/(sun|void|632146S|sword|excalibur)/gmi", "Sun Void"),
                ]
            },
            CharacterId::INO => {
                vec![
                    (r"/(note|anti|214P)/gmi", "214P"),
                    (r"/(j.?\s*(note|anti|214P))/gmi", "j.214P"),
                    (r"/(s\s*stroke)/gmi", "236S"),
                    (r"/(hs?\s*stroke)/gmi", "236H"),
                    (r"/(j.?\s*236K)/gmi", "j.236K"),
                    (r"/(j.?\s*236S)/gmi", "j.236S"),
                    (r"/(j.?\s*236HS?)/gmi", "j.236H"),
                    (r"/(love|chemical|214K)/gmi", "214K"),
                    (r"/(j.?\s*(love|chemical|214K))/gmi", "j.214K"),
                    (r"/(mega|632146HS?)/gmi", "Megalomania"),
                    (r"/(ultimate|fort|632146S)/gmi", "632146S"),
                    (r"/(j.?\s*(ultimate|fort|632146S))/gmi", "j.632146S"),
                ]
            },
            CharacterId::HAPPYCHAOS => {
                vec![
                    (r"/(H)/gmi", "At The Ready"),
                    (r"/(\]H\[|fire|shot)/gmi", "Fire"),
                    (r"/(atr|236S|flip)/gmi", "At The Ready 236S"),
                    (r"/(steady|aim)/gmi", "Steady Aim"),
                    (r"/((steady|aim)\s*(shot|fire))/gmi", "Fire"),
                    (r"/(cancel|2H|stow)/gmi", "236S 2H"),
                    (r"/((steady|aim)\s*(cancel|stow))/gmi", "214S 214S"),
                    (r"/(reload|22P)/gmi", "Reload"),
                    (r"/(focus|214P)/gmi", "Focus"),
                    (r"/(curse|ball|236P)/gmi", "Curse"),
                    (r"/(clone|236K)/gmi", "Scapegoat"),
                    (r"/(roll|214K)/gmi", "Roll"),
                    (r"/(dem|deus|ex|machina|632146S)/gmi", "Deus Ex Machina"),
                    (r"/(super\s*focus|214214P)/gmi", "Super Focus"),
                ]
            },
            CharacterId::SIN => {
                vec![
                    (r"/(hawk|baker|623S|dp)/gmi", "Hawk Baker"),
                    (r"/((hawk|baker|623S|dp)\s*(~?S|follow))/gmi", "Hawk Baker Follow-up"),
                    (r"/(elk|hunt|236K)/gmi", "236K"),
                    (r"/((elk|hunt|236K)\s*(~?K|follow))/gmi", "236K~K"),
                    (r"/(hoof|stomp|214S)/gmi", "214S"),
                    (r"/((hoof|stomp|214S)\s*(~?S|follow))/gmi", "214S~S"),
                    (r"/(gazelle|dash)/gmi", "Gazelle Step"),
                    (r"/(food|eat|grow|63214P)/gmi", "Still Growing"),
                    (r"/(rtl|ride|lightning|632146HS?)/gmi", "632146H"),
                    (r"/((rtl|ride|lightning|632146HS?)\s*(~?HS?|follow))/gmi", "632146HH"),
                    (r"/(barrel|tyrant|236236P)/gmi", "236236P"),
                    (r"/((barrel|tyrant|236236P)\s*(~?\[?P\]?|follow))/gmi", "236236P~]P["),
                ]
            },
            CharacterId::BAIKEN => {
                vec![
                    (r"/(tatami|mat|gaeshi|236K)/gmi", "236K"),
                    (r"/(j.?\s*(tatami|mat|gaeshi|236K))/gmi", "j.236K"),
                    (r"/(s\s*kabari|41236S)/gmi", "41236S"),
                    (r"/(hs?\s*kabari|41236HS?)/gmi", "41236H"),
                    (r"/((hs?\s*kabari|41236HS?)\s*(follow|~?HS?))/gmi", "41236H~H"),
                    (r"/(yozansen|youzansen|tk|236S)/gmi", "Youzansen"),
                    (r"/(parry|Hiiragi|236P)/gmi", "Hiiragi"),
                    (r"/(236236S|watashi|tsurane|sanzu)/gmi", "Tsurane Sanzu-watashi"),
                    (r"/(gun|kenjyu|214214P)/gmi", "214214P"),
                    (r"/(j.?\s*(gun|kenjyu|214214P))/gmi", "j.214214P"),
                ]
            },
            CharacterId::ANJI => {
                vec![
                    (r"/(butter|shitsu|fire|236P)/gmi", "Shitsu"),
                    (r"/(parry|suigetsu|spin|236K)/gmi", "Suigetsu No Hakobi"),
                    (r"/(fuujin|fujin|236HS?)/gmi", "Fuujin"),
                    (r"/((fuujin|fujin|236HS?)\s*P)/gmi", "Shin: Ichishiki"),
                    (r"/((fuujin|fujin|236HS?)\s*K)/gmi", "Issokutobi"),
                    (r"/((fuujin|fujin|236HS?)\s*S)/gmi", "Nagiha"),
                    (r"/((fuujin|fujin|236HS?)\s*HS?)/gmi", "Rin"),
                    (r"/(kou|236S)/gmi", "Kou"),
                    (r"/(issei|ougi|632146S?)/gmi", "Issei Ougi: Sai"),
                    (r"/(kach|632146S)/gmi", "Kachoufuugetsu Kai"),
                ]
            },
            CharacterId::LEO => {
                vec![
                    (r"/(hyper|guard|\[HS?\]S|\[S\]HS?)/gmi", "Guard"),
                    (r"/(s\s*(fire|ball|grav))/gmi", "[4]6S"),
                    (r"/(hs?\s*(fire|ball|grav))/gmi", "[4]6H"),
                    (r"/((s\*(dp|ein))|\[2\]8S)/gmi", "[2]8S"),
                    (r"/((dp|ein)|\[2\]8HS?)/gmi", "[2]8H"),
                    (r"/(236S|erstes)/gmi", "Erstes Kaltes Gestöber"),
                    (r"/(236HS?|zwe)/gmi", "Zweites Kaltes Gestöber"),
                    (r"/(214S|turb)/gmi", "Turbulenz"),
                    (r"/(parry|kahn|schild|sheild|bt\.D)/gmi", "Kahn-Schild"),
                    (r"/(command|grab|dunkel|214K)/gmi", "Glänzendes Dunkel"),
                    (r"/(blitz|214HS?)/gmi", "Blitzschlag"),
                    (r"/(632146S|stahl)/gmi", "Stahlwirbel"),
                    (r"/(632146HS?|lei)/gmi", "Leidenschaft des Dirigenten"),
                ]
            },
            CharacterId::FAUST => {
                vec![
                    (r"/(scalpel|thrust|41236K)/gmi", "Thrust"),
                    (r"/(pull|back)/gmi", "Pull Back"),
                    (r"/(golf|club|hole|41236K\s*HS?)/gmi", "Hole in One!"),
                    (r"/(item|toss|236P|what)/gmi", "What Could This Be?"),
                    (r"/(mmm|mix|236S)/gmi", "Mix Mix Mix"),
                    (r"/(snip|command|grab|236HS?)/gmi", "Snip Snip Snip"),
                    (r"/((j.?)?love|j.?236P)/gmi", "j.236P"),
                    (r"/((j.?)?love|j.?236P)\s*(afro)/gmi", "j.236P (With Afro)"),
                    (r"/((p\s*crow)|214P)/gmi", "214P"),
                    (r"/((k\s*crow)|214K)/gmi", "214K"),
                    (r"/((s\s*crow)|214S)/gmi", "214S"),
                    (r"/(bone|wheel|chair|reversal|632146HS?)/gmi", "Bone-crushing Excitement"),
                    (r"/(236236P|item\s*super)/gmi", "W-W-What Could This Be?"),
                    (r"/(236236236236P)/gmi", "W-W-W-W-W-W-W-W-W-What Could This Be?"),
                ]
            },
            CharacterId::AXL => {
                vec![
                    (r"/(rensen|rensin|\[4\]6S|flash)/gmi", "Sickle Flash"),
                    (r"/((rensen|rensin|\[4\]6S|flash)\s*(8|up))/gmi", "Soaring Chain Strike"),
                    (r"/((rensen|rensin|\[4\]6S|flash)\s*(2|down))/gmi", "Spinning Chain Strike"),
                    (r"/(cherry|((rensen|rensin|\[4\]6S|flash)\s*(s|bomb)))/gmi", "Winter Cherry"),
                    (r"/(mantis|command|grab|41236HS?)/gmi", "Winter Mantis"),
                    (r"/(rain|water|216S)/gmi", "Rainwater"),
                    (r"/(snail|214HS?)/gmi", "214H"),
                    (r"/(j.?\s*(snail|214HS?))/gmi", "j.214H"),
                    (r"/(bomber|j.?\s*236HS?)/gmi", "Axl Bomber"),
                    (r"/(reversal|storm|236236HS?)/gmi", "Sickle Storm"),
                    (r"/(one|vision|time\s*stop|632146P)/gmi", "632146P"),
                    (r"/((one|vision|time\s*stop|632146P)\s*activ)/gmi", "632146P Attack"),
                ]
            },
            CharacterId::POTEMKIN => {
                vec![
                    (r"/(pb|grab|buster|360P|632146P)/gmi", "Potemkin Buster"),
                    (r"/(heat|knuckle|hk|623HS?)/gmi", "Heat Knuckle"),
                    (r"/(fmf|mf|236P|fist|mega)/gmi", "236P"),
                    (r"/((back|b)\s*(mega|fist|214P|mf))/gmi", "214P"),
                    (r"/((k|kara)\s*(back|b)\s*(mega|fist|214P|mf))/gmi", "2146K~P"),
                    (r"/(slide|head|236S)/gmi", "Slide Head"),
                    (r"/(hammer|fall|\[4\]6HS?|hf)/gmi", "Hammer Fall"),
                    (r"/((hammer|fall|\[4\]6HS?|hf)\s*(break|b))/gmi", "Hammer Fall Break"),
                    (r"/(flick|f.?d.?b.?)/gmi", "F.D.B."),
                    (r"/((flick|f.?d.?b.?)\s*charge)/gmi", "F.D.B. (Charged)"),
                    (r"/((flick|f.?d.?b.?)\s*(hit|reflect))/gmi", "Reflect Projectile"),
                    (r"/(garuda|214HS?)/gmi", "Garuda Impact"),
                    (r"/(hpb|236236S|heavenly)/gmi", "Heavenly Potemkin Buster"),
                    (r"/(giganter|kai|632146HS?)/gmi", "Giganter Kai"),
                    (r"/(giganter|kai|632146HS?)\s*(barrier)/gmi", "Giganter Kai (Barrier)"),
                ]
            },
            CharacterId::RAMLETHAL => {
                vec![
                    (r"/(623P|dp|dauro)/gmi", "Dauro"),
                    (r"/(rekka|214P|erar)/gmi", "214P"),
                    (r"/((rekka|214P|erar)\s*2)/gmi", "214P 214P"),
                    (r"/((rekka|214P|erar)\s*3)/gmi", "214P 214P 214P"),
                    (r"/(flip|214K|slido)/gmi", "214K"),
                    (r"/(j.?\s*(flip|214K|slido))/gmi", "j.214K"),
                    (r"/(sword|throw|toss|bajoneto|236S)/gmi", "236S"),
                    (r"/((hs?)\s*(sword|throw|toss|bajoneto|236HS?))/gmi", "236H"),
                    (r"/(agresa|(j.?\s*214S))/gmi", "Agresa Ordono"),
                    (r"/(wind|wiper|214HS?)/gmi", "Sabrobato"),
                    (r"/(calvados|63214HS?)/gmi", "Calvados"),
                    (r"/(mortobato|reversal|236236S)/gmi", "Mortobato"),
                ]
            },
            CharacterId::GIO => {
                vec![
                    (r"/(kick|214K|sep)/gmi", "Sepultura"),
                    (r"/(drill|dog|236K|tro)/gmi", "Trovão"),
                    (r"/(623S|dp|nascente)/gmi", "Sol Nascente"),
                    (r"/(214S|sol|poente)/gmi", "214S"),
                    (r"/(j.?\s*(214S|sol|poente))/gmi", "j.214S"),
                    (r"/(spin|reversal|63214HS?)/gmi", "Ventania"),
                    (r"/(temp|air|(j.?\s*236236HS?))/gmi", "Tempestade"),
                ]
            },
            CharacterId::GOLDLEWIS => {
                vec![
                    (r"/(426(HS?)?)/gmi", "41236H"),
                    (r"/(j.?\s*(426(HS?)?))/gmi", "j.41236H"),

                    (r"/(624(HS?)?)/gmi", "63214H"),
                    (r"/(j.?\s*(624(HS?)?))/gmi", "j.63214H"),

                    (r"/(268(HS?)?)/gmi", "23698H"),
                    (r"/(j.?\s*(268(HS?)?))/gmi", "j.23698H"),

                    (r"/(684(HS?)?)/gmi", "69874H"),
                    (r"/(j.?\s*(684(HS?)?))/gmi", "j.69874H"),

                    (r"/(426(HS?)?)/gmi", "41236H"),
                    (r"/(j.?\s*(426(HS?)?))/gmi", "j.41236H"),

                    (r"/(486(HS?)?)/gmi", "87412H"),
                    (r"/(j.?\s*(486(HS?)?))/gmi", "j.87412H"),

                    (r"/(842(HS?)?)/gmi", "41236H"),
                    (r"/(j.?\s*(842(HS?)?))/gmi", "j.41236H"),

                    (r"/(862(HS?)?)/gmi", "89632H"),
                    (r"/(j.?\s*(862(HS?)?))/gmi", "j.89632H"),

                    (r"/(drone|214S)/gmi", "214S Level 1"),
                    (r"/((drone|214S)\s*(level|lv|lvl)?\s*2)/gmi", "214S Level 2"),
                    (r"/((drone|214S)\s*(level|lv|lvl)?\s*3)/gmi", "214S Level 3"),

                    (r"/(gun|mini|skyfish|236S)/gmi", "236S Level 1"),
                    (r"/((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*2)/gmi", "236S Level 2"),
                    (r"/((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*3)/gmi", "236S Level 3"),

                    (r"/(dwts|down|system|reversal|360P|63214P)/gmi", "632146P"),
                    (r"/(720P)/gmi", "720P"),
                    (r"/(1080P)/gmi", "1080P"),

                    (r"/(beam|burn|236236K)/gmi", "236236K Level 1"),
                    (r"/((beam|burn|236236K)\s*(level|lv|lvl)?\s*2)/gmi", "236236K Level 2"),
                    (r"/((beam|burn|236236K)\s*(level|lv|lvl)?\s*3)/gmi", "236236K Level 3"),
                ]
            },
            CharacterId::BRIDGET => {
                vec![
                    (r"/((236(S|(HS?))|yoyo|toss))/gmi", "Stop and Dash (Hit on send)"),
                    (r"/(roll|spin|214K)/gmi", "Rolling Movement"),
                    (r"/(dp|star|ship|623P)/gmi", "Starship"),
                    (r"/(car|kick|start|heart|236K)/gmi", "Kick Start My Heart"),
                    (r"/(brake|((car|kick|start|heart|236K)\s*P))/gmi", "Brake"),
                    (r"/(shoot|((car|kick|start|heart|236K)\s*K))/gmi", "Shoot"),
                    (r"/(dive|(j.?\s*236K))/gmi", "Roger Dive"),
                    (r"/(command|grab|rock|baby|63214P)/gmi", "Rock the Baby"),
                    (r"/(loop|632146S)/gmi", "Loop the Loop"),
                    (r"/(motor|killing|632146HS?|return)/gmi", "Return of the Killing Machine"),
                ]
            },
            _ => vec![]
        }.into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect::<Vec<(String, String)>>()
    }
}