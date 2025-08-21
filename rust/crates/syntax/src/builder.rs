mod checkpoint;

use core::cell::Cell;

use crate::links::Links;
use crate::{
    Error, Flavor, FlavorDefault, Index, Length, Pointer, Span, Storage, Tree, TreeIndex, Width,
};

pub use self::checkpoint::Checkpoint;

#[derive(Debug)]
pub struct Builder<T, F = FlavorDefault>
where
    T: Copy,
    F: Flavor,
{
    /// Data in the tree being built.
    tree: Tree<T, F>,
    /// The last checkpoint that was handed out.
    checkpoint: Option<Checkpoint<F::Pointer>>,
    /// Reference the current parent to the node being built. It itself has its
    /// parent set in the tree, so that is what is used to traverse ancestors of
    /// a node.
    parent: Option<F::Pointer>,
    /// Reference to last sibling inserted.
    sibling: Option<F::Pointer>,
    /// The current cursor.
    cursor: F::Index,
}

impl<T> Builder<T, FlavorDefault>
where
    T: Copy,
{
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with()
    }
}

impl<T, F> Builder<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[must_use]
    pub const fn new_with() -> Self {
        Builder {
            tree: Tree::new_with(),
            parent: None,
            checkpoint: None,
            sibling: None,
            cursor: F::Index::EMPTY,
        }
    }

    #[inline]
    pub const fn cursor(&self) -> &F::Index {
        &self.cursor
    }

    pub fn set_cursor(&mut self, cursor: F::Index) {
        self.cursor = cursor;
    }

    pub fn open(&mut self, data: T) -> Result<F::Pointer, Error<F::Error>> {
        let id = self.insert(data, Span::point(self.cursor))?;
        self.parent = Some(id);
        Ok(id)
    }

    pub fn open_with(
        &mut self,
        data: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let id = self.insert(data, span)?;
        self.parent = Some(id);
        Ok(id)
    }

    pub fn close(&mut self) -> Result<(), Error<F::Error>> {
        let head = self.parent.take().ok_or(Error::CloseError)?;

        self.sibling = Some(head);

        let &mut Links { parent, span, .. } = self
            .tree
            .get_mut(head)
            .ok_or_else(|| Error::MissingNode(head.get()))?;

        if let Some(id) = parent {
            let parent = self
                .tree
                .get_mut(id)
                .ok_or_else(|| Error::MissingNode(id.get()))?;

            parent.span = parent.span.join(&span);
            self.parent = Some(id);
        }

        self.cursor = span.end;
        Ok(())
    }

    pub fn token(&mut self, value: T, len: F::Length) -> Result<F::Pointer, Error<F::Error>> {
        let start = self.cursor;

        if !len.is_empty() {
            self.cursor = self.cursor.checked_add_len(len).ok_or(Error::Overflow)?;
            self.tree.span_mut().end = self.cursor;
        }

        let id = self.insert(value, Span::new(start, self.cursor))?;
        self.sibling = Some(id);

        if !len.is_empty() {
            self.tree.indexes_mut().push(TreeIndex {
                index: self.cursor,
                id,
            })?;
        }

        Ok(id)
    }

    pub fn token_with(
        &mut self,
        value: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let id = self.insert(value, span)?;

        self.sibling = Some(id);
        self.tree.indexes_mut().push(TreeIndex {
            index: span.start,
            id,
        })?;

        if let Some(parent) = self.parent.and_then(|id| self.tree.get_mut(id)) {
            parent.span = parent.span.join(&span);
        }

        self.cursor = span.end;
        Ok(id)
    }

    pub fn token_empty(&mut self, value: T) -> Result<F::Pointer, Error<F::Error>> {
        self.token(value, F::Length::EMPTY)
    }

    pub fn checkpoint(&mut self) -> Result<Checkpoint<F::Pointer>, Error<F::Error>> {
        let node = F::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        if let Some(c) = &self.checkpoint {
            if c.node() == node {
                return Ok(c.clone());
            }
        }

        let c = Checkpoint::new(node, self.parent);
        self.checkpoint = Some(c.clone());
        Ok(c)
    }

    pub fn close_at(
        &mut self,
        c: &Checkpoint<F::Pointer>,
        data: T,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let (id, parent) = c.get();

        if parent != self.parent {
            return Err(Error::CloseAtError);
        }

        let new_id = F::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        let Some(links) = self.tree.get_mut(id) else {
            let new_id = self.insert(data, Span::point(self.cursor))?;

            if new_id != id {
                return Err(Error::MissingNode(new_id.get()));
            }

            self.sibling = Some(new_id);
            return Ok(new_id);
        };

        let parent = links.parent.replace(new_id);
        let prev = links.prev.take();

        // Restructuring is necessary to calculate the full span of the newly
        // inserted node and update parent references to point to the newly
        // inserted node.
        let (last, span) = if let Some(next) = links.next {
            let span = links.span;
            let (last, end) = restructure_close_at(&mut self.tree, new_id, next)?;
            (last, Span::new(span.start, end))
        } else {
            (id, links.span)
        };

        if let Some(parent) = parent.and_then(|id| self.tree.get_mut(id)) {
            if parent.first == Some(id) {
                parent.first = Some(new_id);
            }

            if parent.last == Some(id) {
                parent.last = Some(new_id);
            }
        }

        if let Some(prev) = prev.and_then(|id| self.tree.get_mut(id)) {
            prev.next = Some(new_id);
        }

        // If we're replacing the first node of the tree, the newly inserted
        // node should be set as the first node.
        let (first, _) = self.tree.links_mut();

        if *first == Some(id) {
            *first = Some(new_id);
        }

        // Do necessary accounting.
        self.tree.push(Links {
            data: Cell::new(data),
            span,
            prev,
            parent,
            next: None,
            first: Some(id),
            last: Some(last),
        })?;

        self.sibling = Some(new_id);
        c.set(new_id, parent);
        Ok(new_id)
    }

    pub fn close_at_with(
        &mut self,
        c: &Checkpoint<F::Pointer>,
        data: T,
        span: Span<F::Index>,
    ) -> Result<F::Pointer, Error<F::Error>> {
        let (id, parent) = c.get();

        if parent != self.parent {
            return Err(Error::CloseAtError);
        }

        let new_id = F::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        let Some(links) = self.tree.get_mut(id) else {
            let new_id = self.insert(data, span)?;

            if new_id != id {
                return Err(Error::MissingNode(new_id.get()));
            }

            self.sibling = Some(new_id);
            return Ok(new_id);
        };

        let parent = links.parent.replace(new_id);
        let prev = links.prev.take();

        // Restructuring is necessary to calculate the full span of the newly
        // inserted node and update parent references to point to the newly
        // inserted node.
        let last = if let Some(next) = links.next {
            let (last, _) = restructure_close_at(&mut self.tree, new_id, next)?;
            last
        } else {
            id
        };

        if let Some(parent) = parent.and_then(|id| self.tree.get_mut(id)) {
            if parent.first == Some(id) {
                parent.first = Some(new_id);
            }

            if parent.last == Some(id) {
                parent.last = Some(new_id);
            }
        }

        if let Some(prev) = prev.and_then(|id| self.tree.get_mut(id)) {
            prev.next = Some(new_id);
        }

        // If we're replacing the first node of the tree, the newly inserted
        // node should be set as the first node.
        let (first, _) = self.tree.links_mut();

        if *first == Some(id) {
            *first = Some(new_id);
        }

        // Do necessary accounting.
        self.tree.push(Links {
            data: Cell::new(data),
            span,
            prev,
            parent,
            next: None,
            first: Some(id),
            last: Some(last),
        })?;

        self.sibling = Some(new_id);
        c.set(new_id, parent);
        Ok(new_id)
    }

    pub fn build(self) -> Result<Tree<T, F>, Error<F::Error>> {
        if self.parent.is_some() {
            return Err(Error::BuildError);
        }

        Ok(self.tree)
    }

    fn insert(&mut self, data: T, span: Span<F::Index>) -> Result<F::Pointer, Error<F::Error>> {
        let new = F::Pointer::new(self.tree.len()).ok_or(Error::Overflow)?;

        let prev = self.sibling.take();

        self.tree.push(Links {
            data: Cell::new(data),
            span,
            parent: self.parent,
            prev,
            next: None,
            first: None,
            last: None,
        })?;

        if let Some(id) = self.parent {
            if let Some(node) = self.tree.links_at_mut(id) {
                if node.first.is_none() {
                    node.first = Some(new);
                }

                node.last = Some(new);
                node.span.end = span.end;
            }
        } else {
            let (first, last) = self.tree.links_mut();

            if first.is_none() {
                *first = Some(new);
            }

            *last = Some(new);
        }

        if let Some(node) = prev.and_then(|id| self.tree.links_at_mut(id)) {
            node.next = Some(new);
        }

        Ok(new)
    }
}

