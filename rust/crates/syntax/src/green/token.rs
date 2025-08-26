use crate::{SyntaxKind, syntax_kind_facts};

const FIRST_TOKEN_WITH_WELL_KNOWN_TEXT: SyntaxKind = SyntaxKind::TrueKeyword;
const LAST_TOKEN_WITH_WELL_KNOWN_TEXT: SyntaxKind = SyntaxKind::EndOfFileToken;

pub struct SyntaxToken<'a> {
    kind: SyntaxKind,
    full_width: usize,
    // TODO: Consider using a more efficient representation
    // TODO: arena maybe?
    text: Option<&'a [u8]>,
}

impl<'a> SyntaxToken<'a> {
    #[inline]
    pub fn new_by_kind(kind: SyntaxKind) -> Self {
        let text = syntax_kind_facts::get_text(kind);
        let full_width = text.len();
        Self {
            kind,
            full_width,
            text: Some(text),
        }
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.text.unwrap_or(&[])
    }

    #[inline]
    pub fn full_width(&self) -> usize {
        self.full_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::true_keyword(SyntaxKind::TrueKeyword, b"true")]
    #[case::false_keyword(SyntaxKind::FalseKeyword, b"false")]
    #[case::null_keyword(SyntaxKind::NullKeyword, b"null")]
    #[case::indirect_object_keyword(SyntaxKind::IndirectObjectKeyword, b"obj")]
    #[case::indirect_end_object_keyword(SyntaxKind::IndirectEndObjectKeyword, b"endobj")]
    #[case::indirect_reference_keyword(SyntaxKind::IndirectReferenceKeyword, b"R")]
    #[case::stream_keyword(SyntaxKind::StreamKeyword, b"stream")]
    #[case::end_stream_keyword(SyntaxKind::EndStreamKeyword, b"endstream")]
    #[case::xref_keyword(SyntaxKind::XRefKeyword, b"xref")]
    #[case::xref_free_entry_keyword(SyntaxKind::XRefFreeEntryKeyword, b"f")]
    #[case::xref_in_use_entry_keyword(SyntaxKind::XRefInUseEntryKeyword, b"n")]
    #[case::file_trailer_keyword(SyntaxKind::FileTrailerKeyword, b"trailer")]
    #[case::start_xref_keyword(SyntaxKind::StartXRefKeyword, b"startxref")]
    #[case::open_bracket_token(SyntaxKind::OpenBracketToken, b"[")]
    #[case::close_bracket_token(SyntaxKind::CloseBracketToken, b"]")]
    #[case::open_dict_token(SyntaxKind::OpenDictToken, b"<<")]
    #[case::close_dict_token(SyntaxKind::CloseDictToken, b">>")]
    #[case::close_fill_stroke_path_operator(SyntaxKind::CloseFillStrokePathOperator, b"b")]
    #[case::fill_stroke_path_operator(SyntaxKind::FillStrokePathOperator, b"B")]
    #[case::close_fill_stroke_path_even_odd_operator(SyntaxKind::CloseFillStrokePathEvenOddOperator, b"b*")]
    #[case::fill_stroke_path_even_odd_operator(SyntaxKind::FillStrokePathEvenOddOperator, b"B*")]
    #[case::begin_marked_content_property_operator(SyntaxKind::BeginMarkedContentPropertyOperator, b"BDC")]
    #[case::begin_inline_image_operator(SyntaxKind::BeginInlineImageOperator, b"BI")]
    #[case::begin_marked_content_operator(SyntaxKind::BeginMarkedContentOperator, b"BMC")]
    #[case::begin_text_operator(SyntaxKind::BeginTextOperator, b"BT")]
    #[case::begin_compatibility_operator(SyntaxKind::BeginCompatibilityOperator, b"BX")]
    #[case::curve_to_operator(SyntaxKind::CurveToOperator, b"c")]
    #[case::concat_matrix_operator(SyntaxKind::ConcatMatrixOperator, b"cm")]
    #[case::set_stroke_color_space_operator(SyntaxKind::SetStrokeColorSpaceOperator, b"CS")]
    #[case::set_non_stroke_color_space_operator(SyntaxKind::SetNonStrokeColorSpaceOperator, b"cs")]
    #[case::set_dash_pattern_operator(SyntaxKind::SetDashPatternOperator, b"d")]
    #[case::set_char_width_operator(SyntaxKind::SetCharWidthOperator, b"d0")]
    #[case::set_cache_device_operator(SyntaxKind::SetCacheDeviceOperator, b"d1")]
    #[case::invoke_xobject_operator(SyntaxKind::InvokeXObjectOperator, b"Do")]
    #[case::define_marked_content_property_operator(SyntaxKind::DefineMarkedContentPropertyOperator, b"DP")]
    #[case::end_inline_image_operator(SyntaxKind::EndInlineImageOperator, b"EI")]
    #[case::end_marked_content_operator(SyntaxKind::EndMarkedContentOperator, b"EMC")]
    #[case::end_text_operator(SyntaxKind::EndTextOperator, b"ET")]
    #[case::end_compatibility_operator(SyntaxKind::EndCompatibilityOperator, b"EX")]
    #[case::fill_path_operator(SyntaxKind::FillPathOperator, b"f")]
    #[case::fill_path_deprecated_operator(SyntaxKind::FillPathDeprecatedOperator, b"F")]
    #[case::fill_path_even_odd_operator(SyntaxKind::FillPathEvenOddOperator, b"f*")]
    #[case::set_stroke_gray_operator(SyntaxKind::SetStrokeGrayOperator, b"G")]
    #[case::set_non_stroke_gray_operator(SyntaxKind::SetNonStrokeGrayOperator, b"g")]
    #[case::set_graphics_state_parameters_operator(SyntaxKind::SetGraphicsStateParametersOperator, b"gs")]
    #[case::close_subpath_operator(SyntaxKind::CloseSubpathOperator, b"h")]
    #[case::set_flatness_tolerance_operator(SyntaxKind::SetFlatnessToleranceOperator, b"i")]
    #[case::begin_inline_image_data_operator(SyntaxKind::BeginInlineImageDataOperator, b"ID")]
    #[case::set_line_join_operator(SyntaxKind::SetLineJoinOperator, b"j")]
    #[case::set_line_cap_operator(SyntaxKind::SetLineCapOperator, b"J")]
    #[case::set_stroke_cmyk_color_operator(SyntaxKind::SetStrokeCMYKColorOperator, b"K")]
    #[case::set_non_stroke_cmyk_color_operator(SyntaxKind::SetNonStrokeCMYKColorOperator, b"k")]
    #[case::line_to_operator(SyntaxKind::LineToOperator, b"l")]
    #[case::move_to_operator(SyntaxKind::MoveToOperator, b"m")]
    #[case::set_miter_limit_operator(SyntaxKind::SetMiterLimitOperator, b"M")]
    #[case::define_marked_content_point_operator(SyntaxKind::DefineMarkedContentPointOperator, b"MP")]
    #[case::end_path_operator(SyntaxKind::EndPathOperator, b"n")]
    #[case::save_graphics_state_operator(SyntaxKind::SaveGraphicsStateOperator, b"q")]
    #[case::restore_graphics_state_operator(SyntaxKind::RestoreGraphicsStateOperator, b"Q")]
    #[case::rectangle_operator(SyntaxKind::RectangleOperator, b"re")]
    #[case::set_stroke_rgb_color_operator(SyntaxKind::SetStrokeRGBColorOperator, b"RG")]
    #[case::set_non_stroke_rgb_color_operator(SyntaxKind::SetNonStrokeRGBColorOperator, b"rg")]
    #[case::set_rendering_intent_operator(SyntaxKind::SetRenderingIntentOperator, b"ri")]
    #[case::close_stroke_path_operator(SyntaxKind::CloseStrokePathOperator, b"s")]
    #[case::stroke_path_operator(SyntaxKind::StrokePathOperator, b"S")]
    #[case::set_stroke_color_operator(SyntaxKind::SetStrokeColorOperator, b"SC")]
    #[case::set_non_stroke_color_operator(SyntaxKind::SetNonStrokeColorOperator, b"sc")]
    #[case::set_stroke_color_icc_special_operator(SyntaxKind::SetStrokeColorICCSpecialOperator, b"SCN")]
    #[case::set_non_stroke_color_icc_special_operator(SyntaxKind::SetNonStrokeColorICCSpecialOperator, b"scn")]
    #[case::shade_fill_operator(SyntaxKind::ShadeFillOperator, b"sh")]
    #[case::text_next_line_operator(SyntaxKind::TextNextLineOperator, b"T*")]
    #[case::set_char_spacing_operator(SyntaxKind::SetCharSpacingOperator, b"Tc")]
    #[case::move_text_position_operator(SyntaxKind::MoveTextPositionOperator, b"Td")]
    #[case::move_text_set_leading_operator(SyntaxKind::MoveTextSetLeadingOperator, b"TD")]
    #[case::set_text_font_operator(SyntaxKind::SetTextFontOperator, b"Tf")]
    #[case::show_text_operator(SyntaxKind::ShowTextOperator, b"Tj")]
    #[case::show_text_adjusted_operator(SyntaxKind::ShowTextAdjustedOperator, b"TJ")]
    #[case::set_text_leading_operator(SyntaxKind::SetTextLeadingOperator, b"TL")]
    #[case::set_text_matrix_operator(SyntaxKind::SetTextMatrixOperator, b"Tm")]
    #[case::set_text_rendering_mode_operator(SyntaxKind::SetTextRenderingModeOperator, b"Tr")]
    #[case::set_text_rise_operator(SyntaxKind::SetTextRiseOperator, b"Ts")]
    #[case::set_word_spacing_operator(SyntaxKind::SetWordSpacingOperator, b"Tw")]
    #[case::set_horizontal_scaling_operator(SyntaxKind::SetHorizontalScalingOperator, b"Tz")]
    #[case::curve_to_initial_replicated_operator(SyntaxKind::CurveToInitialReplicatedOperator, b"v")]
    #[case::set_line_width_operator(SyntaxKind::SetLineWidthOperator, b"w")]
    #[case::clip_operator(SyntaxKind::ClipOperator, b"W")]
    #[case::even_odd_clip_operator(SyntaxKind::EvenOddClipOperator, b"W*")]
    #[case::curve_to_final_replicated_operator(SyntaxKind::CurveToFinalReplicatedOperator, b"y")]
    #[case::none(SyntaxKind::None, b"")]
    #[case::end_of_file_token(SyntaxKind::EndOfFileToken, b"")]
    #[case::integer_literal_token(SyntaxKind::IntegerLiteralToken, b"")]
    #[case::real_literal_token(SyntaxKind::RealLiteralToken, b"")]
    #[case::name_literal_token(SyntaxKind::NameLiteralToken, b"")]
    #[case::string_literal_token(SyntaxKind::StringLiteralToken, b"")]
    #[case::hex_string_literal_token(SyntaxKind::HexStringLiteralToken, b"")]
    fn test_new_by_kind_should_get_interned_text(#[case] kind: SyntaxKind, #[case] expected_text: &[u8]) {
        let token = SyntaxToken::new_by_kind(kind);
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), expected_text);
        assert_eq!(token.full_width(), expected_text.len());
    }

    #[test]
    fn test_new_by_kind_when_call_multiple_times_text_should_return_same_slice() {
        let token = SyntaxToken::new_by_kind(SyntaxKind::TrueKeyword);
        let text1 = token.text();
        let text2 = token.text();
        assert_eq!(text1, text2);
    }
}
