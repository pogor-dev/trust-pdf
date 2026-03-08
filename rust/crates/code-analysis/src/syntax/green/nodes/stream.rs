use crate::{
    GreenCst, GreenDiagnostic, GreenDictionaryExpressionSyntax, GreenExpressionSyntax, GreenListSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax,
    GreenTokenElement, SyntaxKind,
};

/// Represents a stream object with optional compression/decoding
/// ISO 32000-2:2020, 7.5.8 — Stream objects
#[derive(Clone)]
pub(crate) struct GreenStreamExpressionSyntax(GreenExpressionSyntax);

impl GreenStreamExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        stream_token: GreenNodeElement,
        body: GreenNodeElement,
        end_stream_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![stream_token.into(), body.into(), end_stream_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenStreamExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn stream_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn body(&self) -> Option<GreenStreamBodySyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenStreamBodySyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn end_stream_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
pub(crate) struct GreenStreamBodySyntax(GreenExpressionSyntax);

impl GreenStreamBodySyntax {
    pub(crate) fn new(kind: SyntaxKind, data: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![data];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenStreamBodySyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn raw_data(&self) -> Option<GreenStreamRawDataSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenStreamRawDataSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn decoded_data(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
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
pub(crate) struct GreenStreamRawDataSyntax(GreenExpressionSyntax);

impl GreenStreamRawDataSyntax {
    pub(crate) fn new(kind: SyntaxKind, data: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![data.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenStreamRawDataSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn data(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => Some(n.clone()),
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
pub(crate) struct GreenStreamOperatorOperandExpressionSyntax(GreenExpressionSyntax);

impl GreenStreamOperatorOperandExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, operands: GreenNodeElement, operator: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![operands.into(), operator.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenStreamOperatorOperandExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn operands(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn operator(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
pub(crate) struct GreenTextObjectSyntax(GreenExpressionSyntax);

impl GreenTextObjectSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        bt_token: GreenNodeElement,
        content: GreenNodeElement,
        et_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![bt_token.into(), content.into(), et_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenTextObjectSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn bt_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn et_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
pub(crate) struct GreenInlineImageSyntax(GreenExpressionSyntax);

impl GreenInlineImageSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        bi_token: GreenNodeElement,
        inline_dict: GreenNodeElement,
        id_token: GreenNodeElement,
        image_data: GreenNodeElement,
        ei_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![bi_token.into(), inline_dict.into(), id_token.into(), image_data.into(), ei_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenInlineImageSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn bi_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn inline_dict(&self) -> Option<GreenDictionaryExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenDictionaryExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn id_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn image_data(&self) -> Option<GreenNode> {
        match self.0.green().slot(3) {
            Some(GreenNodeElement::Node(n)) => Some(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn ei_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(4) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
pub(crate) struct GreenMarkedContentSyntax(GreenExpressionSyntax);

impl GreenMarkedContentSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        open_token: GreenNodeElement,
        content: GreenNodeElement,
        close_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![open_token.into(), content.into(), close_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenMarkedContentSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn open_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn close_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
pub(crate) struct GreenCompatibilityExpressionSyntax(GreenExpressionSyntax);

impl GreenCompatibilityExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        bx_token: GreenNodeElement,
        content: GreenNodeElement,
        ex_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![bx_token.into(), content.into(), ex_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenCompatibilityExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn bx_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn content(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn ex_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
