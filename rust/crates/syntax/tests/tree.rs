#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Nested,
    Number,
    String,
    Lit,
    Whitespace,
    Comment,
    LineFeed,
}

use syntax::Builder;

#[test]
fn test_tree_structure() -> Result<()> {
    // Arrange

    let mut tree = Builder::new();
    let c = tree.checkpoint()?;

    tree.open_node(Syntax::Number)?;
    tree.token(Syntax::Lit, 1)?;
    tree.token(Syntax::Lit, 3)?;

    tree.open_node(Syntax::Nested)?;

    // tree.open_token(Syntax::String, 4)?;
    // tree.trivia(Syntax::Comment, 4)?; // %abc
    // tree.trivia(Syntax::LineFeed, 1)?; // \n

    tree.token(Syntax::String, 4)?; // (ab)

    // tree.trivia(Syntax::Whitespace, 3)?; // [space][space][space]
    // tree.close_token()?;

    tree.close_node()?;
    tree.close_node()?;

    tree.close_at(&c, Syntax::Root)?;

    let expected = syntax::tree! {
        Syntax::Root => {
            Syntax::Number => {
                (Syntax::Lit, 1),
                (Syntax::Lit, 3),
            },
            Syntax::Nested => {
                (Syntax::String, 4)
            }
        }
    };

    // Act
    let tree = tree.build()?;

    // Assert
    assert_eq!(tree, expected);
    Ok(())
}
