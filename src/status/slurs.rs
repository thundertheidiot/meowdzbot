use regex::Regex;

const REPLACEMENT: &str = "🐈 censored 🐈";

pub fn filter(name: &str) -> String {
    let mut new_name = String::from(name);
    let regex =
        Regex::new(r"(?i)(nigger|nigga|faggot|fag|retard|tranny|troon|\+\d{7,15})").unwrap();

    while let Some(m) = regex.find(&new_name) {
        let start = m.start();
        let end = m.end();

        new_name = String::from(&new_name[..start]) + REPLACEMENT + &new_name[end..];
    }

    new_name
}

mod tests {
    use std::iter::zip;

    use super::*;

    #[test]
    fn slurs() {
        let test = [
            "hi nigga",
            "nigger",
            "CS2 TSHIRT, orders +51997696358",
            "fuck you retard",
        ];

        let expected = [
            "hi 🐈 censored 🐈",
            "🐈 censored 🐈",
            "CS2 TSHIRT, orders 🐈 censored 🐈",
            "fuck you 🐈 censored 🐈",
        ];

        for (t, e) in zip(test, expected) {
            assert!(filter(t) == e);
        }
    }
}
