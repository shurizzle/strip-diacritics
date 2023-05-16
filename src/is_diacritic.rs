#[inline]
pub fn is_diacritic(ch: char) -> bool {
    ('\u{0300}'..='\u{036f}').contains(&ch)
}
