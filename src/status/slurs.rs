const REPLACEMENT: &str = "meow";
// didn't think i'd have to filter names

pub fn filter(name: Box<str>) -> String {
    name.replace("nigga", REPLACEMENT)
        .replace("nigger", REPLACEMENT)
        .replace("tranny", REPLACEMENT)
        .replace("faggot", REPLACEMENT)
        .replace("fag", REPLACEMENT)
}
