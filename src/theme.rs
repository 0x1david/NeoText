use crossterm::style::Color;

pub trait Theme {
    fn from_str(&self, element: &str) -> Color;
}

pub struct DefaultTheme {}

impl Theme for DefaultTheme {
    fn from_str(&self, el: &str) -> Color {
        match el {
            // Functions and methods
            "function" | "method" | "constructor" => Color::Yellow,

            // Keywords and control flow
            "keyword" | "conditional" | "repeat" | "label" | "operator" | "keyword.function"
            | "keyword.return" => Color::Red,

            // Comments
            "comment" | "comment.line" | "comment.block" => Color::DarkGrey,

            // Strings and characters
            "string" | "string.special" | "character" | "character.special" => Color::Green,

            // Numbers and boolean values
            "number" | "float" | "boolean" => Color::Magenta,

            // Variables and parameters
            "variable" | "parameter" | "variable.builtin" => Color::Cyan,

            // Types and classes
            "type" | "type.builtin" | "class" | "struct" | "enum" | "union" | "trait" => {
                Color::Blue
            }

            // Punctuation and delimiters
            "punctuation" | "punctuation.bracket" | "punctuation.delimiter" => Color::White,

            // Modules and namespaces
            "module" | "namespace" => Color::Blue,

            // Constants and attributes
            "constant" | "constant.builtin" | "attribute" => Color::Magenta,

            // Tags (for markup languages)
            "tag" | "tag.attribute" => Color::Blue,

            // Markup
            "text" | "text.strong" | "text.emphasis" | "text.underline" | "text.strike"
            | "text.title" | "text.literal" | "text.uri" => Color::White,

            // Errors and warnings
            "error" => Color::Red,

            // Default case
            _ => Color::Reset,
        }
    }
}

// All credits for this theme go to sainnhe - `https://github.com/sainnhe/sonokai`
pub struct Sonokai;
impl Theme for Sonokai {
    fn from_str(&self, el: &str) -> Color {
        match el {
            // Keywords
            "keyword"
            | "keyword.operator"
            | "keyword.function"
            | "keyword.coroutine"
            | "keyword.import"
            | "keyword.type"
            | "keyword.modifier"
            | "keyword.repeat"
            | "keyword.return"
            | "keyword.debug"
            | "keyword.exception"
            | "keyword.conditional"
            | "keyword.directive" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },

            // Functions and methods
            "function"
            | "function.builtin"
            | "function.call"
            | "function.macro"
            | "function.method"
            | "function.method.call"
            | "constructor" => Color::Rgb {
                r: 12,
                g: 215,
                b: 237,
            },

            // Strings and characters
            "string"
            | "string.documentation"
            | "string.regexp"
            | "string.escape"
            | "string.special"
            | "string.special.symbol"
            | "string.special.url"
            | "string.special.path"
            | "character"
            | "character.special" => Color::Rgb {
                r: 230,
                g: 219,
                b: 116,
            },

            // Numbers and constants
            "number" | "number.float" | "constant" | "constant.builtin" | "constant.macro"
            | "boolean" => Color::Rgb {
                r: 174,
                g: 129,
                b: 255,
            },

            // Comments
            "comment"
            | "comment.documentation"
            | "comment.error"
            | "comment.warning"
            | "comment.todo"
            | "comment.note" => Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            },

            // Variables
            "variable"
            | "variable.builtin"
            | "variable.parameter"
            | "variable.parameter.builtin"
            | "variable.member" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Types and classes
            "type" | "type.builtin" | "type.definition" | "attribute" | "attribute.builtin"
            | "property" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Modules
            "module" | "module.builtin" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Punctuation and delimiters
            "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Markup
            "markup.heading"
            | "markup.strong"
            | "markup.italic"
            | "markup.strikethrough"
            | "markup.underline"
            | "markup.quote"
            | "markup.math"
            | "markup.link"
            | "markup.list" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },

            // Diff
            "diff.plus" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },
            "diff.minus" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },
            "diff.delta" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Tags
            "tag" | "tag.builtin" | "tag.attribute" | "tag.delimiter" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Labels
            "label" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Default for unspecified elements
            _ => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Default text color in Monokai
        }
    }
}
pub struct MonoAndromeda;

