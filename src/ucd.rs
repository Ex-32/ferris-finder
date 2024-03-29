const FERRIS: u32 = 0x1F980;

#[derive(Debug, Clone)]
pub struct CharEntry {
    pub codepoint: u32,
    pub name: Box<str>,
    pub category: GeneralCategory,
    pub unicode_1_name: Box<str>,
}

impl CharEntry {
    pub fn from_ucd_line(ucd_line: &str) -> Option<CharEntry> {
        let mut is_ferris = false;
        let data_entry = ucd_line.trim().split(';').collect::<Box<[&str]>>();

        let codepoint = match u32::from_str_radix(data_entry.first()?, 16) {
            Ok(x) => {
                if x == FERRIS {
                    is_ferris = true;
                }
                x
            }
            Err(_) => return None,
        };

        let name = if is_ferris {
            format!("{} (FERRIS)", data_entry.get(1)?).into()
        } else {
            (*(data_entry.get(1)?)).into()
        };

        let category = match data_entry.get(2) {
            Some(x) => match *x {
                "Lu" => GeneralCategory::LetterUppercase,
                "Ll" => GeneralCategory::LetterLowercase,
                "Lt" => GeneralCategory::LetterTitlecase,
                "Lm" => GeneralCategory::LetterModifier,
                "Lo" => GeneralCategory::LetterOther,
                "Mn" => GeneralCategory::MarkNonspacing,
                "Mc" => GeneralCategory::MarkSpacingCombining,
                "Me" => GeneralCategory::MarkEnclosing,
                "Nd" => GeneralCategory::NumberDecimalDigit,
                "Nl" => GeneralCategory::NumberLetter,
                "No" => GeneralCategory::NumberOther,
                "Pc" => GeneralCategory::PunctuationConnector,
                "Pd" => GeneralCategory::PunctuationDash,
                "Ps" => GeneralCategory::PunctuationOpen,
                "Pe" => GeneralCategory::PunctuationClose,
                "Pi" => GeneralCategory::PunctuationInitialQuote,
                "Pf" => GeneralCategory::PunctuationFinalQuote,
                "Po" => GeneralCategory::PunctuationOther,
                "Sm" => GeneralCategory::SymbolMath,
                "Sc" => GeneralCategory::SymbolCurrency,
                "Sk" => GeneralCategory::SymbolModifier,
                "So" => GeneralCategory::SymbolOther,
                "Zs" => GeneralCategory::SeparatorSpace,
                "Zl" => GeneralCategory::SeparatorLine,
                "Zp" => GeneralCategory::SeparatorParagraph,
                "Cc" => GeneralCategory::OtherControl,
                "Cf" => GeneralCategory::OtherFormat,
                "Cs" => GeneralCategory::OtherSurrogate,
                "Co" => GeneralCategory::OtherPrivateUse,
                "Cn" => GeneralCategory::OtherNotAssigned,
                _ => GeneralCategory::OtherNotAssigned,
            },
            None => return None,
        };
        let unicode_1_name = (*(data_entry.get(10)?)).into();

        Some(CharEntry {
            codepoint,
            name,
            category,
            unicode_1_name,
        })
    }

    pub fn fmt_codepoint(codepoint: u32) -> Box<str> {
        let code = format!("{:X}", codepoint);
        let mut padded = String::new();
        while (padded.len() + code.len()) < 4 {
            padded.push('0');
        }
        padded.push_str(&code);
        format!("U+{}", padded).into_boxed_str()
    }
}

// ANCHOR General Category
// LINK https://www.unicode.org/Public/5.1.0/ucd/UCD.html#General_Category_Values
#[derive(Debug, Clone, Copy)]
pub enum GeneralCategory {
    LetterUppercase,
    LetterLowercase,
    LetterTitlecase,
    LetterModifier,
    LetterOther,
    MarkNonspacing,
    MarkSpacingCombining,
    MarkEnclosing,
    NumberDecimalDigit,
    NumberLetter,
    NumberOther,
    PunctuationConnector,
    PunctuationDash,
    PunctuationOpen,
    PunctuationClose,
    PunctuationInitialQuote,
    PunctuationFinalQuote,
    PunctuationOther,
    SymbolMath,
    SymbolCurrency,
    SymbolModifier,
    SymbolOther,
    SeparatorSpace,
    SeparatorLine,
    SeparatorParagraph,
    OtherControl,
    OtherFormat,
    OtherSurrogate,
    OtherPrivateUse,
    OtherNotAssigned,
}
