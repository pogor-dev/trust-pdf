use std::borrow::Cow;

use crate::{
    GreenNode, GreenToken,
    green::{NodeOrToken, Trivia},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemOrList<Item, List> {
    /// Single syntax element
    Item(Item),
    /// Collection of syntax elements
    List(List),
}

impl<'a, Item, List> ItemOrList<Item, List>
where
    Item: GreenNode<'a>,
    List: GreenNode<'a>,
{
    pub fn get_first_non_null_child_index(node: &Self) -> u8 {
        for i in 0..node.slot_count() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    pub fn get_last_non_null_child_index(node: &Self) -> u8 {
        for i in (0..node.slot_count()).rev() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    // pub fn get_first_terminal(&self) -> Option<&Item> {
    //     let mut node = Some(self);

    //     loop {
    //         let current = node?;

    //         if let ItemOrList::Item(item) = current
    //             && item.is_token()
    //         {
    //             return Some(item);
    //         }

    //         let mut first_child = None;

    //         for i in 0..current.slot_count() {
    //             if let Some(child) = current.slot(i) {
    //                 first_child = Some(child);
    //                 break;
    //             }
    //         }

    //         node = first_child;

    //         // Optimization: if no children or reached terminal, stop
    //         if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
    //             break;
    //         }
    //     }

    //     node
    // }
}

impl<'a, Item, List> GreenNode<'a> for ItemOrList<Item, List>
where
    Item: GreenNode<'a>,
    List: GreenNode<'a>,
{
    #[inline]
    fn kind(&self) -> crate::SyntaxKind {
        todo!()
    }

    #[inline]
    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn full_width(&self) -> u64 {
        match self {
            ItemOrList::Item(item) => item.full_width(),
            ItemOrList::List(list) => list.full_width(),
        }
    }

    #[inline]
    fn is_token(&self) -> bool {
        if let ItemOrList::Item(item) = self { item.is_token() } else { false }
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        if let ItemOrList::Item(item) = self { item.is_trivia() } else { false }
    }

    #[inline]
    fn is_list(&self) -> bool {
        matches!(self, ItemOrList::List(_))
    }

    #[inline]
    // TODO: abstraction
    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        todo!()
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        todo!()
    }

    #[inline]
    // TODO: abstraction
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    // TODO: abstraction
    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    #[inline]
    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }

    fn width(&self) -> u64 {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn has_leading_trivia(&self) -> bool {
        self.leading_trivia_width() != <u64>::default()
    }

    fn has_trailing_trivia(&self) -> bool {
        self.trailing_trivia_width() != <u64>::default()
    }
}