impl Theme for MonoAndromeda {
    fn from_str(&self, el: &str) -> Color {
        match el {
            // Keywords
            "keyword"
            | "keyword.operator"
            | "keyword.function"
            | "keyword.coroutine"
            | "keyword.import"
            | "keyword.type"
            | "keyword.modifier"
            | "keyword.repeat"
            | "keyword.return"
            | "keyword.debug"
            | "keyword.exception"
            | "keyword.conditional"
            | "keyword.directive" => Color::Rgb {
                r: 0xbb,
                g: 0x97,
                b: 0xee,
            }, // purple

            // Functions and methods
            "function"
            | "function.builtin"
            | "function.call"
            | "function.macro"
            | "function.method"
            | "function.method.call"
            | "constructor" => Color::Rgb {
                r: 0x9e,
                g: 0xd0,
                b: 0x6c,
            }, // green

            // Strings and characters
            "string"
            | "string.documentation"
            | "string.regexp"
            | "string.escape"
            | "string.special"
            | "string.special.symbol"
            | "string.special.url"
            | "string.special.path"
            | "character"
            | "character.special" => Color::Rgb {
                r: 0xed,
                g: 0xc7,
                b: 0x63,
            }, // yellow

            // Numbers and constants
            "number" | "number.float" | "constant" | "constant.builtin" | "constant.macro"
            | "boolean" => Color::Rgb {
                r: 0xf8,
                g: 0x98,
                b: 0x60,
            }, // orange

            // Comments
            "comment"
            | "comment.documentation"
            | "comment.error"
            | "comment.warning"
            | "comment.todo"
            | "comment.note" => Color::Rgb {
                r: 0x7e,
                g: 0x82,
                b: 0x94,
            }, // grey

            // Variables
            "variable"
            | "variable.builtin"
            | "variable.parameter"
            | "variable.parameter.builtin"
            | "variable.member" => Color::Rgb {
                r: 0xe1,
                g: 0xe3,
                b: 0xe4,
            }, // fg (foreground)

            // Types and classes
            "type" | "type.builtin" | "type.definition" | "attribute" | "attribute.builtin"
            | "property" => Color::Rgb {
                r: 0x6d,
                g: 0xca,
                b: 0xe8,
            }, // blue

            // Modules
            "module" | "module.builtin" => Color::Rgb {
                r: 0xfb,
                g: 0x61,
                b: 0x7e,
            }, // red

            // Punctuation and delimiters
            "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" => Color::Rgb {
                r: 0xe1,
                g: 0xe3,
                b: 0xe4,
            }, // fg

            // Markup
            "markup.heading"
            | "markup.strong"
            | "markup.italic"
            | "markup.strikethrough"
            | "markup.underline"
            | "markup.quote"
            | "markup.math"
            | "markup.link"
            | "markup.list" => Color::Rgb {
                r: 0xbb,
                g: 0x97,
                b: 0xee,
            }, // purple

            // Diff
            "diff.plus" => Color::Rgb {
                r: 0x9e,
                g: 0xd0,
                b: 0x6c,
            }, // green
            "diff.minus" => Color::Rgb {
                r: 0xfb,
                g: 0x61,
                b: 0x7e,
            }, // red
            "diff.delta" => Color::Rgb {
                r: 0xf8,
                g: 0x98,
                b: 0x60,
            }, // orange

            // Tags
            "tag" | "tag.builtin" | "tag.attribute" | "tag.delimiter" => Color::Rgb {
                r: 0x6d,
                g: 0xca,
                b: 0xe8,
            }, // blue

            // Labels
            "label" => Color::Rgb {
                r: 0xf8,
                g: 0x98,
                b: 0x60,
            }, // orange

            // Default for unspecified elements
            _ => Color::Rgb {
                r: 0xe1,
                g: 0xe3,
                b: 0xe4,
            }, // fg (foreground)
        }
    }
}

pub struct Monokai;

impl Theme for Monokai {
    fn from_str(&self, el: &str) -> Color {
        match el {
            // Keywords
            "keyword"
            | "keyword.operator"
            | "keyword.function"
            | "keyword.coroutine"
            | "keyword.import"
            | "keyword.type"
            | "keyword.modifier"
            | "keyword.repeat"
            | "keyword.return"
            | "keyword.debug"
            | "keyword.exception"
            | "keyword.conditional"
            | "keyword.directive" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },

            // Functions and methods
            "function"
            | "function.builtin"
            | "function.call"
            | "function.macro"
            | "function.method"
            | "function.method.call"
            | "constructor" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Strings and characters
            "string"
            | "string.documentation"
            | "string.regexp"
            | "string.escape"
            | "string.special"
            | "string.special.symbol"
            | "string.special.url"
            | "string.special.path"
            | "character"
            | "character.special" => Color::Rgb {
                r: 230,
                g: 219,
                b: 116,
            },

            // Numbers and constants
            "number" | "number.float" | "constant" | "constant.builtin" | "constant.macro"
            | "boolean" => Color::Rgb {
                r: 174,
                g: 129,
                b: 255,
            },

            // Comments
            "comment"
            | "comment.documentation"
            | "comment.error"
            | "comment.warning"
            | "comment.todo"
            | "comment.note" => Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            },

            // Variables
            "variable"
            | "variable.builtin"
            | "variable.parameter"
            | "variable.parameter.builtin"
            | "variable.member" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Types and classes
            "type" | "type.builtin" | "type.definition" | "attribute" | "attribute.builtin"
            | "property" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Modules
            "module" | "module.builtin" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Punctuation and delimiters
            "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Markup
            "markup.heading"
            | "markup.strong"
            | "markup.italic"
            | "markup.strikethrough"
            | "markup.underline"
            | "markup.quote"
            | "markup.math"
            | "markup.link"
            | "markup.list" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },

            // Diff
            "diff.plus" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },
            "diff.minus" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },
            "diff.delta" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Tags
            "tag" | "tag.builtin" | "tag.attribute" | "tag.delimiter" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Labels
            "label" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Default for unspecified elements
            _ => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Default text color in Monokai
        }
    }
}
