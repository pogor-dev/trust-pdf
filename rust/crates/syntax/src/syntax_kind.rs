/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    None = 0,

    List = 1,

    // keywords
    TrueKeyword,
    FalseKeyword,
    NullKeyword,
    IndirectObjectKeyword,
    IndirectEndObjectKeyword,
    IndirectReferenceKeyword,
    StreamKeyword,
    EndStreamKeyword,
    XRefKeyword,
    XRefFreeEntryKeyword,
    XRefInUseEntryKeyword,
    FileTrailerKeyword,
    StartXRefKeyword,

    // punctuation
    /// `[`
    OpenBracketToken,
    /// `]`
    CloseBracketToken,
    /// `<<`
    OpenDictToken,
    /// `>>`
    CloseDictToken,

    // PDF content stream operators as defined by ISO 32000-2, Annex A.2, Table A.1
    /// Close, fill, and stroke path using non-zero winding number rule (`b`).
    CloseFillStrokePathOperator,
    /// Fill and stroke path using non-zero winding number rule (`B`).
    FillStrokePathOperator,
    /// CloseOperator, fillOperator, and stroke path using even-odd rule (`b*`).
    CloseFillStrokePathEvenOddOperator,
    /// Fill and stroke path using even-odd rule (`B*`).
    FillStrokePathEvenOddOperator,
    /// Begin marked-content sequence with property list (PDF 1.2) (`BDC`).
    BeginMarkedContentPropertyOperator,
    /// Begin inline image object (`BI`).
    BeginInlineImageOperator,
    /// Begin marked-content sequence (PDF 1.2) (`BMC`).
    BeginMarkedContentOperator,
    /// Begin text object (`BT`).
    BeginTextOperator,
    /// Begin compatibility section (PDF 1.1) (`BX`).
    BeginCompatibilityOperator,
    /// Append curved segment to path (three control points) (`c`).
    CurveToOperator,
    /// Concatenate matrix to current transformation matrix (`cm`).
    ConcatMatrixOperator,
    /// Set color space for stroking operations (PDF 1.1) (`CS`).
    SetStrokeColorSpaceOperator,
    /// Set color space for nonstroking operations (PDF 1.1) (`cs`).
    SetNonStrokeColorSpaceOperator,
    /// Set line dash pattern (`d`).
    SetDashPatternOperator,
    /// Set glyph width in Type 3 font (`d0`).
    SetCharWidthOperator,
    /// Set glyph width and bounding box in Type 3 font (`d1`).
    SetCacheDeviceOperator,
    /// Invoke named XObject (`Do`).
    InvokeXObjectOperator,
    /// Define marked-content point with property list (PDF 1.2) (`DP`).
    DefineMarkedContentPropertyOperator,
    /// End inline image object (`EI`).
    EndInlineImageOperator,
    /// End marked-content sequence (PDF 1.2) (`EMC`).
    EndMarkedContentOperator,
    /// End text object (`ET`).
    EndTextOperator,
    /// End compatibility section (PDF 1.1) (`EX`).
    EndCompatibilityOperator,
    /// Fill path using non-zero winding number rule (`f`).
    FillPathOperator,
    /// Fill path using non-zero winding number rule (deprecated PDF 2.0) (`F`).
    FillPathDeprecatedOperator,
    /// Fill path using even-odd rule (`f*`).
    FillPathEvenOddOperator,
    /// Set gray level for stroking operations (`G`).
    SetStrokeGrayOperator,
    /// Set gray level for nonstroking operations (`g`).
    SetNonStrokeGrayOperator,
    /// Set parameters from graphics state parameter dictionary (PDF 1.2) (`gs`).
    SetGraphicsStateParametersOperator,
    /// Close subpath (`h`).
    CloseSubpathOperator,
    /// Set flatness tolerance (`i`).
    SetFlatnessToleranceOperator,
    /// Begin inline image data (`ID`).
    BeginInlineImageDataOperator,
    /// Set line join style (`j`).
    SetLineJoinOperator,
    /// Set line cap style (`J`).
    SetLineCapOperator,
    /// Set CMYK color for stroking operations (`K`).
    SetStrokeCMYKColorOperator,
    /// Set CMYK color for nonstroking operations (`k`).
    SetNonStrokeCMYKColorOperator,
    /// Append straight line segment to path (`l`).
    LineToOperator,
    /// Begin new subpath (`m`).
    MoveToOperator,
    /// Set miter limit (`M`).
    SetMiterLimitOperator,
    /// Define marked-content point (PDF 1.2) (`MP`).
    DefineMarkedContentPointOperator,
    /// End path without filling or stroking (`n`).
    EndPathOperator,
    /// Save graphics state (`q`).
    SaveGraphicsStateOperator,
    /// Restore graphics state (`Q`).
    RestoreGraphicsStateOperator,
    /// Append rectangle to path (`re`).
    RectangleOperator,
    /// Set RGB color for stroking operations (`RG`).
    SetStrokeRGBColorOperator,
    /// Set RGB color for nonstroking operations (`rg`).
    SetNonStrokeRGBColorOperator,
    /// Set color rendering intent (`ri`).
    SetRenderingIntentOperator,
    /// Close and stroke path (`s`).
    CloseStrokePathOperator,
    /// Stroke path (`S`).
    StrokePathOperator,
    /// Set color for stroking operations (PDF 1.1) (`SC`).
    SetStrokeColorOperator,
    /// Set color for nonstroking operations (PDF 1.1) (`sc`).
    SetNonStrokeColorOperator,
    /// Set color for stroking operations (ICCBasedOperator, special color spacesOperator, PDF 1.2) (`SCN`).
    SetStrokeColorICCSpecialOperator,
    /// Set color for nonstroking operations (ICCBasedOperator, special color spacesOperator, PDF 1.2) (`scn`).
    SetNonStrokeColorICCSpecialOperator,
    /// Paint area defined by shading pattern (PDF 1.3) (`sh`).
    ShadeFillOperator,
    /// Move to start of next text line (`T*`).
    TextNextLineOperator,
    /// Set character spacing (`Tc`).
    SetCharSpacingOperator,
    /// Move text position (`Td`).
    MoveTextPositionOperator,
    /// Move text position and set leading (`TD`).
    MoveTextSetLeadingOperator,
    /// Set text font and size (`Tf`).
    SetTextFontOperator,
    /// Show text (`Tj`).
    ShowTextOperator,
    /// Show textOperator, allowing individual glyph positioning (`TJ`).
    ShowTextAdjustedOperator,
    /// Set text leading (`TL`).
    SetTextLeadingOperator,
    /// Set text matrix and text line matrix (`Tm`).
    SetTextMatrixOperator,
    /// Set text rendering mode (`Tr`).
    SetTextRenderingModeOperator,
    /// Set text rise (`Ts`).
    SetTextRiseOperator,
    /// Set word spacing (`Tw`).
    SetWordSpacingOperator,
    /// Set horizontal text scaling (`Tz`).
    SetHorizontalScalingOperator,
    /// Append curved segment to path (initial point replicated) (`v`).
    CurveToInitialReplicatedOperator,
    /// Set line width (`w`).
    SetLineWidthOperator,
    /// Set clipping path using non-zero winding number rule (`W`).
    ClipOperator,
    /// Set clipping path using even-odd rule (`W*`).
    EvenOddClipOperator,
    /// Append curved segment to path (final point replicated) (`y`).
    CurveToFinalReplicatedOperator,

    // EOF
    EndOfFileToken,

    // trivia
    // \r, \n, \r\n
    EndOfLineTrivia,
    /// Null, horizontal tab, form feed, vertical tab, space, non-breaking space
    WhitespaceTrivia,
    /// % Comment 1
    CommentTrivia,

    // primitives
    IntegerLiteralToken,
    RealLiteralToken,
    NameLiteralToken,
    StringLiteralToken,
    HexStringLiteralToken,

    // primary expressions
    NumericLiteralExpression,
    NameLiteralExpression,
    StringLiteralExpression,
    HexStringLiteralExpression,
    TrueLiteralExpression,
    FalseLiteralExpression,
    NullLiteralExpression,

    // direct object expressions
    DirectObjectExpression,
    ArrayExpression,
    ArrayElementExpression,
    DictionaryExpression,
    DictionaryElementExpression,

    // indirect object expressions
    IndirectObjectExpression,
    IndirectObjectDefinition,
    IndirectObjectBody,

    // indirect reference expressions
    IndirectReferenceExpression,

    // stream expressions
    StreamExpression,
    StreamBodyExpression,
    RawStreamBytesExpression,
    OperandOperatorExpression,
    OperatorExpression,

    // cross-reference expressions
    XRefTableExpression,
    XRefSectionExpression,
    XRefSubSectionExpression,
    XRefEntryExpression,

    // trailer expressions
    FileTrailerExpression,
}

