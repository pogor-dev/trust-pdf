<div align="center">
  <img src="assets/trust-pdf-logo.png" alt="TRust PDF Logo" width="400">
</div>

Rust library for reading, viewing, editing and validating the PDF files.

Currently in development.

## Roadmap

| Category                     | Subsystem / Feature                   | Specification Reference        | Description                                                                  | Implementation Status |
| ---------------------------- | ------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------- | --------------------- |
| **File Structure**           | Header / Version / Comments           | §7.5.2 – 7.5.3 / §7.2.4        | `%PDF-n.m` header, comment syntax and lexical rules                          | ☐ Planned             |
|                              | Cross-reference Table and Trailer     | §7.5.4 – 7.5.5                 | Traditional `xref` tables, `trailer`, `startxref`, `%%EOF`                   | ☐ Planned             |
|                              | Cross-reference Streams & Hybrid Refs | §7.5.8 / Tables 17–19          | Compressed xref streams, hybrid compatibility                                | ☐ Planned             |
|                              | Incremental Updates                   | §7.5.6                         | Append-only updates, multiple trailers                                       | ☐ Planned             |
|                              | Linearization (Optimized PDF)         | Annex F / Tables F.1 etc.      | Support for linearized (“Fast Web View”) PDFs                                | ☐ Planned             |
| **Lexical & Syntax Layer**   | Tokens and Delimiters                 | §7.2 / Tables 1–2              | Whitespace, delimiter, and comment handling                                  | ☐ Planned             |
|                              | Object Model (Core 9 Types)           | §7.3.1 – §7.3.10               | boolean, number, string, name, array, dictionary, stream, null, indirect obj | 🚧 In progress        |
| **Streams & Filters**        | Stream Encoding/Decoding              | §7.4 / Tables 4–14             | Flate, LZW, ASCII85, RunLength, JBIG2, DCT, JPX, Crypt filters               | ☐ Planned             |
|                              | Object Streams                        | §7.5.7                         | Object compression containers                                                | ☐ Planned             |
| **Encryption & Security**    | Standard Security Handler             | §7.6.4 / Table 22              | Password-based encryption, permissions                                       | ☐ Planned             |
|                              | Public-Key Encryption & Crypt Filters | §7.6.5 – §7.6.6 / Tables 23–27 | CMS-based encryption using PKCS#7 / AES / ECC                                | ☐ Planned             |
|                              | File Identifiers & Metadata Integrity | §14.4 / §7.5.5                 | ID array for document integrity                                              | ☐ Planned             |
| **Graphics Model**           | Coordinate Systems & Paths            | §8.1 – §8.4                    | Path construction and painting operators                                     | ☐ Planned             |
|                              | Color Spaces & Transparency           | §8.6 / §11                     | Device, CIE-based colors, blend modes                                        | ☐ Planned             |
|                              | Text Objects and Fonts                | §9 / §10                       | Text showing, font subsets, glyph metrics                                    | ☐ Planned             |
|                              | Images and XObjects                   | §8.9 / §8.10                   | Image XObjects and Form XObjects                                             | ☐ Planned             |
| **Interactive Features**     | Annotations & Actions                 | §12.5 / §12.6                  | Links, widgets, movie/sound actions                                          | ☐ Planned             |
|                              | Forms (AcroForm / XFA)                | §12.7                          | Form fields, FDF support                                                     | ☐ Planned             |
|                              | Multimedia / 3D / Rich Media          | §13 / §13.6 – 13.7             | 3D PRC, U3D, Rich Media annotations                                          | ☐ Planned             |
| **Structure & Semantics**    | Document Catalog and Page Tree        | §7.7.2 / §7.7.3                | Root object, page hierarchy, resources                                       | ☐ Planned             |
|                              | Logical Structure / Tagged PDF        | §14.7 / §14.8                  | Structure elements, role maps, accessibility                                 | ☐ Planned             |
|                              | Associated Files (AF) and Parts       | §14.13 / §14.12                | AF relationships to pages, objects, structure                                | ☐ Planned             |
|                              | Metadata (XMP / Info Dict)            | §14.3 / §14.4                  | Document info and XMP metadata                                               | ☐ Planned             |
| **Digital Signatures**       | Signature Fields / Certs              | §12.8 / Annex A                | CMS (CAdES), LTV, DSS, DTS                                                   | ☐ Planned             |
|                              | Validation & Timestamping             | §12.8.4 – 12.8.5               | Long-term validation and timestamps                                          | ☐ Planned             |
| **Extensions & Conformance** | Namespaces & Extensions Mechanism     | Annex E                        | Vendor extensions and compatibility                                          | ☐ Planned             |
|                              | PDF/A, PDF/X, PDF/E Profiles          | Annex A References             | Conformance to ISO 19005 (PDF/A), etc.                                       | ☐ Planned             |
