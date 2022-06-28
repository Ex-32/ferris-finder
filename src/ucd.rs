
#[derive(Debug, Clone)]
pub struct CharEntry {
    codepoint: u32,
    name: String,
    category: GeneralCategoryValue,
    combining_class: Option<CanonicalCombiningClassValue>
}

impl CharEntry {
    pub fn from_ucd_line(ucd_line: &str) -> Option<CharEntry> {
        let entry = ucd_line.trim().split(";").collect::<Vec<&str>>();

        let codepoint = match entry.get(0) {
            Some(x) => match u32::from_str_radix(x, 16) {
                Ok(x) => x,
                Err(_) => return None
            }
            None => return None
        };
        let name = match entry.get(1) {
            Some(x) => x,
            None => return None
        };
        let category = match entry.get(2) {
            Some(x) => match *x {
                "Lu" => GeneralCategoryValue::LetterUppercase,
                "Ll" => GeneralCategoryValue::LetterLowercase,
                "Lt" => GeneralCategoryValue::LetterTitlecase,
                "Lm" => GeneralCategoryValue::LetterModifier,
                "Lo" => GeneralCategoryValue::LetterOther,
                "Mn" => GeneralCategoryValue::MarkNonspacing,
                "Mc" => GeneralCategoryValue::MarkSpacingCombining,
                "Me" => GeneralCategoryValue::MarkEnclosing,
                "Nd" => GeneralCategoryValue::NumberDecimalDigit,
                "Nl" => GeneralCategoryValue::NumberLetter,
                "No" => GeneralCategoryValue::NumberOther,
                "Pc" => GeneralCategoryValue::PunctuationConnector,
                "Pd" => GeneralCategoryValue::PunctuationDash,
                "Ps" => GeneralCategoryValue::PunctuationOpen,
                "Pe" => GeneralCategoryValue::PunctuationClose,
                "Pi" => GeneralCategoryValue::PunctuationInitialQuote,
                "Pf" => GeneralCategoryValue::PunctuationFinalQuote,
                "Po" => GeneralCategoryValue::PunctuationOther,
                "Sm" => GeneralCategoryValue::SymbolMath,
                "Sc" => GeneralCategoryValue::SymbolCurrency,
                "Sk" => GeneralCategoryValue::SymbolModifier,
                "So" => GeneralCategoryValue::SymbolOther,
                "Zs" => GeneralCategoryValue::SeparatorSpace,
                "Zl" => GeneralCategoryValue::SeparatorLine,
                "Zp" => GeneralCategoryValue::SeparatorParagraph,
                "Cc" => GeneralCategoryValue::OtherControl,
                "Cf" => GeneralCategoryValue::OtherFormat,
                "Cs" => GeneralCategoryValue::OtherSurrogate,
                "Co" => GeneralCategoryValue::OtherPrivateUse,
                "Cn" => GeneralCategoryValue::OtherNotAssigned,
                _    => GeneralCategoryValue::OtherNotAssigned,
            },
            None => return None
        };
        let combining_class = match entry.get(3) {
            Some(x) => match x.parse::<u8>() {
                Ok(x) => CanonicalCombiningClassValue::from_u8(x),
                Err(_) => None
            },
            None => None
        };

        Some(CharEntry {
            codepoint: codepoint,
            name: (*name).to_owned(),
            category: category,
            combining_class: combining_class
        })
    }
}

// https://www.unicode.org/Public/5.1.0/ucd/UCD.html#General_Category_Values
#[derive(Debug, Clone)]
enum GeneralCategoryValue {
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

// https://www.unicode.org/Public/5.1.0/ucd/UCD.html#Canonical_Combining_Class_Values
#[repr(u8)]
#[derive(Debug, Clone)]
enum CanonicalCombiningClassValue {
    Spacing              = 0,
    Overlays             = 1,
    Nuktas               = 7,
    VoicingMarks         = 8,
    Virmas               = 9,
    StartOfFixedPosition = 10,
    EndOfFixedPosition   = 199,
    BellowLeftAttached   = 200,
    BellowAttached       = 202,
    BellowRightAttached  = 204,
    LeftAttached         = 208,
    RightAttached        = 210,
    AboveLeftAttached    = 212,
    AboveAttached        = 214,
    AboveRightAttached   = 216,
    BellowLeft           = 218,
    Bellow               = 220,
    BellowRight          = 222,
    Left                 = 224,
    Right                = 226,
    AboveLeft            = 228,
    Above                = 230,
    AboveRight           = 232,
    DoubleBellow         = 233,
    DoubleAbove          = 234,
    BellowIotaScript     = 240
}

impl CanonicalCombiningClassValue {

    fn from_u8(num: u8) -> Option<Self> {
        match num {
            0   => Some(CanonicalCombiningClassValue::Spacing),
            1   => Some(CanonicalCombiningClassValue::Overlays),
            7   => Some(CanonicalCombiningClassValue::Nuktas),
            8   => Some(CanonicalCombiningClassValue::VoicingMarks),
            9   => Some(CanonicalCombiningClassValue::Virmas),
            10  => Some(CanonicalCombiningClassValue::StartOfFixedPosition),
            199 => Some(CanonicalCombiningClassValue::EndOfFixedPosition),
            200 => Some(CanonicalCombiningClassValue::BellowLeftAttached),
            202 => Some(CanonicalCombiningClassValue::BellowAttached),
            204 => Some(CanonicalCombiningClassValue::BellowRightAttached),
            208 => Some(CanonicalCombiningClassValue::LeftAttached),
            210 => Some(CanonicalCombiningClassValue::RightAttached),
            212 => Some(CanonicalCombiningClassValue::AboveLeftAttached),
            214 => Some(CanonicalCombiningClassValue::AboveAttached),
            216 => Some(CanonicalCombiningClassValue::AboveRightAttached),
            218 => Some(CanonicalCombiningClassValue::BellowLeft),
            220 => Some(CanonicalCombiningClassValue::Bellow),
            222 => Some(CanonicalCombiningClassValue::BellowRight),
            224 => Some(CanonicalCombiningClassValue::Left),
            226 => Some(CanonicalCombiningClassValue::Right),
            228 => Some(CanonicalCombiningClassValue::AboveLeft),
            230 => Some(CanonicalCombiningClassValue::Above),
            232 => Some(CanonicalCombiningClassValue::AboveRight),
            233 => Some(CanonicalCombiningClassValue::DoubleBellow),
            234 => Some(CanonicalCombiningClassValue::DoubleAbove),
            240 => Some(CanonicalCombiningClassValue::BellowIotaScript),
            _   => None
        }

    }
}
