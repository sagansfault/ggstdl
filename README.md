# ggstdl
A webscraper/api for Guilty Gear Strive Dustloop

```rust
let char_query = "jack";
let move_query = "shoot";

let data: Vec<Character> = ggstdl::load().await;

let character: Character = characters.iter().find(|c| c.regex.is_match(char_query.as_str()))?;
let move_found: Move = character.moves.iter().find(|m| m.regex.is_match(move_query.as_str()))?;
```
