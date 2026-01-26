use crate::{GreenNode, GreenNodeSyntax};
#[derive(Clone)]
pub struct GreenExpressionSyntax(pub GreenNode);

impl GreenNodeSyntax for GreenExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0
    }
}
