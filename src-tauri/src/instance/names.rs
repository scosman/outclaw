use rand::seq::SliceRandom;

/// Curated list of positive/memorable adjectives (~100)
const ADJECTIVES: &[&str] = &[
    "cosmic",
    "stellar",
    "nebular",
    "lunar",
    "solar",
    "aurora",
    "nova",
    "comet",
    "swift",
    "rapid",
    "fleet",
    "nimble",
    "agile",
    "brisk",
    "zippy",
    "snappy",
    "golden",
    "silver",
    "crimson",
    "azure",
    "emerald",
    "amber",
    "coral",
    "ivory",
    "midnight",
    "dawn",
    "twilight",
    "eclipse",
    "zenith",
    "horizon",
    "equinox",
    "solstice",
    "electric",
    "magnetic",
    "quantum",
    "atomic",
    "ionic",
    "photon",
    "neutron",
    "cosmic",
    "phantom",
    "shadow",
    "mist",
    "frost",
    "storm",
    "thunder",
    "lightning",
    "blaze",
    "crystal",
    "diamond",
    "opal",
    "jade",
    "onyx",
    "pearl",
    "ruby",
    "sapphire",
    "ancient",
    "eternal",
    "primeval",
    "mythic",
    "legendary",
    "fabled",
    "arcane",
    "mystic",
    "gentle",
    "calm",
    "serene",
    "peaceful",
    "tranquil",
    "quiet",
    "still",
    "silent",
    "brave",
    "bold",
    "fierce",
    "noble",
    "valiant",
    "gallant",
    "heroic",
    "daring",
    "clever",
    "witty",
    "keen",
    "sharp",
    "bright",
    "brilliant",
    "luminous",
    "radiant",
    "curious",
    "wonder",
    "dream",
    "vision",
    "muse",
    "whim",
    "fancy",
    "caprice",
    "wild",
    "free",
    "roaming",
    "wandering",
    "drifting",
    "floating",
    "soaring",
    "gliding",
];

/// Curated list of memorable animals/creatures (~100)
const ANIMALS: &[&str] = &[
    "otter",
    "falcon",
    "raven",
    "buffalo",
    "panther",
    "mantis",
    "condor",
    "viper",
    "lynx",
    "osprey",
    "badger",
    "beaver",
    "cobra",
    "cougar",
    "coyote",
    "dingo",
    "eagle",
    "egret",
    "ferret",
    "fox",
    "gecko",
    "gibbon",
    "goshawk",
    "grouse",
    "harrier",
    "hawk",
    "heron",
    "hornet",
    "hummingbird",
    "iguana",
    "jackal",
    "jaguar",
    "kestrel",
    "kingfisher",
    "kite",
    "kookaburra",
    "lemur",
    "leopard",
    "lion",
    "lizard",
    "macaw",
    "marmot",
    "meerkat",
    "merlin",
    "mockingbird",
    "moose",
    "mongoose",
    "moorhen",
    "narwhal",
    "newt",
    "ocelot",
    "oriole",
    "osprey",
    "owl",
    "ox",
    "panda",
    "parrot",
    "peacock",
    "pelican",
    "penguin",
    "phoenix",
    "pika",
    "plover",
    "porcupine",
    "puffin",
    "python",
    "quail",
    "rabbit",
    "raccoon",
    "ram",
    "ratel",
    "raven",
    "red panda",
    "reindeer",
    "robin",
    "salamander",
    "sandpiper",
    "serval",
    "shark",
    "skua",
    "sloth",
    "snipe",
    "sparrow",
    "squirrel",
    "stork",
    "swallow",
    "swift",
    "tanager",
    "tapir",
    "termite",
    "tern",
    "thrush",
    "tiger",
    "toad",
    "toucan",
    "turkey",
    "turtle",
    "vulture",
    "wallaby",
    "warbler",
    "wasp",
    "weasel",
    "whale",
    "wombat",
    "woodpecker",
    "wren",
    "yak",
    "zebra",
];

/// Generate a random two-word name (adjective + animal)
///
/// # Arguments
/// * `existing_names` - List of names to avoid (for collision detection)
///
/// # Returns
/// A unique name like "Cosmic Otter" or "Swift Falcon"
pub fn generate_name(existing_names: &[String]) -> String {
    let mut rng = rand::thread_rng();

    // Try up to 10 times to generate a unique name
    for _ in 0..10 {
        let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"curious");
        let animal = ANIMALS.choose(&mut rng).unwrap_or(&"otter");

        // Capitalize first letter of each word
        let name = format!("{} {}", capitalize(adjective), capitalize(animal));

        if !existing_names.contains(&name) {
            return name;
        }
    }

    // If all attempts failed (highly unlikely), append a number
    let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"curious");
    let animal = ANIMALS.choose(&mut rng).unwrap_or(&"otter");
    let suffix: u32 = rand::random::<u32>() % 1000;
    format!(
        "{} {} {}",
        capitalize(adjective),
        capitalize(animal),
        suffix
    )
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_name() {
        let name = generate_name(&[]);
        assert!(!name.is_empty());

        // Should have exactly two words (or three with number suffix)
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert!(parts.len() >= 2 && parts.len() <= 3);

        // First letter should be capitalized
        assert!(name.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_generate_name_avoids_collision() {
        let existing = vec!["Cosmic Otter".to_string()];

        // Generate multiple names and verify none match the existing one
        for _ in 0..20 {
            let name = generate_name(&existing);
            assert_ne!(name, "Cosmic Otter");
        }
    }

    #[test]
    fn test_generate_name_with_many_collisions() {
        // Create many existing names to force suffix generation
        let mut existing: Vec<String> = Vec::new();
        for adj in ADJECTIVES.iter().take(20) {
            for animal in ANIMALS.iter().take(20) {
                existing.push(format!("{} {}", capitalize(adj), capitalize(animal)));
            }
        }

        // Should still generate a unique name (with suffix)
        let name = generate_name(&existing);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("otter"), "Otter");
        assert_eq!(capitalize("Otter"), "Otter");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }
}
