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

pub struct Monokai;

impl Theme for Monokai {
    fn from_str(&self, el: &str) -> Color {
        match el {
            // Keywords
            "keyword" | "storage" | "type" | "keyword.control" | "keyword.operator" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            },

            // Functions and methods
            "function" | "method" | "constructor" | "keyword.function" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Strings and characters
            "string" | "string.quoted" | "string.template" | "character" | "character.special" => {
                Color::Rgb {
                    r: 230,
                    g: 219,
                    b: 116,
                }
            }

            // Numbers
            "number" | "constant.numeric" => Color::Rgb {
                r: 174,
                g: 129,
                b: 255,
            },

            // Comments
            "comment" | "comment.line" | "comment.block" => Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            },

            // Variables
            "variable" | "variable.parameter" | "variable.other" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Constants and support
            "constant" | "constant.language" | "support" | "support.constant" => Color::Rgb {
                r: 174,
                g: 129,
                b: 255,
            },

            // Classes, entities, and tags
            "entity.name.class" | "entity.name.type" | "entity.other" | "tag" => Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            },

            // Attributes
            "entity.other.attribute-name" | "variable.language" => Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            },

            // Punctuation
            "punctuation" | "meta.bracket" => Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },

            // Markup
            "markup.heading" | "markup.bold" | "markup.italic" => Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
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