pub mod syntax_kind_facts {
    use crate::SyntaxKind;

    pub fn get_text(kind: SyntaxKind) -> &'static [u8] {
        match kind {
            SyntaxKind::TrueKeyword => b"true",
            SyntaxKind::FalseKeyword => b"false",
            SyntaxKind::NullKeyword => b"null",
            SyntaxKind::IndirectObjectKeyword => b"obj",
            SyntaxKind::IndirectEndObjectKeyword => b"endobj",
            SyntaxKind::IndirectReferenceKeyword => b"R",
            SyntaxKind::StreamKeyword => b"stream",
            SyntaxKind::EndStreamKeyword => b"endstream",
            SyntaxKind::XRefKeyword => b"xref",
            SyntaxKind::XRefFreeEntryKeyword => b"f",
            SyntaxKind::XRefInUseEntryKeyword => b"n",
            SyntaxKind::FileTrailerKeyword => b"trailer",
            SyntaxKind::StartXRefKeyword => b"startxref",
            SyntaxKind::OpenBracketToken => b"[",
            SyntaxKind::CloseBracketToken => b"]",
            SyntaxKind::OpenDictToken => b"<<",
            SyntaxKind::CloseDictToken => b">>",
            SyntaxKind::CloseFillStrokePathOperator => b"b",
            SyntaxKind::FillStrokePathOperator => b"B",
            SyntaxKind::CloseFillStrokePathEvenOddOperator => b"b*",
            SyntaxKind::FillStrokePathEvenOddOperator => b"B*",
            SyntaxKind::BeginMarkedContentPropertyOperator => b"BDC",
            SyntaxKind::BeginInlineImageOperator => b"BI",
            SyntaxKind::BeginMarkedContentOperator => b"BMC",
            SyntaxKind::BeginTextOperator => b"BT",
            SyntaxKind::BeginCompatibilityOperator => b"BX",
            SyntaxKind::CurveToOperator => b"c",
            SyntaxKind::ConcatMatrixOperator => b"cm",
            SyntaxKind::SetStrokeColorSpaceOperator => b"CS",
            SyntaxKind::SetNonStrokeColorSpaceOperator => b"cs",
            SyntaxKind::SetDashPatternOperator => b"d",
            SyntaxKind::SetCharWidthOperator => b"d0",
            SyntaxKind::SetCacheDeviceOperator => b"d1",
            SyntaxKind::InvokeXObjectOperator => b"Do",
            SyntaxKind::DefineMarkedContentPropertyOperator => b"DP",
            SyntaxKind::EndInlineImageOperator => b"EI",
            SyntaxKind::EndMarkedContentOperator => b"EMC",
            SyntaxKind::EndTextOperator => b"ET",
            SyntaxKind::EndCompatibilityOperator => b"EX",
            SyntaxKind::FillPathOperator => b"f",
            SyntaxKind::FillPathDeprecatedOperator => b"F",
            SyntaxKind::FillPathEvenOddOperator => b"f*",
            SyntaxKind::SetStrokeGrayOperator => b"G",
            SyntaxKind::SetNonStrokeGrayOperator => b"g",
            SyntaxKind::SetGraphicsStateParametersOperator => b"gs",
            SyntaxKind::CloseSubpathOperator => b"h",
            SyntaxKind::SetFlatnessToleranceOperator => b"i",
            SyntaxKind::BeginInlineImageDataOperator => b"ID",
            SyntaxKind::SetLineJoinOperator => b"j",
            SyntaxKind::SetLineCapOperator => b"J",
            SyntaxKind::SetStrokeCMYKColorOperator => b"K",
            SyntaxKind::SetNonStrokeCMYKColorOperator => b"k",
            SyntaxKind::LineToOperator => b"l",
            SyntaxKind::MoveToOperator => b"m",
            SyntaxKind::SetMiterLimitOperator => b"M",
            SyntaxKind::DefineMarkedContentPointOperator => b"MP",
            SyntaxKind::EndPathOperator => b"n",
            SyntaxKind::SaveGraphicsStateOperator => b"q",
            SyntaxKind::RestoreGraphicsStateOperator => b"Q",
            SyntaxKind::RectangleOperator => b"re",
            SyntaxKind::SetStrokeRGBColorOperator => b"RG",
            SyntaxKind::SetNonStrokeRGBColorOperator => b"rg",
            SyntaxKind::SetRenderingIntentOperator => b"ri",
            SyntaxKind::CloseStrokePathOperator => b"s",
            SyntaxKind::StrokePathOperator => b"S",
            SyntaxKind::SetStrokeColorOperator => b"SC",
            SyntaxKind::SetNonStrokeColorOperator => b"sc",
            SyntaxKind::SetStrokeColorICCSpecialOperator => b"SCN",
            SyntaxKind::SetNonStrokeColorICCSpecialOperator => b"scn",
            SyntaxKind::ShadeFillOperator => b"sh",
            SyntaxKind::TextNextLineOperator => b"T*",
            SyntaxKind::SetCharSpacingOperator => b"Tc",
            SyntaxKind::MoveTextPositionOperator => b"Td",
            SyntaxKind::MoveTextSetLeadingOperator => b"TD",
            SyntaxKind::SetTextFontOperator => b"Tf",
            SyntaxKind::ShowTextOperator => b"Tj",
            SyntaxKind::ShowTextAdjustedOperator => b"TJ",
            SyntaxKind::SetTextLeadingOperator => b"TL",
            SyntaxKind::SetTextMatrixOperator => b"Tm",
            SyntaxKind::SetTextRenderingModeOperator => b"Tr",
            SyntaxKind::SetTextRiseOperator => b"Ts",
            SyntaxKind::SetWordSpacingOperator => b"Tw",
            SyntaxKind::SetHorizontalScalingOperator => b"Tz",
            SyntaxKind::CurveToInitialReplicatedOperator => b"v",
            SyntaxKind::SetLineWidthOperator => b"w",
            SyntaxKind::ClipOperator => b"W",
            SyntaxKind::EvenOddClipOperator => b"W*",
            SyntaxKind::CurveToFinalReplicatedOperator => b"y",
            _ => b"",
        }
    }
}
