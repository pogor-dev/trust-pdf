export interface MemoryStructDefinition {
  name: string
  description?: string
  align?: number
  packed?: boolean
  fields: MemoryFieldDefinition[]
}

export interface MemoryLeafFieldDefinition {
  kind?: "field"
  name: string
  size: number
  align?: number
  label?: string
  typeLabel?: string
  description?: string
  color?: string
}

export interface MemoryStructFieldDefinition {
  kind: "struct"
  name: string
  struct: MemoryStructDefinition
  align?: number
  label?: string
  typeLabel?: string
  description?: string
  color?: string
}

export type MemoryFieldDefinition = MemoryLeafFieldDefinition | MemoryStructFieldDefinition

export interface ComputedMemoryLayout {
  name: string
  description?: string
  size: number
  align: number
  rootPath: string
  ranges: LayoutRange[]
  segments: LayoutSegment[]
}

export interface LayoutRange {
  kind: "range"
  path: string
  parentPath?: string
  name: string
  label: string
  description?: string
  typeLabel?: string
  offset: number
  size: number
  align: number
  depth: number
  color: string
}

export interface LayoutSegment {
  kind: "field" | "padding"
  path: string
  parentPath?: string
  name: string
  label: string
  description?: string
  typeLabel?: string
  offset: number
  size: number
  align: number
  depth: number
  color: string
}

export interface MemoryLayoutRows {
  totalRows: number
  rows: MemoryLayoutRow[]
}

export interface MemoryLayoutRow {
  index: number
  offset: number
  label: string
  fragments: MemoryLayoutFragment[]
}

export interface MemoryLayoutFragment {
  path: string
  parentPath?: string
  kind: "field" | "padding"
  label: string
  typeLabel?: string
  description?: string
  offset: number
  size: number
  start: number
  span: number
  color: string
  depth: number
  continuesFromPrevious: boolean
  continuesToNext: boolean
  showLabel: boolean
  active: boolean
  dimmed: boolean
}

export type MemoryLayoutItem = LayoutRange | LayoutSegment

export const MEMORY_LAYOUT_BYTES_PER_ROW = 8

const PADDING_COLOR = "linear-gradient(135deg, rgba(136, 118, 63, 0.92), rgba(98, 72, 21, 0.96))"

export function computeMemoryLayout(definition: MemoryStructDefinition): ComputedMemoryLayout {
  const segments: LayoutSegment[] = []
  const ranges: LayoutRange[] = []
  const rootPath = "root"
  const { size, align } = buildStruct(definition, {
    baseOffset: 0,
    depth: 0,
    path: rootPath,
    parentPath: undefined,
    color: definitionColor(definition.name, 0),
    rangeLabel: definition.name,
    rangeTypeLabel: definition.name,
    rangeDescription: definition.description,
    segments,
    ranges,
  })

  return {
    name: definition.name,
    description: definition.description,
    size,
    align,
    rootPath,
    ranges,
    segments,
  }
}

export function createMemoryLayoutRows(
  layout: ComputedMemoryLayout,
  showPadding = true,
  selectedPath = layout.rootPath,
): MemoryLayoutRows {
  const rows: MemoryLayoutRow[] = Array.from(
    { length: Math.max(1, Math.ceil(layout.size / MEMORY_LAYOUT_BYTES_PER_ROW)) },
    (_, index) => ({
      index,
      offset: index * MEMORY_LAYOUT_BYTES_PER_ROW,
      label: formatOffset(index * MEMORY_LAYOUT_BYTES_PER_ROW),
      fragments: [],
    }),
  )

  for (const segment of layout.segments) {
    if (!showPadding && segment.kind === "padding") {
      continue
    }

    let consumed = 0
    while (consumed < segment.size) {
      const absoluteOffset = segment.offset + consumed
      const rowIndex = Math.floor(absoluteOffset / MEMORY_LAYOUT_BYTES_PER_ROW)
      const start = absoluteOffset % MEMORY_LAYOUT_BYTES_PER_ROW
      const span = Math.min(segment.size - consumed, MEMORY_LAYOUT_BYTES_PER_ROW - start)
      const related = arePathsRelated(segment.path, selectedPath)

      rows[rowIndex]?.fragments.push({
        path: segment.path,
        parentPath: segment.parentPath,
        kind: segment.kind,
        label: segment.label,
        typeLabel: segment.typeLabel,
        description: segment.description,
        offset: segment.offset,
        size: segment.size,
        start,
        span,
        color: segment.color,
        depth: segment.depth,
        continuesFromPrevious: consumed > 0,
        continuesToNext: consumed + span < segment.size,
        showLabel: consumed === 0,
        active: related,
        dimmed: Boolean(selectedPath) && !related,
      })

      consumed += span
    }
  }

  for (const row of rows) {
    row.fragments.sort((left, right) => left.start - right.start || right.span - left.span)
  }

  return {
    totalRows: rows.length,
    rows,
  }
}

