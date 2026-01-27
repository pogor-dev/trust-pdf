use crate::{
    GreenCst, GreenDiagnostics, GreenDictionaryExpressionSyntax, GreenElement, GreenExpressionSyntax, GreenListSyntax, GreenNode, GreenNodeSyntax, GreenToken,
    GreenTrait, SyntaxKind,
};

/// Represents a stream object with optional compression/decoding
/// ISO 32000-2:2020, 7.5.8 — Stream objects
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

/// Stream body content: either raw data or decoded operations
/// ISO 32000-2:2020, 7.5.8.2 — Stream data
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

/// Raw stream data before decoding filters are applied
/// ISO 32000-2:2020, 7.4 — Filters
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

/// Stream operator with operands: operand1 operand2 ... operatorName
/// ISO 32000-2:2020, 8.2 — Graphics objects
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

/// Represents a text object: BT ... ET
/// ISO 32000-2:2020, Table 108 — Text state operators
#[derive(Clone)]
pub struct GreenTextObjectSyntax(GreenExpressionSyntax);

impl GreenTextObjectSyntax {
    pub fn new(kind: SyntaxKind, bt_token: GreenToken, content: GreenNode, et_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(bt_token), GreenElement::Node(content), GreenElement::Token(et_token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenTextObjectSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn bt_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn et_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenTextObjectSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::TextObjectExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenTextObjectSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Represents an inline image object: BI ... EI
/// ISO 32000-2:2020, 8.9.7 — Inline images
#[derive(Clone)]
pub struct GreenInlineImageSyntax(GreenExpressionSyntax);

impl GreenInlineImageSyntax {
    pub fn new(
        kind: SyntaxKind,
        bi_token: GreenToken,
        inline_dict: GreenNode,
        id_token: GreenToken,
        image_data: GreenNode,
        ei_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Token(bi_token),
            GreenElement::Node(inline_dict),
            GreenElement::Token(id_token),
            GreenElement::Node(image_data),
            GreenElement::Token(ei_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenInlineImageSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn bi_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn inline_dict(&self) -> Option<GreenDictionaryExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenDictionaryExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn id_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn image_data(&self) -> Option<GreenNode> {
        match self.0.green().slot(3) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }

    #[inline]
    pub fn ei_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(4) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenInlineImageSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::InlineImageExpression && node.slot_count() == 5
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenInlineImageSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Represents a marked-content block: BMC...EMC or BDC...EMC
/// ISO 32000-2:2020, 14.6 — Marked content
#[derive(Clone)]
pub struct GreenMarkedContentSyntax(GreenExpressionSyntax);

impl GreenMarkedContentSyntax {
    pub fn new(kind: SyntaxKind, open_token: GreenToken, content: GreenNode, close_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(open_token), GreenElement::Node(content), GreenElement::Token(close_token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenMarkedContentSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn open_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn close_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenMarkedContentSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::MarkedContentExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenMarkedContentSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Represents a compatibility section: BX ... EX
/// ISO 32000-2:2020, 8.12 — Compatibility
#[derive(Clone)]
pub struct GreenCompatibilityExpressionSyntax(GreenExpressionSyntax);

impl GreenCompatibilityExpressionSyntax {
    pub fn new(kind: SyntaxKind, bx_token: GreenToken, content: GreenNode, ex_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(bx_token), GreenElement::Node(content), GreenElement::Token(ex_token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenCompatibilityExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn bx_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn ex_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenCompatibilityExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::CompatibilityExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenCompatibilityExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
