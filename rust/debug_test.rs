use syntax::green::{GreenNode, GreenToken, SyntaxKind};

fn main() {
    // Create a simple PDF object structure for debugging
    let obj_token = GreenToken::new(SyntaxKind::OBJ_KEYWORD, b"obj").unwrap();
    let number_token = GreenToken::new(SyntaxKind::NUMBER, b"123").unwrap();
    let endobj_token = GreenToken::new(SyntaxKind::ENDOBJ_KEYWORD, b"endobj").unwrap();

    let children = vec![obj_token.into(), number_token.into(), endobj_token.into()];

    let object_node = GreenNode::new(SyntaxKind::PDF_OBJECT, children);

    println!("Debug output with full_text:");
    println!("{:#?}", object_node);
}
