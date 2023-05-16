use std::{collections::HashMap, fmt, str::FromStr};

use const_format::formatcp;

#[path = "../../src/is_diacritic.rs"]
mod is_diacritic;
use is_diacritic::is_diacritic;

const UNICODE_VERSION: (u32, u32, u32) = (15, 0, 0);
const UCD_URL: &str = formatcp!(
    "https://www.unicode.org/Public/{}.{}.{}/ucd/",
    UNICODE_VERSION.0,
    UNICODE_VERSION.1,
    UNICODE_VERSION.2
);

// Constants from Unicode 9.0.0 Section 3.12 Conjoining Jamo Behavior
// http://www.unicode.org/versions/Unicode9.0.0/ch03.pdf#M9.32468.Heading.310.Combining.Jamo.Behavior
const S_BASE: u32 = 0xAC00;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const S_COUNT: u32 = L_COUNT * V_COUNT * T_COUNT;

#[derive(Debug)]
struct StrError(&'static str);

impl fmt::Display for StrError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for StrError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    UpperCaseLetter,
    LowerCaseLetter,
    TitleCaseLetter,
    ModifierLetter,
    OtherLetter,
    NonspacingMark,
    SpacingMark,
    EnclosingMark,
    DecimalNumber,
    LetterNumber,
    OtherNumber,
    ConnectorPunctuation,
    DashPunctuation,
    OpenPunctuation,
    ClosePunctuation,
    InitialPunctuation,
    FinalPunctuation,
    OtherPunctuation,
    MathSymbol,
    CurrencySymbol,
    ModifierSymbol,
    OtherSymbol,
    SpaceSeparator,
    LineSeparator,
    ParagraphSeparator,
    Control,
    Format,
    Surrogate,
    PrivateUse,
    Unassigned,
}

impl FromStr for Category {
    type Err = StrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Category::*;
        match s {
            "Lu" => Ok(UpperCaseLetter),
            "Ll" => Ok(LowerCaseLetter),
            "Lt" => Ok(TitleCaseLetter),
            // "LC" => Ok(CasedLetter),
            "Lm" => Ok(ModifierLetter),
            "Lo" => Ok(OtherLetter),
            // "L" => Ok(Letter),
            "Mn" => Ok(NonspacingMark),
            "Mc" => Ok(SpacingMark),
            "Me" => Ok(EnclosingMark),
            // "M" => Ok(Mark),
            "Nd" => Ok(DecimalNumber),
            "Nl" => Ok(LetterNumber),
            "No" => Ok(OtherNumber),
            // "N" => Ok(Number),
            "Pc" => Ok(ConnectorPunctuation),
            "Pd" => Ok(DashPunctuation),
            "Ps" => Ok(OpenPunctuation),
            "Pe" => Ok(ClosePunctuation),
            "Pi" => Ok(InitialPunctuation),
            "Pf" => Ok(FinalPunctuation),
            "Po" => Ok(OtherPunctuation),
            // "P" => Ok(Punctuation),
            "Sm" => Ok(MathSymbol),
            "Sc" => Ok(CurrencySymbol),
            "Sk" => Ok(ModifierSymbol),
            "So" => Ok(OtherSymbol),
            // "S" => Ok(Symbol),
            "Zs" => Ok(SpaceSeparator),
            "Zl" => Ok(LineSeparator),
            "Zp" => Ok(ParagraphSeparator),
            // "Z" => Ok(Separator),
            "Cc" => Ok(Control),
            "Cf" => Ok(Format),
            "Cs" => Ok(Surrogate),
            "Co" => Ok(PrivateUse),
            "Cn" => Ok(Unassigned),
            // "C" => Ok(Other),
            _ => Err(StrError("Invalid Category")),
        }
    }
}

#[allow(dead_code)]
impl Category {
    pub fn is_cased_letter(&self) -> bool {
        use Category::*;
        matches!(self, UpperCaseLetter | LowerCaseLetter | TitleCaseLetter)
    }

    pub fn is_letter(&self) -> bool {
        use Category::*;
        matches!(
            self,
            UpperCaseLetter | LowerCaseLetter | TitleCaseLetter | ModifierLetter | OtherLetter
        )
    }

    pub fn is_mark(&self) -> bool {
        use Category::*;
        matches!(self, NonspacingMark | SpacingMark | EnclosingMark)
    }

    pub fn is_number(&self) -> bool {
        use Category::*;
        matches!(self, DecimalNumber | LetterNumber | OtherNumber)
    }

