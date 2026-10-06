use aho_corasick::AhoCorasick;
use regex::{Captures, Regex};

const HEROES: &[&str] = &[
    "Abrams",
    "Apollo",
    "Bebop",
    "Billy",
    "Calico",
    "Celeste",
    "Doorman",
    "Drifter",
    "Dynamo",
    "Graves",
    "Grey Talon",
    "Haze",
    "Holliday",
    "Infernus",
    "Ivy",
    "Kelvin",
    "Lady Geist",
    "Lash",
    "McGinnis",
    "Mina",
    "Mirage",
    "Mo & Krill",
    "Paige",
    "Paradox",
    "Pocket",
    "Rat King",
    "Rem",
    "Seven",
    "Shiv",
    "Silver",
    "Sinclair",
    "Venator",
    "Victor",
    "Vindicta",
    "Viscous",
    "Vyper",
    "Warden",
    "Wraith",
    "Yamato",
];

pub fn format_steam(contents: impl Into<String>) -> String {
    let link_re = Regex::new(r#"\[url=(.*?)\](.*)\[/url\]"#).unwrap();
    let image_re = Regex::new(r#"\[img.*\[/img\]"#).unwrap();
    let mut contents = contents.into();

    contents = image_re.replace_all(&contents, "").into_owned();
    contents = link_re
        .replace_all(&contents, |caps: &Captures| {
            let url = caps.get(1).unwrap().as_str();
            let text = caps.get(2).unwrap().as_str();
            if text == url {
                url.to_string()
            } else {
                format!("[{text}]({url})")
            }
        })
        .into_owned();
    contents = AhoCorasick::new(["[p]", "[/p]", "[b]", "[/b]", "[u]", "[/u]", "[i]", "[/i]"])
        .unwrap()
        .replace_all(
            &contents,
            ["", "\n", "**", "**", "__", "__", "_", "_"].as_slice(),
        );

    format_inner(&contents)
}

// TODO: more advanced formatting
fn format_inner(md: &str) -> String {
    md.replace("\n\n\n", "\n\n")
}
