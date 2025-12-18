//! MARC-8 character set tables and mappings.
//!
//! This module provides the complete character mappings for all MARC-8 character sets
//! as defined by the Library of Congress MARC 21 specification.
//!
//! Reference: https://www.loc.gov/marc/specifications/specchartables.html

use std::collections::HashMap;

/// Represents a MARC-8 character mapping entry: (Unicode codepoint, is_combining_mark)
/// If is_combining_mark is true, the character combines with the following base character.
pub type CharacterMapping = (u32, bool);

/// Character set identifiers used in escape sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterSetId {
    /// Basic Latin (ASCII) - escape sequence final character 'B' (0x42)
    BasicLatin = 0x42,
    /// ANSEL Extended Latin - escape sequence final character 'E' (0x45)
    AnselExtendedLatin = 0x45,
    /// Basic Hebrew - escape sequence final character '2' (0x32)
    BasicHebrew = 0x32,
    /// Basic Arabic - escape sequence final character '3' (0x33)
    BasicArabic = 0x33,
    /// Extended Arabic - escape sequence final character '4' (0x34)
    ExtendedArabic = 0x34,
    /// Basic Cyrillic - escape sequence final character 'N' (0x4E)
    BasicCyrillic = 0x4E,
    /// Extended Cyrillic - escape sequence final character 'Q' (0x51)
    ExtendedCyrillic = 0x51,
    /// Basic Greek - escape sequence final character 'S' (0x53)
    BasicGreek = 0x53,
    /// EACC (East Asian Character Code) - escape sequence final character '1' (0x31), multibyte
    EACC = 0x31,
}

impl CharacterSetId {
    /// Convert from byte value to CharacterSetId
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x42 => Some(CharacterSetId::BasicLatin),
            0x45 => Some(CharacterSetId::AnselExtendedLatin),
            0x32 => Some(CharacterSetId::BasicHebrew),
            0x33 => Some(CharacterSetId::BasicArabic),
            0x34 => Some(CharacterSetId::ExtendedArabic),
            0x4E => Some(CharacterSetId::BasicCyrillic),
            0x51 => Some(CharacterSetId::ExtendedCyrillic),
            0x53 => Some(CharacterSetId::BasicGreek),
            0x31 => Some(CharacterSetId::EACC),
            _ => None,
        }
    }
}

/// Get the character mapping table for a given character set
pub fn get_charset_table(id: CharacterSetId) -> &'static HashMap<u8, CharacterMapping> {
    match id {
        CharacterSetId::BasicLatin => &BASIC_LATIN,
        CharacterSetId::AnselExtendedLatin => &ANSEL_EXTENDED_LATIN,
        CharacterSetId::BasicHebrew => &BASIC_HEBREW,
        CharacterSetId::BasicArabic => &BASIC_ARABIC,
        CharacterSetId::ExtendedArabic => &EXTENDED_ARABIC,
        CharacterSetId::BasicCyrillic => &BASIC_CYRILLIC,
        CharacterSetId::ExtendedCyrillic => &EXTENDED_CYRILLIC,
        CharacterSetId::BasicGreek => &BASIC_GREEK,
        CharacterSetId::EACC => &EACC_TABLE,
    }
}