export function findMemoryLayoutItem(
  layout: ComputedMemoryLayout,
  path?: string,
): MemoryLayoutItem | undefined {
  if (!path) {
    return undefined
  }

  return (
    layout.segments.find(segment => segment.path === path) ??
    layout.ranges.find(range => range.path === path)
  )
}

export function collectAncestorRanges(layout: ComputedMemoryLayout, path?: string): LayoutRange[] {
  const ancestors: LayoutRange[] = []
  let cursor = findMemoryLayoutItem(layout, path)

  while (cursor?.parentPath) {
    const parent = layout.ranges.find(range => range.path === cursor?.parentPath)
    if (!parent) {
      break
    }

    ancestors.unshift(parent)
    cursor = parent
  }

  const selfRange = layout.ranges.find(range => range.path === path)
  if (selfRange) {
    return [...ancestors, selfRange]
  }

  return ancestors
}

export function formatBytes(size: number): string {
  return `${size} byte${size === 1 ? "" : "s"}`
}

export function formatOffset(offset: number): string {
  return `0x${offset.toString(16).toUpperCase().padStart(4, "0")}`
}

export function arePathsRelated(path?: string, selectedPath?: string): boolean {
  if (!path || !selectedPath) {
    return false
  }

  return (
    path === selectedPath ||
    path.startsWith(`${selectedPath}.`) ||
    selectedPath.startsWith(`${path}.`)
  )
}

