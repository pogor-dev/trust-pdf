use core::convert::Infallible;
use core::fmt;

/// Errors raised while building a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error<E = Infallible> {
    CloseError,
    BuildError,
    CloseAtError,
    Overflow,
    MissingNode(usize),
    Flavor(E),
}

impl<E> From<E> for Error<E> {
    #[inline]
    fn from(error: E) -> Self {
        Error::Flavor(error)
    }
}

impl<E> core::error::Error for Error<E>
where
    E: 'static + core::error::Error,
{
    #[inline]
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Flavor(error) => Some(error),
            _ => None,
        }
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CloseError => {
                write!(f, "no node being built")
            }
            Error::BuildError => {
                write!(f, "tree is currently being built")
            }
            Error::CloseAtError => {
                write!(
                    f,
                    "trying to close a node which is not a sibling of the checkpoint being closed"
                )
            }
            Error::Overflow => {
                write!(f, "numerical overflow")
            }
            Error::MissingNode(p) => {
                write!(f, "missing node with id `{p}`")
            }
            Error::Flavor(error) => error.fmt(f),
        }
    }
}
