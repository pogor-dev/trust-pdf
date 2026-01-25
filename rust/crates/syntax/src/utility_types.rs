use std::{fmt, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeOrTokenOrTrivia<N, T, R> {
    Node(N),
    Token(T),
    Trivia(R),
}

impl<N, T, R> NodeOrTokenOrTrivia<N, T, R> {
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => Some(node),
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn into_token(self) -> Option<T> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(token) => Some(token),
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn into_trivia(self) -> Option<R> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(trivia) => Some(trivia),
        }
    }

    pub fn as_node(&self) -> Option<&N> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => Some(node),
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(token) => Some(token),
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn as_trivia(&self) -> Option<&R> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(trivia) => Some(trivia),
        }
    }
}

impl<N: Deref, T: Deref, R: Deref> NodeOrTokenOrTrivia<N, T, R> {
    pub(crate) fn as_deref(&self) -> NodeOrTokenOrTrivia<&N::Target, &T::Target, &R::Target> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => NodeOrTokenOrTrivia::Node(node),
            NodeOrTokenOrTrivia::Token(token) => NodeOrTokenOrTrivia::Token(token),
            NodeOrTokenOrTrivia::Trivia(trivia) => NodeOrTokenOrTrivia::Trivia(trivia),
        }
    }
}

impl<N: fmt::Display, T: fmt::Display, R: fmt::Display> fmt::Display for NodeOrTokenOrTrivia<N, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrTokenOrTrivia::Node(node) => fmt::Display::fmt(node, f),
            NodeOrTokenOrTrivia::Token(token) => fmt::Display::fmt(token, f),
            NodeOrTokenOrTrivia::Trivia(trivia) => fmt::Display::fmt(trivia, f),
        }
    }
}