function buildStruct(
  definition: MemoryStructDefinition,
  context: {
    baseOffset: number
    depth: number
    path: string
    parentPath?: string
    color: string
    rangeLabel: string
    rangeTypeLabel?: string
    rangeDescription?: string
    segments: LayoutSegment[]
    ranges: LayoutRange[]
  },
): { size: number; align: number } {
  const fieldAlignments = definition.fields.map(field => resolveFieldAlignment(field))
  const structAlign = normalizeAlignment(
    definition.align ?? (definition.packed ? 1 : Math.max(1, ...fieldAlignments, 1)),
  )
  let cursor = 0
  let paddingIndex = 0

  definition.fields.forEach((field, index) => {
    const fieldKind = isStructField(field) ? "struct" : "field"
    const fieldAlign = definition.packed ? 1 : resolveFieldAlignment(field)
    const alignedOffset = alignTo(cursor, fieldAlign)

    if (alignedOffset > cursor) {
      const size = alignedOffset - cursor
      context.segments.push({
        kind: "padding",
        path: `${context.path}.__padding${paddingIndex}`,
        parentPath: context.path,
        name: `padding-${paddingIndex}`,
        label: "padding",
        description: `Compiler-inserted padding before ${field.name}.`,
        offset: context.baseOffset + cursor,
        size,
        align: 1,
        depth: context.depth + 1,
        color: PADDING_COLOR,
      })
      paddingIndex += 1
      cursor = alignedOffset
    }

    const path = `${context.path}.${slugify(field.name)}-${index}`
    const color = field.color ?? definitionColor(path, context.depth + 1)

    if (fieldKind === "field") {
      const primitive = field as MemoryLeafFieldDefinition
      const size = primitive.size
      if (size <= 0) {
        throw new Error(`Field "${primitive.name}" must have a positive size.`)
      }

      context.segments.push({
        kind: "field",
        path,
        parentPath: context.path,
        name: primitive.name,
        label: primitive.label ?? primitive.name,
        description: primitive.description,
        typeLabel: primitive.typeLabel,
        offset: context.baseOffset + cursor,
        size,
        align: fieldAlign,
        depth: context.depth + 1,
        color,
      })
      cursor += size
      return
    }

    const nested = field as MemoryStructFieldDefinition
    const nestedLayout = buildStruct(nested.struct, {
      baseOffset: context.baseOffset + cursor,
      depth: context.depth + 1,
      path,
      parentPath: context.path,
      color,
      rangeLabel: nested.label ?? nested.name,
      rangeTypeLabel: nested.typeLabel ?? nested.struct.name,
      rangeDescription: nested.description ?? nested.struct.description,
      segments: context.segments,
      ranges: context.ranges,
    })
    cursor += nestedLayout.size
  })

  const finalSize = alignTo(cursor, structAlign)
  if (finalSize > cursor) {
    context.segments.push({
      kind: "padding",
      path: `${context.path}.__padding${paddingIndex}`,
      parentPath: context.path,
      name: `padding-${paddingIndex}`,
      label: "padding",
      description: `Trailing padding to align ${context.rangeLabel}.`,
      offset: context.baseOffset + cursor,
      size: finalSize - cursor,
      align: 1,
      depth: context.depth + 1,
      color: PADDING_COLOR,
    })
  }

  context.ranges.push({
    kind: "range",
    path: context.path,
    parentPath: context.parentPath,
    name: definition.name,
    label: context.rangeLabel,
    description: context.rangeDescription,
    typeLabel: context.rangeTypeLabel,
    offset: context.baseOffset,
    size: finalSize,
    align: structAlign,
    depth: context.depth,
    color: context.color,
  })

  return { size: finalSize, align: structAlign }
}

function isStructField(field: MemoryFieldDefinition): field is MemoryStructFieldDefinition {
  return field.kind === "struct"
}

function resolveFieldAlignment(field: MemoryFieldDefinition): number {
  if (isStructField(field)) {
    return normalizeAlignment(field.align ?? resolveStructAlignment(field.struct))
  }

  return normalizeAlignment(field.align ?? inferAlignmentFromSize(field.size))
}

function resolveStructAlignment(definition: MemoryStructDefinition): number {
  if (definition.packed) {
    return normalizeAlignment(definition.align ?? 1)
  }

  const alignments = definition.fields.map(field => resolveFieldAlignment(field))
  return normalizeAlignment(definition.align ?? Math.max(1, ...alignments, 1))
}

function inferAlignmentFromSize(size: number): number {
  if (size <= 1) {
    return 1
  }

  let alignment = 1
  while (alignment * 2 <= size && alignment < 16) {
    alignment *= 2
  }

  return alignment
}

function alignTo(offset: number, alignment: number): number {
  const normalizedAlignment = normalizeAlignment(alignment)
  return Math.ceil(offset / normalizedAlignment) * normalizedAlignment
}

function normalizeAlignment(alignment: number): number {
  if (!Number.isFinite(alignment) || alignment <= 0) {
    return 1
  }

  return Math.max(1, Math.floor(alignment))
}

function definitionColor(seed: string, depth: number): string {
  const hash = Array.from(seed).reduce(
    (accumulator, character) => {
      return (accumulator * 31 + character.charCodeAt(0)) % 360
    },
    19 + depth * 17,
  )

  const saturation = Math.max(52, 68 - depth * 3)
  const lightnessA = Math.max(36, 48 - depth * 2)
  const lightnessB = Math.min(72, lightnessA + 14)
  return `linear-gradient(135deg, hsl(${hash} 72% ${lightnessB}% / 0.95), hsl(${(hash + 18) % 360} ${saturation}% ${lightnessA}% / 0.98))`
}

function slugify(value: string): string {
  return (
    value
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "field"
  )
}
