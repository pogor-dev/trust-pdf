use crate::{
    SyntaxKind,
    green::{flags::GreenFlags, green_node::GreenNode},
};

pub(crate) trait GreenNodeTrait {
    fn kind(&self) -> SyntaxKind;

    fn flags(&self) -> GreenFlags;

    fn set_flags(&mut self, flags: GreenFlags);

    #[inline]
    fn is_missing(&self) -> bool {
        !self.flags().contains(GreenFlags::IS_NOT_MISSING)
    }

    #[inline]
    fn contains_diagnostics(&self) -> bool {
        self.flags().contains(GreenFlags::CONTAINS_DIAGNOSTICS)
    }

    fn is_list(&self) -> bool {
        self.kind() == SyntaxKind::List
    }

    #[inline]
    fn is_token(&self) -> bool {
        false
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        false
    }

    fn language(&self) -> &'static str {
        "PDF"
    }

    fn width(&self) -> u32 {
        self.full_width() - self.get_leading_trivia_width() - self.get_trailing_trivia_width()
    }

    fn full_width(&self) -> u32;

    fn get_leading_trivia_width(&self) -> u32 {
        match self.full_width() {
            0 => 0,
            _ => self.get_first_terminal().map_or(0, |terminal| terminal.get_leading_trivia_width()),
        }
    }

    fn get_trailing_trivia_width(&self) -> u32 {
        match self.full_width() {
            0 => 0,
            _ => self.get_last_terminal().map_or(0, |terminal| terminal.get_trailing_trivia_width()),
        }
    }

    #[inline]
    fn has_leading_trivia(&self) -> bool {
        self.get_leading_trivia_width() > 0
    }

    #[inline]
    fn has_trailing_trivia(&self) -> bool {
        self.get_trailing_trivia_width() > 0
    }

    fn get_slot_count(&self) -> u32;

    /// This should only be called for nodes that couldn't store their slot count in the
    /// `node_flags_and_slot_count` field. The only nodes that cannot do that are
    /// `WithManyChildren` list types, and those should override this method.
    fn get_large_slot_count(&self) -> u32 {
        0
    }

    fn set_slot_count(&mut self, value: u8);

    fn get_slot(&self, index: usize) -> Option<GreenNode>;

    fn get_slot_offset(&self, index: usize) -> u32 {
        let mut offset = 0;
        for i in 0..index {
            if let Some(child) = self.get_slot(i) {
                offset += child.full_width();
            }
        }

        offset
    }

    fn get_first_non_null_child_index(&self) -> usize {
        let count = self.get_slot_count() as usize;
        for i in 0..count {
            if self.get_slot(i).is_some() {
                return i;
            }
        }

        count
    }

    fn get_last_non_null_child_index(&self) -> usize {
        let count = self.get_slot_count() as usize;
        for i in (0..count).rev() {
            if self.get_slot(i).is_some() {
                return i;
            }
        }

        0
    }

    #[inline]
    fn text(&self) -> Vec<u8> {
        self.write_to(false, false)
    }

    #[inline]
    fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    #[inline]
    fn get_leading_trivia(&self) -> Option<GreenNode> {
        self.get_first_terminal().and_then(|terminal| terminal.get_leading_trivia())
    }

    #[inline]
    fn get_trailing_trivia(&self) -> Option<GreenNode> {
        self.get_last_terminal().and_then(|terminal| terminal.get_trailing_trivia())
    }

    fn get_first_terminal(&self) -> Option<GreenNode> {
        for i in 0..self.get_slot_count() as usize {
            let child = self.get_slot(i);
            if let Some(child) = child {
                if child.is_token() {
                    return Some(child);
                }

                if let Some(token) = child.get_first_terminal() {
                    return Some(token);
                }
            }
        }

        None
    }

    fn get_last_terminal(&self) -> Option<GreenNode> {
        let count = self.get_slot_count() as usize;
        for i in (0..count).rev() {
            let child = self.get_slot(i);
            if let Some(child) = child {
                if child.is_token() {
                    return Some(child);
                }

                if let Some(token) = child.get_last_terminal() {
                    return Some(token);
                }
            }
        }

        None
    }

    #[inline]
    fn write_trivia_to(&self) -> &[u8] {
        &[]
    }

    #[inline]
    fn write_token_to(&self, _leading: bool, _trailing: bool) -> Vec<u8> {
        Vec::new()
    }

    /// Returns the node's text as a byte vector.
    ///
    /// Similar to Roslyn's WriteTo implementation, uses an explicit stack to avoid
    /// stack overflow on deeply nested structures.
    ///
    /// # Parameters
    /// * `leading` - If true, include the first token's leading trivia
    /// * `trailing` - If true, include the last token's trailing trivia
    fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut output = Vec::new();
        let mut stack: Vec<(GreenNode, bool, bool)> = Vec::with_capacity(64);

        let first_index = self.get_first_non_null_child_index();
        let last_index = self.get_last_non_null_child_index();

        if first_index > last_index {
            return output;
        }

        for i in (first_index..=last_index).rev() {
            if let Some(child) = self.get_slot(i) {
                let is_first = i == first_index;
                let is_last = i == last_index;
                stack.push((child, leading || !is_first, trailing || !is_last));
            }
        }

        while let Some((current, current_leading, current_trailing)) = stack.pop() {
            if current.is_token() {
                output.extend_from_slice(&current.write_token_to(current_leading, current_trailing));
                continue;
            }

            if current.is_trivia() {
                output.extend_from_slice(&current.write_trivia_to());
                continue;
            }

            let first_index = current.get_first_non_null_child_index();
            let last_index = current.get_last_non_null_child_index();
            if first_index > last_index {
                continue;
            }

            for i in (first_index..=last_index).rev() {
                if let Some(child) = current.get_slot(i) {
                    let is_first = i == first_index;
                    let is_last = i == last_index;
                    stack.push((child, current_leading || !is_first, current_trailing || !is_last));
                }
            }
        }

        output
    }
}
