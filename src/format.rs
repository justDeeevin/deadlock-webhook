use aho_corasick::AhoCorasick;
use scraper::ElementRef;

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

pub fn format_html(element: ElementRef) -> String {
    format_inner(
        AhoCorasick::new(["<br>", "<b>", "</b>"])
            .unwrap()
            .replace_all(&element.inner_html(), ["", "**", "**"].as_slice()),
    )
}

pub fn format_steam(contents: &str) -> String {
    format_inner(
        AhoCorasick::new(["\\[", "[p]", "[/p]", "[b]", "[/b]", "[u]", "[/u]"])
            .unwrap()
            .replace_all(contents, ["[", "", "\n", "**", "**", "__", "__"].as_slice()),
    )
}

// TODO: more advanced formatting
fn format_inner(md: String) -> String {
    md.replace("\n\n\n", "\n\n")
}
