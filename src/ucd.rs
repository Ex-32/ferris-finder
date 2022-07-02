
#[derive(Debug, Clone)]
pub struct CharEntry {
    pub codepoint: u32,
    pub name: String,
    pub category: GeneralCategory,
    pub unicode_1_name: String,
}

impl CharEntry {

    pub fn from_ucd_line(ucd_line: &str) -> Option<CharEntry> {
        let data_entry = ucd_line.trim()
            .split(";")
            .collect::<Vec<&str>>();

        let codepoint = match data_entry.get(0) {
            Some(x) => match u32::from_str_radix(x, 16) {
                Ok(x) => x,
                Err(_) => return None
            }
            None => return None
        };
        let name = match data_entry.get(1) {
            Some(x) => (*x).to_owned(),
            None => return None
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
                _    => GeneralCategory::OtherNotAssigned,
            },
            None => return None
        };
        let unicode_1_name = match data_entry.get(10) {
            Some(x) => (*x).to_owned(),
            None => return None,
        };

        Some(CharEntry{codepoint, name, category, unicode_1_name})
    }
}


// ANCHOR General Category
// https://www.unicode.org/Public/5.1.0/ucd/UCD.html#General_Category_Values
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
