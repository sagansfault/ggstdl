# ggstdl
A webscraper/api for Guilty Gear Strive Dustloop  

There are lots of possible accepted character and move names in the regexes of `Character` and `Move`. This is all done for you using the method below. Inputs will work as well for move queries.

```rust
let data: GGSTDLData = ggstdl::load().await; // loading moves is done async as it scrapes the dustloop pages

let move_found: &Move = data.find_move("jack", "shoot")?; // Jack-O's 236K (minion shoot)
```
