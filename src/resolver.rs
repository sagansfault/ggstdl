pub mod move_import {
    use lazy_static::lazy_static;
    use regex::Regex;
    use scraper::{ElementRef, Selector};
    use crate::{Character, Move, CharacterId};

    use super::move_search::get_binding_regex;

    pub const MOVE_IMPORT_RESOLVERS: [fn(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>>; 8] = [
        work_around_5D_resolver,
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
    fn work_around_5D_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        Some(vec![])
    }

     fn chaos_fire_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        if character.id == CharacterId::HAPPYCHAOS {
            if name.contains("Steady Aim / Fire") {
                let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
                let mut iter = rows.iter();
                let row1 = iter.next().unwrap();
                let row2 = iter.next().unwrap();
                let mut moves: Vec<Move> = vec![];
                let (version_name, 
                    damage, 
                    guard, 
                    startup, 
                    active, 
                    recovery, 
                    onblock, 
                    invuln) = versioned_row_parser(row1).expect(format!("could not load moves for {}", name).as_str());
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({}))", version_name).as_str()).unwrap());
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
                let (_, 
                    damage, 
                    guard, 
                    startup, 
                    active, 
                    recovery, 
                    onblock, 
                    invuln) = versioned_row_parser(row2).expect(format!("could not load moves for {}", name).as_str());
                let version_name = String::from("SA Fire");
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({}))", version_name).as_str()).unwrap());
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
                    let (damage, 
                        guard, 
                        startup, 
                        active, 
                        recovery, 
                        onblock, 
                        invuln) = standard_row_parser(data_row).expect(format!("could not load moves for {}", name).as_str());
                    let regex = get_binding_regex(character.id, name.to_string()).unwrap_or(Regex::new(format!("/({}))", name).as_str()).unwrap());
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
                let (damage, 
                    guard, 
                    startup, 
                    active, 
                    recovery, 
                    onblock, 
                    invuln) = standard_row_parser(data_row).expect(format!("could not load moves for {}", name).as_str());
                let regex = get_binding_regex(character.id, name.to_string()).unwrap_or(Regex::new(format!("/({}))", name).as_str()).unwrap());
                return Some(vec![Move { name: String::from(name), matcher: regex, guard, damage, startup, active, recovery, onblock, invuln }]);
            }
        }
        None
    }

    fn versioned_rows_resolver(character: &Character, name: &str, move_table: ElementRef) -> Option<Vec<Move>> {
        let rows: Vec<ElementRef> = move_table.select(&ROW_SELECTOR).peekable().collect();
        if rows.len() > 2 {
            let mut iter = rows.iter();
            iter.next();
            let mut moves: Vec<Move> = vec![];
            for data_row in iter {
                let (version_name, 
                    damage, 
                    guard, 
                    startup, 
                    active, 
                    recovery, 
                    onblock, 
                    invuln) = versioned_row_parser(data_row).expect(format!("could not load moves for {}", name).as_str());
                let regex = get_binding_regex(character.id, version_name.clone()).unwrap_or(Regex::new(format!("/({}))", version_name).as_str()).unwrap());
                moves.push(Move { name: version_name, matcher: regex, guard, damage, startup, active, recovery, onblock, invuln });
            }
            return Some(moves);
        }
        None
    }

    fn standard_row_parser(row: &ElementRef) -> Option<(String, String, String, String, String, String, String)> {
        let mut vals = row.select(&VAL_SELECTOR);
        let damage = vals.next().map(|v| v.inner_html())?;
        let guard = vals.next().map(|v| v.inner_html())?;
        let startup = vals.next().map(|v| v.inner_html())?;
        let active = vals.next().map(|v| v.inner_html())?;
        let recovery = vals.next().map(|v| v.inner_html())?;
        let onblock = vals.next().map(|v| v.inner_html())?;
        let invuln = vals.next().map(|v| v.inner_html())?;
        Some((damage, guard, startup, active, recovery, onblock, invuln))
    }

    fn versioned_row_parser(row: &ElementRef) -> Option<(String, String, String, String, String, String, String, String)> {
        let mut vals = row.select(&VAL_SELECTOR);
        let version_name = row.select(&NAME_SELECTOR).next()?.inner_html();
        let damage = vals.next().map(|v| v.inner_html())?;
        let guard = vals.next().map(|v| v.inner_html())?;
        let startup = vals.next().map(|v| v.inner_html())?;
        let active = vals.next().map(|v| v.inner_html())?;
        let recovery = vals.next().map(|v| v.inner_html())?;
        let onblock = vals.next().map(|v| v.inner_html())?;
        let invuln = vals.next().map(|v| v.inner_html())?;
        Some((version_name, damage, guard, startup, active, recovery, onblock, invuln))
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
                    (r"(?i)((236HS?|((j.?\s*)?(light)?\s*hs?\s*reaper)))", "236H"),
                    (r"(?i)((236\{HS?\}|((j.?\s*)?(med(ium)?)?\s*hs?\s*reaper)))", "236{H}"),
                    (r"(?i)((236\[HS?\]|((j.?\s*)?(hard|charged?)?\s*hs?\s*reaper)))", "236[H]"),
                    (r"(?i)(?i)(?i)((236S|((j.?\s*)?(light)?\s*s\s*reaper)))", "236S"),
                    (r"(?i)(?i)(?i)(?i)((236\{S\}|((j.?\s*)?(med(ium)?)?\s*s\s*reaper)))", "236{S}"),
                    (r"(?i)((236\[S\]|((j.?\s*)?(hard|charged?)?\s*s\s*reaper)))", "236[S]"),
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
                    (r"(?i)((236K(&|\*)))", "Launched Servant"),
                    (r"(?i)((236K|shoot|kick))", "236K"),
                    (r"(?i)((236P|summon|pull))", "236P"),
                    (r"(?i)((236\[P\]|((pull|summon)\s*hold)))", "236[P]"),
                    (r"(?i)((pick|2P))", "2P"),
                    (r"(?i)((throw|6(P|K|S|HS?|D)))", "Throw Servant"),
                    (r"(?i)((drop|release))", "Release Servant"),
                    (r"(?i)((unsummon|recover|214P))", "Recover Servant"),
                    (r"(?i)((attack|214K))", "Attack Command"),
                    (r"(?i)((defend|block|214S))", "Defend Command"),
                    (r"(?i)((countdown|bomb|214HS?))", "Countdown"),
                    (r"(?i)((632146P|F.?E.?D|forever\s*elysion\s*driver))", "Forever Elysion Driver"),
                    (r"(?i)((s\s*cheer|236236S))", "Cheer"),
                ]
            },
            CharacterId::NAGORIYUKI => {
                vec![
                    (r"(?i)((f.?S(\s*(level|lv|lvl)?\s*1)?))", "f.S Level 1"),
                    (r"(?i)((f.?S\s*(level|lv|lvl)?\s*2))", "f.S Level 2"),
                    (r"(?i)((f.?S\s*(level|lv|lvl)?\s*3))", "f.S Level 3"),
                    (r"(?i)((f.?S\s*(level|lv|lvl)?\s*BR))", "f.S Level BR"),

                    (r"(?i)((f.?SS(\s*(level|lv|lvl)?\s*1)?))", "f.SS Level 1"),
                    (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*2))", "f.SS Level 2"),
                    (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*3))", "f.SS Level 3"),
                    (r"(?i)((f.?SS\s*(level|lv|lvl)?\s*BR))", "f.SS Level BR"),

                    (r"(?i)((f.?SSS(\s*(level|lv|lvl)?\s*1)?))", "f.SSS Level 1"),
                    (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*2))", "f.SSS Level 2"),
                    (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*3))", "f.SSS Level 3"),
                    (r"(?i)((f.?SSS\s*(level|lv|lvl)?\s*BR))", "f.SSS Level BR"),

                    (r"(?i)((5?H(\s*(level|lv|lvl)?\s*1)?))", "5H Level 1"),
                    (r"(?i)((5?H\s*(level|lv|lvl)?\s*2))", "5H Level 2"),
                    (r"(?i)((5?H\s*(level|lv|lvl)?\s*3))", "5H Level 3"),
                    (r"(?i)((5?H\s*(level|lv|lvl)?\s*BR))", "5H Level BR"),

                    (r"(?i)((2S(\s*(level|lv|lvl)?\s*1)?))", "2S Level 1"),
                    (r"(?i)((2S\s*(level|lv|lvl)?\s*2))", "2S Level 2"),
                    (r"(?i)((2S\s*(level|lv|lvl)?\s*3))", "2S Level 3"),
                    (r"(?i)((2S\s*(level|lv|lvl)?\s*BR))", "2S Level BR"),

                    (r"(?i)((2H(\s*(level|lv|lvl)?\s*1)?))", "2H Level 1"),
                    (r"(?i)((2H\s*(level|lv|lvl)?\s*2))", "2H Level 2"),
                    (r"(?i)((2H\s*(level|lv|lvl)?\s*3))", "2H Level 3"),
                    (r"(?i)((2H\s*(level|lv|lvl)?\s*BR))", "2H Level BR"),

                    (r"(?i)((6H(\s*(level|lv|lvl)?\s*1)?))", "6H Level 1"),
                    (r"(?i)((6H\s*(level|lv|lvl)?\s*2))", "6H Level 2"),
                    (r"(?i)((6H\s*(level|lv|lvl)?\s*3))", "6H Level 3"),
                    (r"(?i)((6H\s*(level|lv|lvl)?\s*BR))", "6H Level BR"),

                    (r"(?i)((j.?S(\s*(level|lv|lvl)?\s*1)?))", "j.S Level 1"),
                    (r"(?i)((j.?S\s*(level|lv|lvl)?\s*2))", "j.S Level 2"),
                    (r"(?i)((j.?S\s*(level|lv|lvl)?\s*3))", "j.S Level 3"),
                    (r"(?i)((j.?S\s*(level|lv|lvl)?\s*BR))", "j.S Level BR"),

                    (r"(?i)((j.?H(\s*(level|lv|lvl)?\s*1)?))", "j.H Level 1"),
                    (r"(?i)((j.?H\s*(level|lv|lvl)?\s*2))", "j.H Level 2"),
                    (r"(?i)((j.?H\s*(level|lv|lvl)?\s*3))", "j.H Level 3"),
                    (r"(?i)((j.?H\s*(level|lv|lvl)?\s*BR))", "j.H Level BR"),

                    (r"(?i)((j.?D(\s*(level|lv|lvl)?\s*1)?))", "j.D Level 1"),
                    (r"(?i)((j.?D\s*(level|lv|lvl)?\s*2))", "j.D Level 2"),
                    (r"(?i)((j.?D\s*(level|lv|lvl)?\s*3))", "j.D Level 3"),
                    (r"(?i)((j.?D\s*(level|lv|lvl)?\s*BR))", "j.D Level BR"),

                    (r"(?i)((236K|fukyo(\s*forward)?))", "236K"),
                    (r"(?i)((214K|fukyo(\s*back)?))", "213K"),

                    (r"(?i)((236S|clone|zarameyuki))", "Zarameyuki"),

                    (r"(?i)((214HS?|beyblade|kamuriyuki))", "Kamuriyuki"),

                    (r"(?i)((623HS?|shizuriyuki|dp))", "623H"),
                    (r"(?i)((623HS?HS?|((shizuriyuki|dp)\s*follow)))", "623HH"),

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
                    (r"(?i)((236S|rekka(\s*1)?|resshou))", "Resshou"),
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
                    (r"(?i)((gun|flame|236P))", "Gun Flame"),
                    (r"(?i)((feint|214P))", "Gun Flame (Feint)"),
                    (r"(?i)((vv|623S))", "623S"),
                    (r"(?i)((hvv|623HS?|dp))", "623H"),
                    (r"(?i)((j.?\s*(vv|623S)))", "j.633H"),
                    (r"(?i)((j.?\s*(hvv|623HS?|dp)))", "j.633H"),
                    (r"(?i)((revolver|br|236K))", "236K"),
                    (r"(?i)((236KK))", "236KK"),
                    (r"(?i)((j.?\s*(revolver|br|236K)))", "j.236K"),
                    (r"(?i)((j.?\s*(236KK)))", "2.236KK"),
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
                    (r"(?i)((edge|236S))", "236S"),
                    (r"(?i)(DI\s*(edge|236S))", "DI 236S"),
                    (r"(?i)((charge|236HS?))", "236H"),
                    (r"(?i)(DI\s*(charge|236HS?))", "236H"),
                    (r"(?i)(j.?\s*(arial|236S))", "j.236S"),
                    (r"(?i)(j.?\s*(arial|236HS?))", "j.236H"),
                    (r"(?i)((dip|236K))", "236K"),
                    (r"(?i)(DI\s*(dip|236K))", "DI 236K"),
                    (r"(?i)((flip|foudre|214K))", "214K"),
                    (r"(?i)(DI\s*(flip|foudre|214K))", "DI 214K"),
                    (r"(?i)((623S))", "623S"),
                    (r"(?i)(DI\s*(623S))", "DI 623S"),
                    (r"(?i)((623HS?|dp|vapor|thrust))", "623H"),
                    (r"(?i)(DI\s*(623HS?|dp|vapor|thrust))", "DI 623H"),
                    (r"(?i)((dire|eclat|214S))", "214S"),
                    (r"(?i)(DI\s*(dire|eclat|214S))", "DI 214S"),
                    (r"(?i)((rtl|ride|lightning|632146HS?))", "632146H"),
                    (r"(?i)(DI\s*(rtl|ride|lightning|632146HS?))", "DI 632146H"),
                    (r"(?i)((sacred|236236P))", "236236P"),
                    (r"(?i)(DI\s*(sacred|236236P))", "236236P"),
                    (r"(?i)((di|dragon|install|214214HS?))", "Dragon Install"),
                ]
            },
            CharacterId::MAY => {
                vec![
                    (r"(?i)((\[4\]6S|s\s*dolphin))", "[4]6S"),
                    (r"(?i)((\[4\]6HS?|hs?*dolphin))", "[4]6H"),
                    (r"(?i)((\[2\]8S|up\s*s\s*dolphin))", "[2]8S"),
                    (r"(?i)((\[2\]8HS?|up\s*hs?\s*dolphin))", "[2]8H"),
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
                    (r"(?i)((H))", "At The Ready"),
                    (r"(?i)((\]H\[|fire|shot))", "Fire"),
                    (r"(?i)((atr|236S|flip))", "At The Ready 236S"),
                    (r"(?i)((steady|aim))", "Steady Aim"),
                    (r"(?i)(((steady|aim)\s*(shot|fire)))", "Fire"),
                    (r"(?i)((cancel|2H|stow))", "236S 2H"),
                    (r"(?i)(((steady|aim)\s*(cancel|stow)))", "214S 214S"),
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
                    (r"(?i)((hawk|baker|623S|dp))", "Hawk Baker"),
                    (r"(?i)(((hawk|baker|623S|dp)\s*(~?S|follow)))", "Hawk Baker Follow-up"),
                    (r"(?i)((elk|hunt|236K))", "236K"),
                    (r"(?i)(((elk|hunt|236K)\s*(~?K|follow)))", "236K~K"),
                    (r"(?i)((hoof|stomp|214S))", "214S"),
                    (r"(?i)(((hoof|stomp|214S)\s*(~?S|follow)))", "214S~S"),
                    (r"(?i)((gazelle|dash))", "Gazelle Step"),
                    (r"(?i)((food|eat|grow|63214P))", "Still Growing"),
                    (r"(?i)((rtl|ride|lightning|632146HS?))", "632146H"),
                    (r"(?i)(((rtl|ride|lightning|632146HS?)\s*(~?HS?|follow)))", "632146HH"),
                    (r"(?i)((barrel|tyrant|236236P))", "236236P"),
                    (r"(?i)(((barrel|tyrant|236236P)\s*(~?\[?P\]?|follow)))", "236236P~]P["),
                ]
            },
            CharacterId::BAIKEN => {
                vec![
                    (r"(?i)((tatami|mat|gaeshi|236K))", "236K"),
                    (r"(?i)((j.?\s*(tatami|mat|gaeshi|236K)))", "j.236K"),
                    (r"(?i)((s\s*kabari|41236S))", "41236S"),
                    (r"(?i)((hs?\s*kabari|41236HS?))", "41236H"),
                    (r"(?i)(((hs?\s*kabari|41236HS?)\s*(follow|~?HS?)))", "41236H~H"),
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
                    (r"(?i)((fuujin|fujin|236HS?))", "Fuujin"),
                    (r"(?i)(((fuujin|fujin|236HS?)\s*P))", "Shin: Ichishiki"),
                    (r"(?i)(((fuujin|fujin|236HS?)\s*K))", "Issokutobi"),
                    (r"(?i)(((fuujin|fujin|236HS?)\s*S))", "Nagiha"),
                    (r"(?i)(((fuujin|fujin|236HS?)\s*HS?))", "Rin"),
                    (r"(?i)((kou|236S))", "Kou"),
                    (r"(?i)((issei|ougi|632146S?))", "Issei Ougi: Sai"),
                    (r"(?i)((kach|632146S))", "Kachoufuugetsu Kai"),
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
                    (r"(?i)((scalpel|thrust|41236K))", "Thrust"),
                    (r"(?i)((pull|back))", "Pull Back"),
                    (r"(?i)((golf|club|hole|41236K\s*HS?))", "Hole in One!"),
                    (r"(?i)((item|toss|236P|what))", "What Could This Be?"),
                    (r"(?i)((mmm|mix|236S))", "Mix Mix Mix"),
                    (r"(?i)((snip|command|grab|236HS?))", "Snip Snip Snip"),
                    (r"(?i)(((j.?)?love|j.?236P))", "j.236P"),
                    (r"(?i)(((j.?)?love|j.?236P)\s*(afro))", "j.236P (With Afro)"),
                    (r"(?i)(((p\s*crow)|214P))", "214P"),
                    (r"(?i)(((k\s*crow)|214K))", "214K"),
                    (r"(?i)(((s\s*crow)|214S))", "214S"),
                    (r"(?i)((bone|wheel|chair|reversal|632146HS?))", "Bone-crushing Excitement"),
                    (r"(?i)((236236P|item\s*super))", "W-W-What Could This Be?"),
                    (r"(?i)((236236236236P))", "W-W-W-W-W-W-W-W-W-What Could This Be?"),
                ]
            },
            CharacterId::AXL => {
                vec![
                    (r"(?i)((rensen|rensin|\[4\]6S|flash))", "Sickle Flash"),
                    (r"(?i)(((rensen|rensin|\[4\]6S|flash)\s*(8|up)))", "Soaring Chain Strike"),
                    (r"(?i)(((rensen|rensin|\[4\]6S|flash)\s*(2|down)))", "Spinning Chain Strike"),
                    (r"(?i)((cherry|((rensen|rensin|\[4\]6S|flash)\s*(s|bomb))))", "Winter Cherry"),
                    (r"(?i)((mantis|command|grab|41236HS?))", "Winter Mantis"),
                    (r"(?i)((rain|water|216S))", "Rainwater"),
                    (r"(?i)((snail|214HS?))", "214H"),
                    (r"(?i)((j.?\s*(snail|214HS?)))", "j.214H"),
                    (r"(?i)((bomber|j.?\s*236HS?))", "Axl Bomber"),
                    (r"(?i)((reversal|storm|236236HS?))", "Sickle Storm"),
                    (r"(?i)((one|vision|time\s*stop|632146P))", "632146P"),
                    (r"(?i)(((one|vision|time\s*stop|632146P)\s*activ))", "632146P Attack"),
                ]
            },
            CharacterId::POTEMKIN => {
                vec![
                    (r"(?i)((pb|grab|buster|360P|632146P))", "Potemkin Buster"),
                    (r"(?i)((heat|knuckle|hk|623HS?))", "Heat Knuckle"),
                    (r"(?i)((fmf|mf|236P|fist|mega))", "236P"),
                    (r"(?i)(((back|b)\s*(mega|fist|214P|mf)))", "214P"),
                    (r"(?i)(((k|kara)\s*(back|b)\s*(mega|fist|214P|mf)))", "2146K~P"),
                    (r"(?i)((slide|head|236S))", "Slide Head"),
                    (r"(?i)((hammer|fall|\[4\]6HS?|hf))", "Hammer Fall"),
                    (r"(?i)(((hammer|fall|\[4\]6HS?|hf)\s*(break|b)))", "Hammer Fall Break"),
                    (r"(?i)((flick|f.?d.?b.?))", "F.D.B."),
                    (r"(?i)(((flick|f.?d.?b.?)\s*charge))", "F.D.B. (Charged)"),
                    (r"(?i)(((flick|f.?d.?b.?)\s*(hit|reflect)))", "Reflect Projectile"),
                    (r"(?i)((garuda|214HS?))", "Garuda Impact"),
                    (r"(?i)((hpb|236236S|heavenly))", "Heavenly Potemkin Buster"),
                    (r"(?i)((giganter|kai|632146HS?))", "Giganter Kai"),
                    (r"(?i)((giganter|kai|632146HS?)\s*(barrier))", "Giganter Kai (Barrier)"),
                ]
            },
            CharacterId::RAMLETHAL => {
                vec![
                    (r"(?i)((623P|dp|dauro))", "Dauro"),
                    (r"(?i)((rekka|214P|erar))", "214P"),
                    (r"(?i)(((rekka|214P|erar)\s*2))", "214P 214P"),
                    (r"(?i)(((rekka|214P|erar)\s*3))", "214P 214P 214P"),
                    (r"(?i)((flip|214K|slido))", "214K"),
                    (r"(?i)((j.?\s*(flip|214K|slido)))", "j.214K"),
                    (r"(?i)((sword|throw|toss|bajoneto|236S))", "236S"),
                    (r"(?i)(((hs?)\s*(sword|throw|toss|bajoneto|236HS?)))", "236H"),
                    (r"(?i)((agresa|(j.?\s*214S)))", "Agresa Ordono"),
                    (r"(?i)((wind|wiper|214HS?))", "Sabrobato"),
                    (r"(?i)((calvados|63214HS?))", "Calvados"),
                    (r"(?i)((mortobato|reversal|236236S))", "Mortobato"),
                ]
            },
            CharacterId::GIO => {
                vec![
                    (r"(?i)((kick|214K|sep))", "Sepultura"),
                    (r"(?i)((drill|dog|236K|tro))", "Trovão"),
                    (r"(?i)((623S|dp|nascente))", "Sol Nascente"),
                    (r"(?i)((214S|sol|poente))", "214S"),
                    (r"(?i)((j.?\s*(214S|sol|poente)))", "j.214S"),
                    (r"(?i)((spin|reversal|63214HS?))", "Ventania"),
                    (r"(?i)((temp|air|(j.?\s*236236HS?)))", "Tempestade"),
                ]
            },
            CharacterId::GOLDLEWIS => {
                vec![
                    (r"(?i)((426(HS?)?))", "41236H"),
                    (r"(?i)((j.?\s*(426(HS?)?)))", "j.41236H"),

                    (r"(?i)((624(HS?)?))", "63214H"),
                    (r"(?i)((j.?\s*(624(HS?)?)))", "j.63214H"),

                    (r"(?i)((268(HS?)?))", "23698H"),
                    (r"(?i)((j.?\s*(268(HS?)?)))", "j.23698H"),

                    (r"(?i)((684(HS?)?))", "69874H"),
                    (r"(?i)((j.?\s*(684(HS?)?)))", "j.69874H"),

                    (r"(?i)((426(HS?)?))", "41236H"),
                    (r"(?i)((j.?\s*(426(HS?)?)))", "j.41236H"),

                    (r"(?i)((486(HS?)?))", "87412H"),
                    (r"(?i)((j.?\s*(486(HS?)?)))", "j.87412H"),

                    (r"(?i)((842(HS?)?))", "41236H"),
                    (r"(?i)((j.?\s*(842(HS?)?)))", "j.41236H"),

                    (r"(?i)((862(HS?)?))", "89632H"),
                    (r"(?i)((j.?\s*(862(HS?)?)))", "j.89632H"),

                    (r"(?i)((drone|214S))", "214S Level 1"),
                    (r"(?i)(((drone|214S)\s*(level|lv|lvl)?\s*2))", "214S Level 2"),
                    (r"(?i)(((drone|214S)\s*(level|lv|lvl)?\s*3))", "214S Level 3"),

                    (r"(?i)((gun|mini|skyfish|236S))", "236S Level 1"),
                    (r"(?i)(((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*2))", "236S Level 2"),
                    (r"(?i)(((gun|mini|skyfish|236S)\s*(level|lv|lvl)?\s*3))", "236S Level 3"),

                    (r"(?i)((dwts|down|system|reversal|360P|63214P))", "632146P"),
                    (r"(?i)((720P))", "720P"),
                    (r"(?i)((1080P))", "1080P"),

                    (r"(?i)((beam|burn|236236K))", "236236K Level 1"),
                    (r"(?i)(((beam|burn|236236K)\s*(level|lv|lvl)?\s*2))", "236236K Level 2"),
                    (r"(?i)(((beam|burn|236236K)\s*(level|lv|lvl)?\s*3))", "236236K Level 3"),
                ]
            },
            CharacterId::BRIDGET => {
                vec![
                    (r"(?i)(((236(S|(HS?))|yoyo|toss)))", "Stop and Dash (Hit on send)"),
                    (r"(?i)((roll|spin|214K))", "Rolling Movement"),
                    (r"(?i)((dp|star|ship|623P))", "Starship"),
                    (r"(?i)((car|kick|start|heart|236K))", "Kick Start My Heart"),
                    (r"(?i)((brake|((car|kick|start|heart|236K)\s*P)))", "Brake"),
                    (r"(?i)((shoot|((car|kick|start|heart|236K)\s*K)))", "Shoot"),
                    (r"(?i)((dive|(j.?\s*236K)))", "Roger Dive"),
                    (r"(?i)((command|grab|rock|baby|63214P))", "Rock the Baby"),
                    (r"(?i)((loop|632146S))", "Loop the Loop"),
                    (r"(?i)((motor|killing|632146HS?|return))", "Return of the Killing Machine"),
                ]
            },
            _ => vec![]
        }.into_iter().map(|(k, v)| (String::from(k), String::from(v))).collect::<Vec<(String, String)>>()
    }
}