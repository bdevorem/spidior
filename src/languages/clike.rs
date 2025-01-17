//! Provides the parser for "c-like" languages, including C and Java

use super::parsing::{Function, Functions, Identifier, Identifiers};
use std::collections::HashMap;

/// A Functions and Identifiers parser for Clike languages,
/// including C, C++, and Java.
pub struct Clike {}

impl Clike {
    fn is_allowed(x: &str) -> bool {
        !vec![
            "public",
            "package",
            "private",
            "protected",
            "import",
            "void",
            "true",
            "false",
            "extends",
        ]
        .contains(&x)
    }
}

enum FunctionFsm {
    NAME,
    SPACE,
    BRACE,
    NONE,
    PARENS(i32),
}

impl Functions for Clike {
    /// Parses out function declarations from c-like code
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice that contains the code to be parsed
    ///
    /// # Returns
    ///
    /// A Vec of Function containing information on every function
    /// declared within text
    fn read_functions(&self, text: &str) -> Vec<Function> {
        let mut s = FunctionFsm::NONE;
        let mut start = 0;
        let mut end = 0;
        let mut v = Vec::new();
        for (i, c) in text.chars().enumerate() {
            match s {
                FunctionFsm::NONE => {
                    if c.is_alphanumeric() {
                        s = FunctionFsm::NAME;
                        start = i;
                    }
                }
                FunctionFsm::NAME => {
                    if c == '(' {
                        s = FunctionFsm::PARENS(1);
                        end = i;
                    } else if !c.is_alphanumeric() {
                        s = FunctionFsm::NONE;
                    }
                }
                FunctionFsm::PARENS(j) => {
                    if c == '(' {
                        s = FunctionFsm::PARENS(j + 1);
                    } else if c == ')' {
                        s = FunctionFsm::PARENS(j - 1);
                    } else if c.is_whitespace() && j == 0 {
                        s = FunctionFsm::SPACE;
                    } else if c == '{' && j == 0 {
                        s = FunctionFsm::BRACE;
                    }
                }
                FunctionFsm::SPACE => {
                    if c.is_alphanumeric() {
                        s = FunctionFsm::NAME;
                        start = i;
                    } else if c == '{' {
                        s = FunctionFsm::BRACE;
                    } else if !c.is_whitespace() {
                        s = FunctionFsm::NONE;
                    }
                }
                FunctionFsm::BRACE => {
                    v.push(Function::new(text[start..end].to_string()));
                    s = FunctionFsm::NONE;
                }
            }
        }
        v
    }
}

enum IFsm {
    NONE,
    NAME1,
    SPACE,
    NAME2,
    DOT,
}

