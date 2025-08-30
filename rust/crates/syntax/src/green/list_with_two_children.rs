pub struct SyntaxListWithTwoChildren {
    child0: Box<dyn GreenNode>,
    child1: Box<dyn GreenNode>,
    full_width: usize,
}
