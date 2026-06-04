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

pub fn format(element: ElementRef) -> String {
    element
        .inner_html()
        .replace("<br>", "")
        .replace("<br>", "**")
        .replace("</br>", "**")
}
