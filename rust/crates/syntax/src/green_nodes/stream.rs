use crate::{GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenListSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind};

#[derive(Clone)]
pub struct GreenStreamExpressionSyntax(GreenExpressionSyntax);

impl GreenStreamExpressionSyntax {
    pub fn new(kind: SyntaxKind, stream_token: GreenToken, body: GreenNode, end_stream_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![
            GreenElement::Token(stream_token),
            GreenElement::Node(body),
            GreenElement::Token(end_stream_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenStreamExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn stream_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn body(&self) -> Option<GreenStreamBodySyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenStreamBodySyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn end_stream_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenStreamExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::StreamExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenStreamExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenStreamBodySyntax(GreenExpressionSyntax);

impl GreenStreamBodySyntax {
    pub fn new(kind: SyntaxKind, data: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(data)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenStreamBodySyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn raw_data(&self) -> Option<GreenStreamRawDataSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenStreamRawDataSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn decoded_data(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenStreamBodySyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::StreamBodyExpression && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenStreamBodySyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenStreamRawDataSyntax(GreenExpressionSyntax);

impl GreenStreamRawDataSyntax {
    pub fn new(kind: SyntaxKind, data: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(data)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenStreamRawDataSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn data(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenStreamRawDataSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::StreamRawDataExpression && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenStreamRawDataSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenStreamOperatorOperandExpressionSyntax(GreenExpressionSyntax);

impl GreenStreamOperatorOperandExpressionSyntax {
    pub fn new(kind: SyntaxKind, operands: GreenNode, operator: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(operands), GreenElement::Token(operator)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenStreamOperatorOperandExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn operands(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn operator(&self) -> Option<GreenToken> {
        match self.0.green().slot(1) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenStreamOperatorOperandExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::StreamOperandOperatorExpression && node.slot_count() == 2
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenStreamOperatorOperandExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