impl Identifiers for Clike {
    /// Parses out identifier uses from c-like code
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice that contains the code to be parsed
    ///
    /// # Returns
    ///
    /// A Vec of Identifier containing information on every use of
    /// an identifier declared within the code
    fn read_identifiers(&self, text: &str) -> Vec<Identifier> {
        let mut s = IFsm::NONE;
        let mut n1s = 0;
        let mut n1e = 0;
        let mut n2s = 0;
        let mut n2e;
        let mut v = Vec::new();
        let mut stack = Vec::<HashMap<String, String>>::new();
        stack.push(HashMap::new());
        for (i, c) in text.chars().enumerate() {
            if c == '{' {
                stack.push(HashMap::new());
                s = IFsm::NONE;
            } else if c == '}' {
                stack.pop();
                s = IFsm::NONE;
            }
            match s {
                IFsm::NONE => {
                    if c == '.' {
                        s = IFsm::DOT;
                    } else if c.is_alphabetic() {
                        s = IFsm::NAME1;
                        n1s = i;
                    }
                }
                IFsm::DOT => {
                    if c.is_whitespace() {
                        s = IFsm::SPACE;
                    }
                }
                IFsm::NAME1 => {
                    if c.is_whitespace() {
                        s = IFsm::SPACE;
                        n1e = i;
                    } else if !c.is_alphanumeric() {
                        //Push declared identifier
                        if c == '.' {
                            s = IFsm::DOT;
                        } else {
                            s = IFsm::NONE;
                        }
                        n1e = i;
                        let name = text[n1s..n1e].to_string();
                        for frame in stack.iter().rev() {
                            if let Some(typ) = frame.get(&name) {
                                v.push(Identifier::new(name, typ.to_string(), n1s, n1e));
                                break;
                            }
                        }
                    }
                }
                IFsm::SPACE => {
                    if c.is_alphabetic() {
                        s = IFsm::NAME2;
                        n2s = i;
                    } else if !c.is_whitespace() {
                        //Push declared identifier
                        s = IFsm::NONE;
                        let name = text[n1s..n1e].to_string();
                        for frame in stack.iter().rev() {
                            if let Some(typ) = frame.get(&name) {
                                v.push(Identifier::new(name.clone(), typ.to_string(), n1s, n1e));
                                break;
                            }
                        }
                    }
                }
                IFsm::NAME2 => {
                    if !c.is_alphanumeric() {
                        //Push new delcaration
                        if c == '.' {
                            s = IFsm::DOT;
                        } else {
                            s = IFsm::NONE;
                        }
                        n2e = i;
                        let name = text[n2s..n2e].to_string();
                        let typ = text[n1s..n1e].to_string();
                        if Clike::is_allowed(name.as_ref()) && Clike::is_allowed(typ.as_ref()) {
                            v.push(Identifier::new(name.clone(), typ.clone(), n2s, n2e));
                            stack.last_mut().unwrap().insert(name, typ);
                        }
                    }
                }
            }
        }
        v
    }
}

#[test]
fn test_functions() {
    let expected = "[Function { name: \"LightningOvercharge\" }, Function { name: \"getAction\" }, Function { name: \"onSpawn\" }, Function { name: \"getPassiveAction\" }, Function { name: \"getCost\" }, Function { name: \"getName\" }, Function { name: \"getTip\" }, Function { name: \"getActionNetwork\" }]";
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let clike = Clike {};
    d.push("resources/test/functions.java");
    let text = std::fs::read_to_string(d).unwrap();
    let result = format!("{:?}", clike.read_functions(&text));
    assert_eq!(result, expected);
}

#[test]
fn test_identifiers() {
    let expected = "[Identifier { name: \"com\", typ: \"static\", start: 67, end: 70 }, Identifier { name: \"com\", typ: \"static\", start: 232, end: 235 }, Identifier { name: \"com\", typ: \"static\", start: 273, end: 276 }, Identifier { name: \"com\", typ: \"static\", start: 316, end: 319 }, Identifier { name: \"com\", typ: \"static\", start: 361, end: 364 }, Identifier { name: \"LightningOvercharge\", typ: \"class\", start: 414, end: 433 }, Identifier { name: \"charge\", typ: \"int\", start: 462, end: 468 }, Identifier { name: \"charge\", typ: \"int\", start: 517, end: 523 }, Identifier { name: \"number\", typ: \"double\", start: 547, end: 553 }, Identifier { name: \"me\", typ: \"Session\", start: 601, end: 603 }, Identifier { name: \"number\", typ: \"double\", start: 615, end: 621 }, Identifier { name: \"me\", typ: \"Session\", start: 635, end: 637 }, Identifier { name: \"me\", typ: \"Session\", start: 635, end: 637 }]";
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/identifiers.java");
    let clike = Clike {};
    let text = std::fs::read_to_string(d).unwrap();
    let result = format!("{:?}", clike.read_identifiers(&text));
    assert_eq!(result, expected);
    assert!(Clike::is_allowed("bob"));
    assert!(!Clike::is_allowed("private"));
}

#[test]
fn test_replace_whole() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/identifiers.java");
    let text = std::fs::read_to_string(d).unwrap();
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/identifiers_replaced.java");
    let expected = std::fs::read_to_string(d).unwrap();
    assert_eq!(
        crate::nfa::replacer::replace(
            &text,
            crate::regexparser::parse("%s/[[type=Session]]/sess/g").unwrap(),
            |x, y| true
        )
        .unwrap(),
        expected
    );
}
