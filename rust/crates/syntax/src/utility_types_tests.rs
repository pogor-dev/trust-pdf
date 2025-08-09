use crate::NodeOrToken;

#[test]
fn test_into_node_when_node_variant_expect_some_node() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.into_node(), Some("test_node"));
    assert_eq!(node_or_token.into_token(), None);
}

#[test]
fn test_into_token_when_token_variant_expect_some_token() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.into_token(), Some("test_token"));
    assert_eq!(node_or_token.into_node(), None);
}

#[test]
fn test_as_node_when_node_variant_expect_some_node_ref() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.as_node(), Some(&"test_node"));
    assert_eq!(node_or_token.as_token(), None);
}

#[test]
fn test_as_token_when_token_variant_expect_some_token_ref() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.as_token(), Some(&"test_token"));
    assert_eq!(node_or_token.as_node(), None);
}

#[test]
fn test_as_deref_when_node_variant_expect_dereferenced_node() {
    let node_data = String::from("test_node");
    let node_or_token: NodeOrToken<&String, &String> = NodeOrToken::Node(&node_data);
    let deref_result = node_or_token.as_deref();

    match deref_result {
        NodeOrToken::Node(node_str) => assert_eq!(node_str, "test_node"),
        NodeOrToken::Token(_) => panic!("Expected Node variant"),
    }
}

#[test]
fn test_as_deref_when_token_variant_expect_dereferenced_token() {
    let token_data = String::from("test_token");
    let node_or_token: NodeOrToken<&String, &String> = NodeOrToken::Token(&token_data);
    let deref_result = node_or_token.as_deref();

    match deref_result {
        NodeOrToken::Node(_) => panic!("Expected Token variant"),
        NodeOrToken::Token(token_str) => assert_eq!(token_str, "test_token"),
    }
}

#[test]
fn test_fmt_when_node_variant_expect_node_display() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    let formatted = format!("{}", node_or_token);
    assert_eq!(formatted, "test_node");
}

#[test]
fn test_fmt_when_token_variant_expect_token_display() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    let formatted = format!("{}", node_or_token);
    assert_eq!(formatted, "test_token");
}
