//TODO: Come back to this
use super::{BufferPlane, TextBuffer};
use crate::modals::Modal;
use crate::{Error, LineCol, Pattern, Result};

enum RopeNode {
    Leaf(String),
    Node(Box<RopeBuffer>),
}

struct RopeBuffer {
    left_size: usize,
    text: String,
    left: RopeNode,
    right: RopeNode,
}

// impl RopeBuffer {
//     fn find_byte_idx(&self, idx: usize) -> Option<&RopeBuffer> {
//         if self.start_byte < idx && idx < self.end_byte {
//             Some(self)
//         } else if self.start_byte < idx {
//         } else if self.end_byte > idx {
//         } else {
//             None
//         }
//     }
// }
