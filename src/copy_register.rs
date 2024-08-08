use crate::{Error, Result};
use std::collections::{HashMap, VecDeque};

const MAX_NAMED_REGISTERS: usize = 26;
const MAX_NUMBERED_REGISTERS: usize = 10;

pub struct CopyRegister {
    named_registers: HashMap<char, CopyObject>,
    /// `VecDeque` is used instead of a Vec to avoid having to use indexing by numbers not matching
    /// the register (e.g. storing register 0 at index 9, due to the pushing)
    numbered_register: VecDeque<CopyObject>,
}

pub enum CopyObject {
    // for Yanking Text
    Text(String),
    // Later macro implementation will use ActionSequences
    ActionSequence(ActionSequence),
}
impl CopyObject {
    fn replace(&mut self, obj: Self) {
        *self = obj;
    }
    const fn as_text(&self) -> Option<&String> {
        if let Self::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
    const fn as_macro(&self) -> Option<&ActionSequence> {
        if let Self::ActionSequence(seq) = self {
            Some(seq)
        } else {
            None
        }
    }
}

pub struct ActionSequence;
impl Default for CopyRegister {
    fn default() -> Self {
        let mut numbered_register = VecDeque::with_capacity(MAX_NUMBERED_REGISTERS);
        numbered_register.push_front(CopyObject::Text(String::new()));
        Self {
            numbered_register,
            named_registers: HashMap::with_capacity(MAX_NAMED_REGISTERS),
        }
    }
}

impl CopyRegister {
    pub fn yank(&mut self, text: impl Into<String>, named: Option<char>) -> Result<()> {
        let text = CopyObject::Text(text.into());

        if let Some(reg) = named {
            if !reg.is_alphabetic() || !reg.is_ascii_lowercase() {
                return Err(Error::ImATeacup);
            }
            self.named_registers.insert(reg, text);
        } else {
            self.unnamed_register_mut().replace(text);
        }
        Ok(())
    }
    /// Grants access to what is simply the zeroth of the unnamed registers
    fn unnamed_register(&self) -> &CopyObject {
        &self.numbered_register[0]
    }
    /// Grants mutable access to what is simply the zeroth of the unnamed registers
    fn unnamed_register_mut(&mut self) -> &mut CopyObject {
        &mut self.numbered_register[0]
    }
    pub fn get_from_register(&self, named: Option<char>) -> Option<&CopyObject> {
        named.map_or_else(
            || Some(self.unnamed_register()),
            |reg| self.named_registers.get(&reg),
        )
    }
    pub fn push_into_numbered_registers(&mut self, text: impl Into<String>) {
        self.numbered_register
            .insert(1, CopyObject::Text(text.into()));
        if self.numbered_register.len() > MAX_NUMBERED_REGISTERS {
            self.numbered_register.pop_back();
        }
    }
}
