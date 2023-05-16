use std::borrow::Cow;

mod is_diacritic;
pub mod phf;
pub mod tables;

pub trait CharDiacriticExt {
    fn is_diacritic(&self) -> bool;

    fn strip_diacritics(&self) -> Option<&'static str>;
}

pub trait StrDiacriticExt {
    fn strip_diacritics(&self) -> Cow<str>;
}

impl CharDiacriticExt for char {
    fn is_diacritic(&self) -> bool {
        is_diacritic::is_diacritic(*self)
    }

    fn strip_diacritics(&self) -> Option<&'static str> {
        if self.is_diacritic() {
            return Some("");
        }
        crate::tables::DIACRITICS_MAPPING.get(*self).copied()
    }
}

fn next_diacritic(s: &str) -> Option<(&str, &'static str, &str)> {
    for (i, c) in s.char_indices() {
        if let Some(t) = c.strip_diacritics() {
            return Some((&s[..i], t, &s[(i + c.len_utf8())..]));
        }
    }
    None
}

impl StrDiacriticExt for str {
    fn strip_diacritics(&self) -> Cow<str> {
        let (mut buf, mut rest) = match next_diacritic(self) {
            Some((init, cont, rest)) => {
                let mut buf = String::with_capacity(init.len() + cont.len());
                buf.push_str(init);
                buf.push_str(cont);
                (buf, rest)
            }
            None => return Cow::Borrowed(self),
        };

        while !rest.is_empty() {
            rest = match next_diacritic(rest) {
                Some((init, cont, r)) => {
                    buf.push_str(init);
                    buf.push_str(cont);
                    r
                }
                None => {
                    buf.push_str(rest);
                    &rest[..rest.len()]
                }
            };
        }

        Cow::Owned(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cow_borrowed() {
        assert_eq!("aeiouy".strip_diacritics(), Cow::Borrowed("aeiouy"));
    }

    #[test]
    fn eu_diacritics() {
        assert_eq!("TÅRÖÄàèéìòù".strip_diacritics(), "TAROAaeeiou");
    }
}
