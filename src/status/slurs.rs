const REPLACEMENT: &str = "meow";
// didn't think i'd have to filter names

pub fn filter(name: &str) -> String {
    name.replace("nigga", REPLACEMENT)
        .replace("NIGGA", REPLACEMENT)
        .replace("nigger", REPLACEMENT)
        .replace("NIGGER", REPLACEMENT)
        .replace("tranny", REPLACEMENT)
        .replace("TRANNY", REPLACEMENT)
        .replace("faggot", REPLACEMENT)
        .replace("FAGGOT", REPLACEMENT)
        .replace("fag", REPLACEMENT)
        .replace("FAG", REPLACEMENT)
}