    pub fn is_punctuation(&self) -> bool {
        use Category::*;
        matches!(
            self,
            ConnectorPunctuation
                | DashPunctuation
                | OpenPunctuation
                | ClosePunctuation
                | InitialPunctuation
                | FinalPunctuation
                | OtherPunctuation
        )
    }

    pub fn is_symbol(&self) -> bool {
        use Category::*;
        matches!(
            self,
            MathSymbol | CurrencySymbol | ModifierSymbol | OtherSymbol
        )
    }

    pub fn is_separator(&self) -> bool {
        use Category::*;
        matches!(self, SpaceSeparator | LineSeparator | ParagraphSeparator)
    }

    pub fn is_other(&self) -> bool {
        use Category::*;
        matches!(self, Control | Format | Surrogate | PrivateUse | Unassigned)
    }
}

fn fetch<S: AsRef<str>>(file: S) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", UCD_URL, file.as_ref());
    Ok(ureq::get(&url).call()?.into_string()?)
}

#[allow(clippy::type_complexity)]
fn load_unicode_data() -> Result<
    (
        HashMap<u32, u8>,
        HashMap<u32, Vec<u32>>,
        HashMap<u32, Vec<u32>>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut combining_classes: HashMap<u32, u8> = HashMap::new();
    let mut compat_decomp: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut canon_decomp: HashMap<u32, Vec<u32>> = HashMap::new();

    for line in fetch("UnicodeData.txt")?.lines() {
        let mut it = line.split(';');
        let hack = unsafe { std::str::from_utf8_unchecked(&[]) };
        let (mut ch, mut category, mut cc, mut decomp) = (hack, hack, hack, hack);
        for i in 0..15 {
            let p = if let Some(p) = it.next() {
                p
            } else {
                return Err(StrError("Invalid line").into());
            };

            match i {
                0 => {
                    ch = p;
                }
                2 => {
                    category = p;
                }
                3 => {
                    cc = p;
                }
                5 => {
                    decomp = p;
                }
                _ => (),
            }
        }
        if it.next().is_some() {
            return Err(StrError("Invalid line").into());
        }
        let ch: u32 = match u32::from_str_radix(ch, 16) {
            Ok(ch) => ch,
            Err(err) => {
                return Err(err.into());
            }
        };

        if cc != "0" {
            combining_classes.insert(ch, cc.parse()?);
        }

        if let Some(decomp) = decomp.strip_prefix('<') {
            compat_decomp.insert(
                ch,
                decomp
                    .split_whitespace()
                    .skip(1)
                    .map(|c| u32::from_str_radix(c, 16))
                    .collect::<Result<_, _>>()?,
            );
        } else if !decomp.is_empty() {
            canon_decomp.insert(
                ch,
                decomp
                    .split_whitespace()
                    .map(|c| u32::from_str_radix(c, 16))
                    .collect::<Result<_, _>>()?,
            );
        }

        let category: Category = category.parse()?;

        assert_ne!(category, Category::Unassigned);
    }

    Ok((combining_classes, compat_decomp, canon_decomp))
}

#[allow(clippy::type_complexity)]
fn compute_fully_decomposed(
    canon_decomp: HashMap<u32, Vec<u32>>,
    compat_decomp: HashMap<u32, Vec<u32>>,
) -> Result<(HashMap<u32, Vec<u32>>, HashMap<u32, Vec<u32>>), Box<dyn std::error::Error>> {
    let mut canon = HashMap::new();
    let mut compat = HashMap::new();

    let end_codepoint = if let Some(max) = canon_decomp
        .keys()
        .copied()
        .max()
        .and_then(|m1| compat_decomp.keys().copied().max().map(|m2| m1.max(m2)))
    {
        max
    } else {
        return Ok((canon, compat));
    };

    fn __decompose(
        ch: u32,
        canon_decomp: &HashMap<u32, Vec<u32>>,
        compat_decomp: &HashMap<u32, Vec<u32>>,
        compatible: bool,
        res: &mut Vec<u32>,
    ) {
        if ch <= 0x7f {
            res.push(ch);
            return;
        }

        if let Some(decomp) = canon_decomp.get(&ch) {
            for &d in decomp {
                __decompose(d, canon_decomp, compat_decomp, compatible, res);
            }
            return;
        }

        if compatible {
            if let Some(chs) = compat_decomp.get(&ch) {
                for ch in chs.iter().copied() {
                    __decompose(ch, canon_decomp, compat_decomp, compatible, res)
                }
                return;
            }
        }

        res.push(ch);
    }

    fn _decompose(
        ch: u32,
        canon_decomp: &HashMap<u32, Vec<u32>>,
        compat_decomp: &HashMap<u32, Vec<u32>>,
        compatible: bool,
    ) -> Option<Vec<u32>> {
        let mut res = Vec::new();
        __decompose(ch, canon_decomp, compat_decomp, compatible, &mut res);
        if res.len() == 1 && unsafe { *res.get_unchecked(0) } == ch {
            None
        } else {
            Some(res)
        }
    }

    for ch in (0..=end_codepoint).filter(|ch| !(S_BASE..S_BASE + S_COUNT).contains(ch)) {
        {
            if let Some(d) = _decompose(ch, &canon_decomp, &compat_decomp, false) {
                canon.insert(ch, d);
            }
        }
        {
            if let Some(d) = _decompose(ch, &canon_decomp, &compat_decomp, true) {
                compat.insert(ch, d);
            }
        }
    }

    for (k, v) in canon.iter() {
        if compat.get(k).map_or(false, |v2| v == v2) {
            compat.remove(k);
        }
    }

    Ok((canon, compat))
}

