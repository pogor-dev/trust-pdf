import type { MemoryStructDefinition } from "./memory-layout"

export const greenTriviaHead: MemoryStructDefinition = {
  name: "GreenTriviaHead",
  description: "Compact header stored ahead of the inline trivia bytes.",
  align: 2,
  fields: [
    {
      name: "kind",
      size: 2,
      typeLabel: "SyntaxKind",
      description: "Two-byte syntax discriminator for the trivia token.",
    },
    {
      name: "flags",
      size: 1,
      typeLabel: "GreenFlags",
      description: "Packed bitflags for missing/diagnostic state.",
    },
  ],
}

export const greenTriviaData: MemoryStructDefinition = {
  name: "GreenTriviaData",
  description:
    "Borrowed trivia view used by the green tree. The nested head stays compact while the payload aligns to pointer width.",
  align: 8,
  fields: [
    {
      kind: "struct",
      name: "header",
      struct: greenTriviaHead,
      typeLabel: "GreenTriviaHead",
      description: "Header copied inline before the dynamically sized trivia text.",
    },
    {
      name: "length",
      size: 8,
      align: 8,
      typeLabel: "usize",
      description: "Slice length tracked at pointer-width alignment.",
    },
    {
      name: "slice",
      size: 8,
      align: 8,
      typeLabel: "[u8; N]",
      description:
        "Inline tail storage for the trivia bytes represented by this green node instance.",
    },
  ],
}

export const nestedSnakeDemo: MemoryStructDefinition = {
  name: "Snake",
  description:
    "A generic nested-struct example showing how the component handles multiple embedded structs and tail padding.",
  align: 4,
  fields: [
    {
      kind: "struct",
      name: "head",
      typeLabel: "Point",
      struct: {
        name: "Point",
        fields: [
          { name: "x", size: 2, typeLabel: "i16" },
          { name: "y", size: 2, typeLabel: "i16" },
        ],
      },
    },
    {
      kind: "struct",
      name: "body",
      typeLabel: "[Point; 2]",
      struct: {
        name: "BodyPoints",
        fields: [
          {
            kind: "struct",
            name: "segment0",
            typeLabel: "Point",
            struct: {
              name: "Point",
              fields: [
                { name: "x", size: 2, typeLabel: "i16" },
                { name: "y", size: 2, typeLabel: "i16" },
              ],
            },
          },
          {
            kind: "struct",
            name: "segment1",
            typeLabel: "Point",
            struct: {
              name: "Point",
              fields: [
                { name: "x", size: 2, typeLabel: "i16" },
                { name: "y", size: 2, typeLabel: "i16" },
              ],
            },
          },
        ],
      },
    },
    { name: "tail", size: 4, typeLabel: "i32" },
    { name: "score", size: 4, typeLabel: "i32" },
    { name: "symbol", size: 1, typeLabel: "char" },
    { name: "gameOver", size: 1, typeLabel: "bool" },
    { name: "food", size: 8, align: 4, typeLabel: "Point" },
  ],
}

export const rowSpanningStructDemo: MemoryStructDefinition = {
  name: "PacketFrame",
  description:
    "A nested-struct example where the timestamp field starts near the end of the first row and continues into the second row.",
  align: 4,
  fields: [
    {
      name: "version",
      size: 1,
      typeLabel: "u8",
      description: "Protocol version byte.",
    },
    {
      name: "flags",
      size: 1,
      typeLabel: "u8",
      description: "Feature flags packed into a byte.",
    },
    {
      kind: "struct",
      name: "metadata",
      typeLabel: "PacketMetadata",
      description:
        "Nested metadata begins at offset 0x0004 after alignment padding, and its leading timestamp field crosses the next row boundary.",
      struct: {
        name: "PacketMetadata",
        align: 4,
        fields: [
          {
            name: "timestamp",
            size: 8,
            typeLabel: "u64",
            description:
              "Capture timestamp stored as a u64 so the field itself spans from row 0 into row 1.",
          },
          {
            name: "payloadLength",
            size: 4,
            typeLabel: "u32",
            description: "Number of bytes in the payload.",
          },
          {
            name: "checksum",
            size: 2,
            typeLabel: "u16",
            description: "Header checksum.",
          },
          {
            name: "state",
            size: 1,
            typeLabel: "u8",
            description: "Small state discriminator.",
          },
          {
            name: "reserved",
            size: 1,
            typeLabel: "u8",
            description: "Reserved trailing byte to finish the block.",
          },
        ],
      },
    },
    {
      name: "crc",
      size: 4,
      typeLabel: "u32",
      description: "Frame-level CRC after the row-spanning timestamp and metadata block.",
    },
  ],
}
