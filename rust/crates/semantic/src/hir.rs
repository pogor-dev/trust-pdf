use std::str;

use la_arena::Idx;

pub type ExprId = Idx<Expr>;
pub type CatalogId = Idx<Catalog>;
pub type PageTreeId = Idx<PageTreeNode>;
pub type PageId = Idx<Page>;
pub type OutlineId = Idx<OutlineNode>;
pub type StructTreeId = Idx<StructNode>;
pub type NamesId = Idx<NamesNode>;

/* PRIMITIVE OBJECTS */

#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(Box<[u8]>),
    HexString(Box<[u8]>),
    Name(Box<[u8]>),
    Null,
}

impl Eq for Literal {}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Real(a), Self::Real(b)) => {
                // PDF spec: treat NaN as equal to NaN for document consistency
                (a.is_nan() && b.is_nan()) || a == b
            }
            _ => std::mem::discriminant(self) == std::mem::discriminant(other) && self == other,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Array(Box<[ExprId]>),
    Dictionary(Box<[DictionaryEntry]>),
    IndirectReference { object_number: u32, generation_number: u16 },
    IndirectObject { object_number: u32, generation_number: u16, body: ExprId },
    Stream { dict: Option<ExprId>, content: ExprId },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DictionaryEntry {
    pub key: ExprId,
    pub value: ExprId,
}

/* FILE STRUCTURE */

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FileStructure {
    Header { version: (u8, u8) },
    Body { objects: Box<[ExprId]> },
    XRefTable { entries: Box<[XRefSection]> },
    Trailer { dict: ExprId },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct XRefSection {
    pub subsections: Box<[XRefSubSection]>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct XRefSubSection {
    pub first_object_number: u32,
    pub object_count: u32,
    pub entries: Box<[XRefEntry]>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct XRefEntry {
    pub byte_offset: u64,
    pub generation_number: u16,
    pub in_use: bool,
}

/* DOCUMENT STRUCTURE */

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Catalog {
    pub page_tree: Option<PageTreeId>,
    pub outlines: Option<OutlineId>,
    pub struct_tree: Option<StructTreeId>,
    pub metadata: Option<ExprId>, // XML stream
    pub names: Option<NamesId>,
    pub interactive_form: Option<ExprId>,
    pub collections: Option<ExprId>,
    pub additional_entries: Box<[DictionaryEntry]>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PageTreeNode {
    Branch { kids: Box<[PageTreeId]>, count: u32 },
    Leaf { page: PageId },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Page {
    pub contents: Vec<[ExprId; 1]>, // `/Contents`, `/PieceInfo`, ...
    pub resources: Option<ExprId>,
    pub annotations: Box<[ExprId]>,
    pub thumb: Option<ExprId>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OutlineNode {
    pub parent: Option<OutlineId>,
    pub first_child: Option<OutlineId>,
    pub next: Option<OutlineId>,
    pub dest: Option<ExprId>, // `/Dest` or `/A`
    pub title: ExprId,        // literal string name object
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructNode {
    pub role: ExprId,              // `/S`
    pub kids: Box<[StructTreeId]>, // `/K`
    pub attributes: Box<[ExprId]>, // `/A`
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NamesNode {
    Branch { kids: Box<[NamesId]> },
    Leaf { names: Box<[NameTreeEntry]> },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameTreeEntry {
    pub name: ExprId,
    pub value: ExprId,
}
