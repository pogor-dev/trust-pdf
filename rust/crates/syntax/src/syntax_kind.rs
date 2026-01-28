#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    None,
    List,
    BadToken,

    /// `%PDF-1.7`
    PdfVersionToken,

    /// `%%EOF`
    EndOfFileMarkerToken,

    EndOfFileToken,

    // trivia
    // \r, \n, \r\n
    EndOfLineTrivia,
    /// Null, horizontal tab, form feed, vertical tab, space, non-breaking space
    WhitespaceTrivia,
    /// % Comment 1
    CommentTrivia,

    // primitives
    NumericLiteralToken,
    NameLiteralToken,
    StringLiteralToken,
    HexStringLiteralToken,

    // keywords
    TrueKeyword,
    FalseKeyword,
    NullKeyword,
    IndirectObjectKeyword,
    IndirectEndObjectKeyword,
    IndirectReferenceKeyword,
    StreamKeyword,
    EndStreamKeyword,
    RawStreamDataToken,
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
    IndirectObjectHeaderExpression,
    IndirectObjectBodyExpression,

    // indirect reference expressions
    IndirectReferenceExpression,

    // stream expressions
    StreamExpression,
    StreamBodyExpression,
    StreamRawDataExpression,
    StreamOperandOperatorExpression,

    // stream content block expressions
    TextObjectExpression,
    InlineImageExpression,
    MarkedContentExpression,
    CompatibilityExpression,

    // cross-reference expressions
    XRefTableExpression,
    XRefSectionExpression,
    XRefSubSectionExpression,
    XRefEntryExpression,

    // trailer expressions
    FileTrailerExpression,
    FileTrailerStartXrefExpression,

    // document and structural elements
    PdfDocument,
    PdfDocumentElementExpression,
    PdfVersionExpression,

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
}