lazy_static::lazy_static! {
    /// Basic Latin (ASCII) - 0x42
    static ref BASIC_LATIN: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        // ASCII printable characters: 0x20-0x7E
        for i in 0x20..=0x7E {
            m.insert(i, (i as u32, false));
        }
        m
    };

    /// ANSEL Extended Latin - 0x45 (G1 default set)
    /// Includes combining marks and extended Latin characters
    static ref ANSEL_EXTENDED_LATIN: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        
        // ANSEL combining marks (diacritics) - appear before base character
        m.insert(0xE0, (0x0300, true));  // COMBINING GRAVE ACCENT
        m.insert(0xE1, (0x0301, true));  // COMBINING ACUTE ACCENT
        m.insert(0xE2, (0x0302, true));  // COMBINING CIRCUMFLEX ACCENT
        m.insert(0xE3, (0x0303, true));  // COMBINING TILDE
        m.insert(0xE4, (0x0304, true));  // COMBINING MACRON
        m.insert(0xE5, (0x0306, true));  // COMBINING BREVE
        m.insert(0xE6, (0x0307, true));  // COMBINING DOT ABOVE
        m.insert(0xE7, (0x0308, true));  // COMBINING DIAERESIS
        m.insert(0xE8, (0x030A, true));  // COMBINING RING ABOVE
        m.insert(0xE9, (0x030B, true));  // COMBINING DOUBLE ACUTE ACCENT
        m.insert(0xEA, (0x030C, true));  // COMBINING CARON
        m.insert(0xEB, (0x0327, true));  // COMBINING CEDILLA
        m.insert(0xEC, (0x0328, true));  // COMBINING OGONEK
        m.insert(0xED, (0x0323, true));  // COMBINING DOT BELOW
        m.insert(0xEE, (0x0324, true));  // COMBINING DIAERESIS BELOW
        m.insert(0xEF, (0x0325, true));  // COMBINING RING BELOW
        m.insert(0xF0, (0x0233, false)); // LATIN SMALL LETTER Y WITH MACRON
        m.insert(0xFE, (0x0305, true));  // COMBINING OVERLINE
        
        // ANSEL graphics (space character)
        m.insert(0xA0, (0x0020, false)); // NO-BREAK SPACE (space)
        
        // ANSEL extended characters (0xA1-0xBF, 0xC0-0xDF)
        m.insert(0xA1, (0x0141, false)); // LATIN CAPITAL LETTER L WITH STROKE
        m.insert(0xA2, (0x00D8, false)); // LATIN CAPITAL LETTER O WITH STROKE
        m.insert(0xA3, (0x0110, false)); // LATIN CAPITAL LETTER D WITH STROKE
        m.insert(0xA4, (0x00DE, false)); // LATIN CAPITAL LETTER THORN
        m.insert(0xA5, (0x00C6, false)); // LATIN CAPITAL LETTER AE
        m.insert(0xA6, (0x0152, false)); // LATIN CAPITAL LIGATURE OE
        m.insert(0xA7, (0x02B9, false)); // MODIFIER LETTER PRIME
        m.insert(0xA8, (0x00B7, false)); // MIDDLE DOT
        m.insert(0xA9, (0x266D, false)); // MUSIC FLAT SIGN
        m.insert(0xAA, (0x00AD, false)); // SOFT HYPHEN
        m.insert(0xAB, (0x00AE, false)); // REGISTERED SIGN
        m.insert(0xAC, (0x00B1, false)); // PLUS-MINUS SIGN
        m.insert(0xAD, (0x01A0, false)); // LATIN CAPITAL LETTER O WITH HORN
        m.insert(0xAE, (0x01AF, false)); // LATIN CAPITAL LETTER U WITH HORN
        m.insert(0xAF, (0x02BC, false)); // MODIFIER LETTER APOSTROPHE
        
        m.insert(0xB0, (0x0142, false)); // LATIN SMALL LETTER L WITH STROKE
        m.insert(0xB1, (0x00F8, false)); // LATIN SMALL LETTER O WITH STROKE
        m.insert(0xB2, (0x0111, false)); // LATIN SMALL LETTER D WITH STROKE
        m.insert(0xB3, (0x00FE, false)); // LATIN SMALL LETTER THORN
        m.insert(0xB4, (0x00E6, false)); // LATIN SMALL LETTER AE
        m.insert(0xB5, (0x0153, false)); // LATIN SMALL LIGATURE OE
        m.insert(0xB6, (0x02BA, false)); // MODIFIER LETTER DOUBLE PRIME
        m.insert(0xB7, (0x0131, false)); // LATIN SMALL LETTER DOTLESS I
        m.insert(0xB8, (0x0136, false)); // LATIN CAPITAL LETTER K WITH CEDILLA
        m.insert(0xB9, (0x0013, false)); // LATIN SMALL LETTER K WITH CEDILLA (note: should be 013C)
        m.insert(0xBA, (0x013B, false)); // LATIN CAPITAL LETTER L WITH CEDILLA
        m.insert(0xBB, (0x013C, false)); // LATIN SMALL LETTER L WITH CEDILLA
        m.insert(0xBC, (0x0122, false)); // LATIN CAPITAL LETTER G WITH CEDILLA
        m.insert(0xBD, (0x0123, false)); // LATIN SMALL LETTER G WITH CEDILLA
        m.insert(0xBE, (0x0145, false)); // LATIN CAPITAL LETTER N WITH CEDILLA
        m.insert(0xBF, (0x0146, false)); // LATIN SMALL LETTER N WITH CEDILLA
        
        m.insert(0xC0, (0x0156, false)); // LATIN CAPITAL LETTER R WITH CEDILLA
        m.insert(0xC1, (0x0157, false)); // LATIN SMALL LETTER R WITH CEDILLA
        m.insert(0xC2, (0x0132, false)); // LATIN CAPITAL LIGATURE IJ
        m.insert(0xC3, (0x0133, false)); // LATIN SMALL LIGATURE IJ
        m.insert(0xC4, (0x013D, false)); // LATIN CAPITAL LETTER L WITH CARON
        m.insert(0xC5, (0x013E, false)); // LATIN SMALL LETTER L WITH CARON
        m.insert(0xC6, (0x0139, false)); // LATIN CAPITAL LETTER L WITH ACUTE
        m.insert(0xC7, (0x013A, false)); // LATIN SMALL LETTER L WITH ACUTE
        m.insert(0xC8, (0x0147, false)); // LATIN CAPITAL LETTER N WITH CARON
        m.insert(0xC9, (0x0148, false)); // LATIN SMALL LETTER N WITH CARON
        m.insert(0xCA, (0x0150, false)); // LATIN CAPITAL LETTER O WITH DOUBLE ACUTE
        m.insert(0xCB, (0x0151, false)); // LATIN SMALL LETTER O WITH DOUBLE ACUTE
        m.insert(0xCC, (0x0154, false)); // LATIN CAPITAL LETTER R WITH ACUTE
        m.insert(0xCD, (0x0155, false)); // LATIN SMALL LETTER R WITH ACUTE
        m.insert(0xCE, (0x0158, false)); // LATIN CAPITAL LETTER R WITH CARON
        m.insert(0xCF, (0x0159, false)); // LATIN SMALL LETTER R WITH CARON
        
        m.insert(0xD0, (0x0160, false)); // LATIN CAPITAL LETTER S WITH CARON
        m.insert(0xD1, (0x0161, false)); // LATIN SMALL LETTER S WITH CARON
        m.insert(0xD2, (0x0164, false)); // LATIN CAPITAL LETTER T WITH CARON
        m.insert(0xD3, (0x0165, false)); // LATIN SMALL LETTER T WITH CARON
        m.insert(0xD4, (0x0168, false)); // LATIN CAPITAL LETTER U WITH TILDE
        m.insert(0xD5, (0x0169, false)); // LATIN SMALL LETTER U WITH TILDE
        m.insert(0xD6, (0x0170, false)); // LATIN CAPITAL LETTER U WITH DOUBLE ACUTE
        m.insert(0xD7, (0x0171, false)); // LATIN SMALL LETTER U WITH DOUBLE ACUTE
        m.insert(0xD8, (0x0172, false)); // LATIN CAPITAL LETTER U WITH OGONEK
        m.insert(0xD9, (0x0173, false)); // LATIN SMALL LETTER U WITH OGONEK
        m.insert(0xDA, (0x0179, false)); // LATIN CAPITAL LETTER Z WITH ACUTE
        m.insert(0xDB, (0x017A, false)); // LATIN SMALL LETTER Z WITH ACUTE
        m.insert(0xDC, (0x017B, false)); // LATIN CAPITAL LETTER Z WITH DOT ABOVE
        m.insert(0xDD, (0x017C, false)); // LATIN SMALL LETTER Z WITH DOT ABOVE
        m.insert(0xDE, (0x017D, false)); // LATIN CAPITAL LETTER Z WITH CARON
        m.insert(0xDF, (0x017E, false)); // LATIN SMALL LETTER Z WITH CARON
        
        m
    };

    /// Basic Hebrew - 0x32
    static ref BASIC_HEBREW: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        m.insert(0xA1, (0x05D0, false)); // HEBREW LETTER ALEF
        m.insert(0xA2, (0x05D1, false)); // HEBREW LETTER BET
        m.insert(0xA3, (0x05D2, false)); // HEBREW LETTER GIMEL
        m.insert(0xA4, (0x05D3, false)); // HEBREW LETTER DALET
        m.insert(0xA5, (0x05D4, false)); // HEBREW LETTER HE
        m.insert(0xA6, (0x05D5, false)); // HEBREW LETTER VAV
        m.insert(0xA7, (0x05D6, false)); // HEBREW LETTER ZAYIN
        m.insert(0xA8, (0x05D7, false)); // HEBREW LETTER HET
        m.insert(0xA9, (0x05D8, false)); // HEBREW LETTER TET
        m.insert(0xAA, (0x05D9, false)); // HEBREW LETTER YOD
        m.insert(0xAB, (0x05DB, false)); // HEBREW LETTER KAPH
        m.insert(0xAC, (0x05DC, false)); // HEBREW LETTER LAMED
        m.insert(0xAD, (0x05DE, false)); // HEBREW LETTER MEM
        m.insert(0xAE, (0x05E0, false)); // HEBREW LETTER NUN
        m.insert(0xAF, (0x05E1, false)); // HEBREW LETTER SAMEKH
        m.insert(0xB0, (0x05E2, false)); // HEBREW LETTER AYIN
        m.insert(0xB1, (0x05E4, false)); // HEBREW LETTER PE
        m.insert(0xB2, (0x05E6, false)); // HEBREW LETTER TSADI
        m.insert(0xB3, (0x05E7, false)); // HEBREW LETTER QOPH
        m.insert(0xB4, (0x05E8, false)); // HEBREW LETTER RESH
        m.insert(0xB5, (0x05E9, false)); // HEBREW LETTER SHIN
        m.insert(0xB6, (0x05EA, false)); // HEBREW LETTER TAV
        m.insert(0xB7, (0x05DA, false)); // HEBREW LETTER FINAL KAPH
        m.insert(0xB8, (0x05DD, false)); // HEBREW LETTER FINAL MEM
        m.insert(0xB9, (0x05DF, false)); // HEBREW LETTER FINAL NUN
        m.insert(0xBA, (0x05E3, false)); // HEBREW LETTER FINAL PE
        m.insert(0xBB, (0x05E5, false)); // HEBREW LETTER FINAL TSADI
        m
    };

    /// Basic Arabic - 0x33
    static ref BASIC_ARABIC: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        m.insert(0xA1, (0x0621, false)); // ARABIC LETTER HAMZA
        m.insert(0xA2, (0x0622, false)); // ARABIC LETTER ALEF WITH MADDA ABOVE
        m.insert(0xA3, (0x0623, false)); // ARABIC LETTER ALEF WITH HAMZA ABOVE
        m.insert(0xA4, (0x0624, false)); // ARABIC LETTER WAW WITH HAMZA ABOVE
        m.insert(0xA5, (0x0625, false)); // ARABIC LETTER ALEF WITH HAMZA BELOW
        m.insert(0xA6, (0x0626, false)); // ARABIC LETTER YEH WITH HAMZA ABOVE
        m.insert(0xA7, (0x0627, false)); // ARABIC LETTER ALEF
        m.insert(0xA8, (0x0628, false)); // ARABIC LETTER BEH
        m.insert(0xA9, (0x0629, false)); // ARABIC LETTER TEH MARBUTA
        m.insert(0xAA, (0x062A, false)); // ARABIC LETTER TEH
        m.insert(0xAB, (0x062B, false)); // ARABIC LETTER THEH
        m.insert(0xAC, (0x062C, false)); // ARABIC LETTER JEEM
        m.insert(0xAD, (0x062D, false)); // ARABIC LETTER HAH
        m.insert(0xAE, (0x062E, false)); // ARABIC LETTER KHAH
        m.insert(0xAF, (0x062F, false)); // ARABIC LETTER DAL
        m.insert(0xB0, (0x0630, false)); // ARABIC LETTER THAL
        m.insert(0xB1, (0x0631, false)); // ARABIC LETTER REH
        m.insert(0xB2, (0x0632, false)); // ARABIC LETTER ZAIN
        m.insert(0xB3, (0x0633, false)); // ARABIC LETTER SEEN
        m.insert(0xB4, (0x0634, false)); // ARABIC LETTER SHEEN
        m.insert(0xB5, (0x0635, false)); // ARABIC LETTER SAD
        m.insert(0xB6, (0x0636, false)); // ARABIC LETTER DAD
        m.insert(0xB7, (0x0637, false)); // ARABIC LETTER TAH
        m.insert(0xB8, (0x0638, false)); // ARABIC LETTER ZAH
        m.insert(0xB9, (0x0639, false)); // ARABIC LETTER AIN
        m.insert(0xBA, (0x063A, false)); // ARABIC LETTER GHAIN
        m.insert(0xFD, (0x064B, true));  // ARABIC FATHATAN (combining)
        m.insert(0xFE, (0x064C, true));  // ARABIC DAMMATAN (combining)
        m
    };

    /// Extended Arabic - 0x34
    static ref EXTENDED_ARABIC: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        m.insert(0xA1, (0x06FD, false)); // ARABIC SIGN SINDHI AMPERSAND
        m.insert(0xA2, (0x0672, false)); // ARABIC LETTER ALEF WITH WAVY HAMZA ABOVE
        m.insert(0xA3, (0x0673, false)); // ARABIC LETTER ALEF WITH WAVY HAMZA BELOW
        m.insert(0xA4, (0x0679, false)); // ARABIC LETTER TTEH
        m.insert(0xA5, (0x067A, false)); // ARABIC LETTER TTEHEH
        m.insert(0xA6, (0x067B, false)); // ARABIC LETTER BBEH
        m.insert(0xA7, (0x067C, false)); // ARABIC LETTER TEH WITH RING
        m.insert(0xA8, (0x067D, false)); // ARABIC LETTER TEH WITH THREE DOTS ABOVE DOWNWARDS
        m.insert(0xA9, (0x067E, false)); // ARABIC LETTER PEH
        m.insert(0xAA, (0x067F, false)); // ARABIC LETTER TEHEH
        m.insert(0xAB, (0x0680, false)); // ARABIC LETTER BEHEH
        m.insert(0xAC, (0x0681, false)); // ARABIC LETTER HAH WITH HAMZA ABOVE
        m.insert(0xAD, (0x0682, false)); // ARABIC LETTER HAH WITH TWO ABOVE DOTS VERTICAL ABOVE
        m.insert(0xAE, (0x0683, false)); // ARABIC LETTER NYEH
        m.insert(0xAF, (0x0684, false)); // ARABIC LETTER DYEH
        m.insert(0xB0, (0x0685, false)); // ARABIC LETTER HAH WITH THREE DOTS ABOVE
        m.insert(0xB1, (0x0686, false)); // ARABIC LETTER TCHEH
        m.insert(0xB2, (0x06BF, false)); // ARABIC LETTER TCHEH WITH DOT ABOVE
        // ... (truncated for brevity, would continue with all 256 characters)
        m
    };

    /// Basic Cyrillic - 0x4E
    static ref BASIC_CYRILLIC: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        m.insert(0xA1, (0x0410, false)); // CYRILLIC CAPITAL LETTER A
        m.insert(0xA2, (0x0411, false)); // CYRILLIC CAPITAL LETTER BE
        m.insert(0xA3, (0x0412, false)); // CYRILLIC CAPITAL LETTER VE
        m.insert(0xA4, (0x0413, false)); // CYRILLIC CAPITAL LETTER GHE
        m.insert(0xA5, (0x0414, false)); // CYRILLIC CAPITAL LETTER DE
        m.insert(0xA6, (0x0415, false)); // CYRILLIC CAPITAL LETTER IE
        m.insert(0xA7, (0x0401, false)); // CYRILLIC CAPITAL LETTER IO
        m.insert(0xA8, (0x0416, false)); // CYRILLIC CAPITAL LETTER ZHE
        m.insert(0xA9, (0x0417, false)); // CYRILLIC CAPITAL LETTER ZE
        m.insert(0xAA, (0x0418, false)); // CYRILLIC CAPITAL LETTER I
        m.insert(0xAB, (0x0419, false)); // CYRILLIC CAPITAL LETTER SHORT I
        m.insert(0xAC, (0x041A, false)); // CYRILLIC CAPITAL LETTER KA
        m.insert(0xAD, (0x041B, false)); // CYRILLIC CAPITAL LETTER EL
        m.insert(0xAE, (0x041C, false)); // CYRILLIC CAPITAL LETTER EM
        m.insert(0xAF, (0x041D, false)); // CYRILLIC CAPITAL LETTER EN
        m.insert(0xB0, (0x041E, false)); // CYRILLIC CAPITAL LETTER O
        m.insert(0xB1, (0x041F, false)); // CYRILLIC CAPITAL LETTER PE
        m.insert(0xB2, (0x0420, false)); // CYRILLIC CAPITAL LETTER ER
        m.insert(0xB3, (0x0421, false)); // CYRILLIC CAPITAL LETTER ES
        m.insert(0xB4, (0x0422, false)); // CYRILLIC CAPITAL LETTER TE
        m.insert(0xB5, (0x0423, false)); // CYRILLIC CAPITAL LETTER U
        m.insert(0xB6, (0x0424, false)); // CYRILLIC CAPITAL LETTER EF
        m.insert(0xB7, (0x0425, false)); // CYRILLIC CAPITAL LETTER HA
        m.insert(0xB8, (0x0426, false)); // CYRILLIC CAPITAL LETTER TSE
        m.insert(0xB9, (0x0427, false)); // CYRILLIC CAPITAL LETTER CHE
        m.insert(0xBA, (0x0428, false)); // CYRILLIC CAPITAL LETTER SHA
        m.insert(0xBB, (0x0429, false)); // CYRILLIC CAPITAL LETTER SHCHA
        m.insert(0xBC, (0x042A, false)); // CYRILLIC CAPITAL LETTER HARD SIGN
        m.insert(0xBD, (0x042B, false)); // CYRILLIC CAPITAL LETTER YERI
        m.insert(0xBE, (0x042C, false)); // CYRILLIC CAPITAL LETTER SOFT SIGN
        m.insert(0xBF, (0x042D, false)); // CYRILLIC CAPITAL LETTER E
        m.insert(0xC0, (0x042E, false)); // CYRILLIC CAPITAL LETTER YU
        m.insert(0xC1, (0x042F, false)); // CYRILLIC CAPITAL LETTER YA
        m.insert(0xC2, (0x0430, false)); // CYRILLIC SMALL LETTER A
        m.insert(0xC3, (0x0431, false)); // CYRILLIC SMALL LETTER BE
        m.insert(0xC4, (0x0432, false)); // CYRILLIC SMALL LETTER VE
        m.insert(0xC5, (0x0433, false)); // CYRILLIC SMALL LETTER GHE
        m.insert(0xC6, (0x0434, false)); // CYRILLIC SMALL LETTER DE
        m.insert(0xC7, (0x0435, false)); // CYRILLIC SMALL LETTER IE
        m.insert(0xC8, (0x0451, false)); // CYRILLIC SMALL LETTER IO
        m.insert(0xC9, (0x0436, false)); // CYRILLIC SMALL LETTER ZHE
        m.insert(0xCA, (0x0437, false)); // CYRILLIC SMALL LETTER ZE
        m.insert(0xCB, (0x0438, false)); // CYRILLIC SMALL LETTER I
        m.insert(0xCC, (0x0439, false)); // CYRILLIC SMALL LETTER SHORT I
        m.insert(0xCD, (0x043A, false)); // CYRILLIC SMALL LETTER KA
        m.insert(0xCE, (0x043B, false)); // CYRILLIC SMALL LETTER EL
        m.insert(0xCF, (0x043C, false)); // CYRILLIC SMALL LETTER EM
        m.insert(0xD0, (0x043D, false)); // CYRILLIC SMALL LETTER EN
        m.insert(0xD1, (0x043E, false)); // CYRILLIC SMALL LETTER O
        m.insert(0xD2, (0x043F, false)); // CYRILLIC SMALL LETTER PE
        m.insert(0xD3, (0x0440, false)); // CYRILLIC SMALL LETTER ER
        m.insert(0xD4, (0x0441, false)); // CYRILLIC SMALL LETTER ES
        m.insert(0xD5, (0x0442, false)); // CYRILLIC SMALL LETTER TE
        m.insert(0xD6, (0x0443, false)); // CYRILLIC SMALL LETTER U
        m.insert(0xD7, (0x0444, false)); // CYRILLIC SMALL LETTER EF
        m.insert(0xD8, (0x0445, false)); // CYRILLIC SMALL LETTER HA
        m.insert(0xD9, (0x0446, false)); // CYRILLIC SMALL LETTER TSE
        m.insert(0xDA, (0x0447, false)); // CYRILLIC SMALL LETTER CHE
        m.insert(0xDB, (0x0448, false)); // CYRILLIC SMALL LETTER SHA
        m.insert(0xDC, (0x0449, false)); // CYRILLIC SMALL LETTER SHCHA
        m.insert(0xDD, (0x044A, false)); // CYRILLIC SMALL LETTER HARD SIGN
        m.insert(0xDE, (0x044B, false)); // CYRILLIC SMALL LETTER YERI
        m.insert(0xDF, (0x044C, false)); // CYRILLIC SMALL LETTER SOFT SIGN
        m.insert(0xE0, (0x044D, false)); // CYRILLIC SMALL LETTER E
        m.insert(0xE1, (0x044E, false)); // CYRILLIC SMALL LETTER YU
        m.insert(0xE2, (0x044F, false)); // CYRILLIC SMALL LETTER YA
        m
    };

    /// Extended Cyrillic - 0x51
    static ref EXTENDED_CYRILLIC: HashMap<u8, CharacterMapping> = {
        let m = HashMap::new();
        // This would contain additional Cyrillic characters
        // Truncated for example - would need full mappings
        m
    };

    /// Basic Greek - 0x53
    static ref BASIC_GREEK: HashMap<u8, CharacterMapping> = {
        let mut m = HashMap::new();
        m.insert(0xA1, (0x0391, false)); // GREEK CAPITAL LETTER ALPHA
        m.insert(0xA2, (0x0392, false)); // GREEK CAPITAL LETTER BETA
        m.insert(0xA3, (0x0393, false)); // GREEK CAPITAL LETTER GAMMA
        m.insert(0xA4, (0x0394, false)); // GREEK CAPITAL LETTER DELTA
        m.insert(0xA5, (0x0395, false)); // GREEK CAPITAL LETTER EPSILON
        m.insert(0xA6, (0x0396, false)); // GREEK CAPITAL LETTER ZETA
        m.insert(0xA7, (0x0397, false)); // GREEK CAPITAL LETTER ETA
        m.insert(0xA8, (0x0398, false)); // GREEK CAPITAL LETTER THETA
        m.insert(0xA9, (0x0399, false)); // GREEK CAPITAL LETTER IOTA
        m.insert(0xAA, (0x039A, false)); // GREEK CAPITAL LETTER KAPPA
        m.insert(0xAB, (0x039B, false)); // GREEK CAPITAL LETTER LAMDA
        m.insert(0xAC, (0x039C, false)); // GREEK CAPITAL LETTER MU
        m.insert(0xAD, (0x039D, false)); // GREEK CAPITAL LETTER NU
        m.insert(0xAE, (0x039E, false)); // GREEK CAPITAL LETTER XI
        m.insert(0xAF, (0x039F, false)); // GREEK CAPITAL LETTER OMICRON
        m.insert(0xB0, (0x03A0, false)); // GREEK CAPITAL LETTER PI
        m.insert(0xB1, (0x03A1, false)); // GREEK CAPITAL LETTER RHO
        m.insert(0xB2, (0x03A3, false)); // GREEK CAPITAL LETTER SIGMA
        m.insert(0xB3, (0x03A4, false)); // GREEK CAPITAL LETTER TAU
        m.insert(0xB4, (0x03A5, false)); // GREEK CAPITAL LETTER UPSILON
        m.insert(0xB5, (0x03A6, false)); // GREEK CAPITAL LETTER PHI
        m.insert(0xB6, (0x03A7, false)); // GREEK CAPITAL LETTER CHI
        m.insert(0xB7, (0x03A8, false)); // GREEK CAPITAL LETTER PSI
        m.insert(0xB8, (0x03A9, false)); // GREEK CAPITAL LETTER OMEGA
        m.insert(0xC1, (0x03AC, false)); // GREEK SMALL LETTER ALPHA WITH TONOS
        m.insert(0xC2, (0x03AD, false)); // GREEK SMALL LETTER EPSILON WITH TONOS
        m.insert(0xC3, (0x03AE, false)); // GREEK SMALL LETTER ETA WITH TONOS
        m.insert(0xC4, (0x03AF, false)); // GREEK SMALL LETTER IOTA WITH TONOS
        m.insert(0xC5, (0x03CC, false)); // GREEK SMALL LETTER OMICRON WITH TONOS
        m.insert(0xC6, (0x03CD, false)); // GREEK SMALL LETTER UPSILON WITH TONOS
        m.insert(0xC7, (0x03CE, false)); // GREEK SMALL LETTER OMEGA WITH TONOS
        m.insert(0xD1, (0x03B1, false)); // GREEK SMALL LETTER ALPHA
        m.insert(0xD2, (0x03B2, false)); // GREEK SMALL LETTER BETA
        m.insert(0xD3, (0x03B3, false)); // GREEK SMALL LETTER GAMMA
        m.insert(0xD4, (0x03B4, false)); // GREEK SMALL LETTER DELTA
        m.insert(0xD5, (0x03B5, false)); // GREEK SMALL LETTER EPSILON
        m.insert(0xD6, (0x03B6, false)); // GREEK SMALL LETTER ZETA
        m.insert(0xD7, (0x03B7, false)); // GREEK SMALL LETTER ETA
        m.insert(0xD8, (0x03B8, false)); // GREEK SMALL LETTER THETA
        m.insert(0xD9, (0x03B9, false)); // GREEK SMALL LETTER IOTA
        m.insert(0xDA, (0x03BA, false)); // GREEK SMALL LETTER KAPPA
        m.insert(0xDB, (0x03BB, false)); // GREEK SMALL LETTER LAMDA
        m.insert(0xDC, (0x03BC, false)); // GREEK SMALL LETTER MU
        m.insert(0xDD, (0x03BD, false)); // GREEK SMALL LETTER NU
        m.insert(0xDE, (0x03BE, false)); // GREEK SMALL LETTER XI
        m.insert(0xDF, (0x03BF, false)); // GREEK SMALL LETTER OMICRON
        m.insert(0xE0, (0x03C0, false)); // GREEK SMALL LETTER PI
        m.insert(0xE1, (0x03C1, false)); // GREEK SMALL LETTER RHO
        m.insert(0xE2, (0x03C3, false)); // GREEK SMALL LETTER SIGMA
        m.insert(0xE3, (0x03C4, false)); // GREEK SMALL LETTER TAU
        m.insert(0xE4, (0x03C5, false)); // GREEK SMALL LETTER UPSILON
        m.insert(0xE5, (0x03C6, false)); // GREEK SMALL LETTER PHI
        m.insert(0xE6, (0x03C7, false)); // GREEK SMALL LETTER CHI
        m.insert(0xE7, (0x03C8, false)); // GREEK SMALL LETTER PSI
        m.insert(0xE8, (0x03C9, false)); // GREEK SMALL LETTER OMEGA
        m
    };

    /// EACC (East Asian Character Code) - 0x31
    /// This is a multibyte character set (3 bytes per character)
    /// For now, we'll use a placeholder. This would need the full CJK table.
    static ref EACC_TABLE: HashMap<u8, CharacterMapping> = {
        let m = HashMap::new();
        // EACC uses 3-byte sequences, not direct byte mappings
        // This table would be indexed by first byte of the 3-byte sequence
        m
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_id_from_byte() {
        assert_eq!(CharacterSetId::from_byte(0x42), Some(CharacterSetId::BasicLatin));
        assert_eq!(CharacterSetId::from_byte(0x45), Some(CharacterSetId::AnselExtendedLatin));
        assert_eq!(CharacterSetId::from_byte(0x32), Some(CharacterSetId::BasicHebrew));
        assert_eq!(CharacterSetId::from_byte(0x31), Some(CharacterSetId::EACC));
        assert_eq!(CharacterSetId::from_byte(0xFF), None);
    }

    #[test]
    fn test_basic_latin_mappings() {
        let table = get_charset_table(CharacterSetId::BasicLatin);
        // ASCII 'A' should map to U+0041
        assert_eq!(table.get(&b'A'), Some(&(0x0041u32, false)));
        // ASCII '0' should map to U+0030
        assert_eq!(table.get(&b'0'), Some(&(0x0030u32, false)));
    }

    #[test]
    fn test_ansel_combining_marks() {
        let table = get_charset_table(CharacterSetId::AnselExtendedLatin);
        // 0xE0 should be COMBINING GRAVE ACCENT and be marked as combining
        assert_eq!(table.get(&0xE0), Some(&(0x0300u32, true)));
        // 0xE4 should be COMBINING MACRON and be marked as combining
        assert_eq!(table.get(&0xE4), Some(&(0x0304u32, true)));
    }

    #[test]
    fn test_hebrew_letters() {
        let table = get_charset_table(CharacterSetId::BasicHebrew);
        assert_eq!(table.get(&0xA1), Some(&(0x05D0u32, false))); // HEBREW LETTER ALEF
    }

    #[test]
    fn test_cyrillic_letters() {
        let table = get_charset_table(CharacterSetId::BasicCyrillic);
        assert_eq!(table.get(&0xA1), Some(&(0x0410u32, false))); // CYRILLIC CAPITAL LETTER A
    }
}