fn sort_codepoints(chars: &[u32], combining_classes: &HashMap<u32, u8>) -> Vec<char> {
    let mut buf = Vec::<(u8, u32)>::new();
    for (class, ch) in chars
        .iter()
        .copied()
        .map(|c| (combining_classes.get(&c).copied().unwrap_or_default(), c))
    {
        if class == 0 {
            buf.sort_by_key(|x| x.0);
        }
        buf.push((class, ch));
    }
    buf.into_iter()
        .map(|x| unsafe { char::from_u32_unchecked(x.1) })
        .collect()
}

fn filter_diacritics<I: IntoIterator<Item = char>>(iter: I) -> Option<Vec<char>> {
    let mut ok = false;
    let mut buf = Vec::new();
    for ch in iter {
        if is_diacritic(ch) {
            ok = true;
        } else {
            buf.push(ch);
        }
    }

    if ok {
        Some(buf)
    } else {
        None
    }
}

fn codepoints_to_utf8(chars: &[char]) -> String {
    let mut buf = Vec::<u8>::with_capacity(chars.len() * 4);
    for ch in chars {
        unsafe {
            buf.reserve(4);
            let len = ch
                .encode_utf8(std::slice::from_raw_parts_mut(
                    buf.as_mut_ptr().add(buf.len()),
                    4,
                ))
                .len();
            buf.set_len(buf.len() + len);
        }
    }
    buf.shrink_to_fit();
    unsafe { String::from_utf8_unchecked(buf) }
}

fn add_mapping(
    src: HashMap<u32, Vec<u32>>,
    combining_classes: &HashMap<u32, u8>,
    dst: &mut HashMap<char, Box<str>>,
) {
    for (k, v) in src {
        let k = unsafe { char::from_u32_unchecked(k) };
        if !is_diacritic(k) {
            if let Some(chars) = filter_diacritics(sort_codepoints(&v, combining_classes)) {
                dst.insert(k, codepoints_to_utf8(&chars).into_boxed_str());
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mapping = {
        let (combining_classes, compat_decomp, canon_decomp) = load_unicode_data()?;
        let (canon_decomp, compat_decomp) = compute_fully_decomposed(canon_decomp, compat_decomp)?;
        let mut mapping = HashMap::<char, Box<str>>::new();
        add_mapping(canon_decomp, &combining_classes, &mut mapping);
        add_mapping(compat_decomp, &combining_classes, &mut mapping);
        mapping
    };
    let mut keys = Vec::with_capacity(mapping.len());
    let mut values = Vec::with_capacity(mapping.len());
    let (mut min, mut max): (Option<char>, Option<char>) = (None, None);
    for (k, v) in mapping {
        keys.push(k);
        values.push(v);
        min = Some(min.map_or(k, |min| min.min(k)));
        max = Some(max.map_or(k, |max| max.max(k)));
    }
    let range = min.expect("Empty data")..=max.expect("Empty data");
    let state = phf_generator::generate_hash(&keys);

    print!(
        "pub const DIACRITICS_MAPPING: crate::phf::CharMap<&'static str> = crate::phf::CharMap {{
    range: {:?},
    key: {:?},
    disps: &[",
        range, state.key
    );

    for &(d1, d2) in &state.disps {
        print!(
            "
        ({:?}, {:?}),",
            d1, d2
        );
    }

    print!(
        "
    ],
    entries: &[",
    );

    for &idx in &state.map {
        print!(
            "
        ({:?}, {:?}),",
            &keys[idx], &values[idx]
        );
    }

    println!(
        "
    ],
}};"
    );

    Ok(())
}
