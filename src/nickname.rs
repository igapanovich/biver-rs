const ADJECTIVES: &[&str] = &[
    "able", "acid", "aged", "airy", "bold", "bony", "boss", "brief", "brisk", "busy", "calm", "cheap", "chief", "civil", "clean", "clear", "close", "cold", "cool", "crisp",
    "curly", "damp", "dark", "dead", "dear", "deep", "dense", "dim", "dizzy", "dry", "dull", "dusty", "early", "east", "easy", "empty", "even", "evil", "fair", "fake", "far",
    "fast", "fat", "few", "fine", "firm", "fit", "flat", "fond", "foul", "free", "fresh", "full", "giant", "glad", "good", "grand", "gray", "great", "green", "gross", "half",
    "happy", "hard", "harsh", "hasty", "heavy", "high", "holy", "hot", "huge", "icy", "ideal", "idle", "iron", "jolly", "joint", "keen", "kind", "known", "lame", "large", "last",
    "late", "lazy", "lean", "least", "left", "legal", "level", "light", "limp", "live", "livid", "lone", "long", "loose", "lost", "loud", "low", "loyal", "lucid", "lucky", "mad",
    "main", "major", "meek", "mere", "messy", "mild", "mint", "misty", "mixed", "moral", "muddy", "mute", "naval", "near", "neat", "new", "next", "nice", "noble", "noted",
    "novel", "odd", "oily", "old", "open", "other", "oval", "pale", "past", "petty", "plain", "plump", "poor", "prime", "prior", "proud", "pure", "quick", "quiet", "rapid",
    "rare", "raw", "ready", "real", "red", "rich", "right", "rigid", "rocky", "rough", "round", "rowdy", "royal", "rural", "rusty", "sad", "safe", "salty", "sandy", "sane",
    "scaly", "scary", "sharp", "shiny", "short", "shy", "sick", "silky", "silly", "sleek", "slim", "slimy", "slow", "small", "smart", "smoky", "soft", "solid", "sorry", "sour",
    "spare", "spicy", "steep", "stiff", "stony", "sunny", "super", "sweet", "swift", "tall", "tame", "tan", "tart", "tasty", "tense", "thick", "thin", "tidy", "tiny", "tired",
    "total", "tough", "true", "twin", "ugly", "ultra", "unfit", "urban", "used", "usual", "vague", "valid", "vast", "vital", "vivid", "warm", "wary", "weak", "weary", "west",
    "wet", "white", "whole", "wide", "wild", "windy", "wise", "witty", "worn", "worst", "wrong", "young", "zany", "zero",
];

const NOUNS: &[&str] = &[
    "ace", "act", "age", "air", "ant", "ape", "arch", "area", "arm", "army", "art", "atom", "axe", "baby", "back", "ball", "band", "bank", "barn", "base", "bath", "bay", "bead",
    "beam", "bean", "bear", "beat", "bed", "bee", "belt", "bend", "bike", "bird", "bit", "blow", "boat", "body", "bolt", "bomb", "bone", "book", "boot", "boss", "bowl", "box",
    "boy", "bush", "cake", "calf", "call", "camp", "cape", "card", "cart", "case", "cash", "cast", "cat", "cave", "cell", "chip", "city", "clay", "clip", "clock", "club", "coal",
    "coat", "code", "coil", "coin", "cone", "copy", "cord", "core", "cork", "corn", "cost", "crab", "crew", "crop", "crow", "cup", "curb", "cure", "dad", "dame", "data", "date",
    "dawn", "day", "deal", "dean", "deck", "deer", "desk", "dial", "dice", "diet", "dime", "disk", "dock", "doll", "dome", "door", "dose", "dot", "dove", "down", "drag", "draw",
    "drop", "drum", "duck", "dune", "dust", "duty", "eagle", "earl", "edge", "eel", "face", "fact", "fall", "fame", "fang", "farm", "fawn", "fear", "feat", "feed", "feel", "fern",
    "file", "film", "fire", "fish", "fist", "flag", "flame", "flat", "flow", "foam", "fog", "fold", "font", "food", "foot", "fork", "form", "fort", "fowl", "fox", "fuel", "game",
    "gap", "gate", "gear", "gift", "girl", "goal", "goat", "gold", "golf", "gown", "grab", "grade", "gram", "grid", "grip", "gulf", "gull", "hall", "hand", "hare", "harm", "hawk",
    "head", "heap", "heat", "heel", "herb", "herd", "hero", "hill", "hint", "hire", "hole", "home", "hook", "hope", "horn", "host", "hour", "hunt", "idea", "inch", "iron", "isle",
    "item", "jade", "jail", "joke", "jury", "keep", "kent", "kick", "king", "kite", "knee", "lake", "lamb", "lamp", "land", "lark", "lawn", "lead", "leaf", "lens", "life", "lift",
    "limb", "line", "link", "lion", "list", "loaf", "loan", "lock", "loft", "log", "look", "loop", "lord", "love", "lung", "mall", "mane", "map", "mark", "mask", "mass", "mate",
    "math", "meal", "meat", "mile", "milk", "mill", "mind", "mint", "mist", "moat", "mode", "mole", "monk", "mood", "moon", "moss", "moth", "move", "myth", "nail", "name", "navy",
    "neck", "need", "nest", "news", "nose", "note", "pace", "pack", "page", "pain", "pair", "palm", "pan", "park", "part", "pass", "past", "path", "peak", "pear", "peg", "pen",
    "pest", "pick", "pier", "pile", "pine", "pipe", "plan", "play", "plot", "poem", "poet", "pole", "poll", "pond", "pony", "pool", "port", "post", "pull", "pump", "race", "rack",
    "raft", "rage", "rail", "rain", "rank", "rate", "ray", "reef", "rent", "rest", "rice", "ride", "ring", "rise", "risk", "road", "rock", "role", "roof", "room", "root", "rope",
    "rose", "row", "rule", "run", "sage", "sail", "sale", "salt", "sand", "seal", "seat", "seed", "self", "shed", "ship", "shop", "shot", "show", "side", "sign", "silk", "site",
    "size", "skin", "slab", "slip", "slot", "snow", "soap", "sock", "soil", "soul", "soup", "spot", "star", "step", "stew", "stop", "suit", "swan", "tale", "talk", "tank", "tape",
    "task", "taxi", "team", "tent", "term", "test", "text", "tide", "tile", "time", "toad", "toll", "tomb", "tone", "tool", "top", "tour", "town", "tray", "tree", "trek", "trim",
    "trip", "tub", "tube", "tune", "turf", "turn", "twig", "type", "unit", "user", "vase", "vast", "veil", "vein", "verb", "vest", "view", "vine", "volt", "vote", "wage", "walk",
    "wall", "wave", "wax", "web", "weed", "week", "well", "west", "whip", "wick", "wife", "wind", "wine", "wing", "wire", "wish", "wolf", "wood", "wool", "word", "work", "worm",
    "wrap", "yard", "year", "yolk", "zone",
];

pub fn new_nickname(random_value: u128) -> String {
    let adjective_index = random_value % (ADJECTIVES.len() as u128);
    let noun_index = random_value % (NOUNS.len() as u128);

    format!("{}-{}", ADJECTIVES[adjective_index as usize], NOUNS[noun_index as usize])
}