impl<T, F> Clone for Builder<T, F>
where
    T: Copy,
    F: Flavor<Indexes: Clone, Width: Width<Pointer: Clone>>,
    F::Storage<Links<T, F::Index, F::Pointer>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            parent: self.parent,
            checkpoint: self.checkpoint.clone(),
            sibling: self.sibling,
            cursor: self.cursor,
        }
    }
}

impl<T, F> Default for Builder<T, F>
where
    T: Copy,
    F: Flavor,
{
    #[inline]
    fn default() -> Self {
        Self::new_with()
    }
}

// Adjust span to encapsulate all children and check that we just inserted the
// checkpointed node in the right location which should be the tail sibling of
// the replaced node.
#[allow(clippy::type_complexity)]
fn restructure_close_at<T, F>(
    tree: &mut Tree<T, F>,
    parent_id: F::Pointer,
    next: F::Pointer,
) -> Result<(F::Pointer, F::Index), Error<F::Error>>
where
    T: Copy,
    F: Flavor,
{
    let mut links = tree
        .get_mut(next)
        .ok_or_else(|| Error::MissingNode(next.get()))?;
    let mut last = (next, links.span.end);
    links.parent = Some(parent_id);

    while let Some(next) = links.next {
        links = tree
            .get_mut(next)
            .ok_or_else(|| Error::MissingNode(next.get()))?;
        last = (next, links.span.end);
        links.parent = Some(parent_id);
    }

    Ok(last)
}
