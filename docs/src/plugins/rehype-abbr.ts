import type { Element, Root, RootContent, Text } from "hast"
import type { Plugin } from "unified"
import { visit } from "unist-util-visit"

const DEFAULT_ABBREVIATIONS: ReadonlyArray<RehypeAbbreviationEntry> = [
  { abbr: "EOL", title: "End of line", matchCase: true },
]

const DEFAULT_TARGET_TAGS = new Set(["p"])
const DEFAULT_BLOCKED_TAGS = new Set(["code", "pre", "script", "style", "abbr"])

export interface RehypeAbbreviationEntry {
  abbr: string
  title: string
  matchCase?: boolean
}

export interface RehypeAbbrOptions {
  abbreviations?: RehypeAbbreviationEntry[]
  targetTags?: string[]
  blockedTags?: string[]
}

interface NormalizedAbbreviation {
  key: string
  title: string
  matchCase: boolean
}

interface CompiledAbbreviations {
  sensitiveMap: Record<string, string>
  insensitiveMap: Record<string, string>
  sensitivePattern: RegExp | null
  insensitivePattern: RegExp | null
}

interface MatchResult {
  index: number
  value: string
  title: string
}

const rehypeAbbr: Plugin<[RehypeAbbrOptions?], Root> = (options = {}) => {
  const normalizedAbbreviations = normalizeAbbreviations(
    options.abbreviations ?? Array.from(DEFAULT_ABBREVIATIONS),
  )
  const compiledAbbreviations = compileAbbreviations(normalizedAbbreviations)

  if (!compiledAbbreviations.sensitivePattern && !compiledAbbreviations.insensitivePattern) {
    return () => {}
  }

  const targetTags = new Set(
    (options.targetTags ?? Array.from(DEFAULT_TARGET_TAGS)).map(normalizeTagName),
  )
  const blockedTags = new Set(
    (options.blockedTags ?? Array.from(DEFAULT_BLOCKED_TAGS)).map(normalizeTagName),
  )

  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      if (!targetTags.has(normalizeTagName(node.tagName))) {
        return
      }

      rewriteChildren(node.children, false, compiledAbbreviations, blockedTags)
    })
  }
}

export default rehypeAbbr

function rewriteChildren(
  children: RootContent[],
  blocked: boolean,
  compiledAbbreviations: CompiledAbbreviations,
  blockedTags: Set<string>,
): void {
  for (let index = 0; index < children.length; index++) {
    const child = children[index]

    if (child.type === "text") {
      if (!blocked) {
        const replacement = expandText(child.value, compiledAbbreviations)
        if (replacement !== null) {
          children.splice(index, 1, ...replacement)
          index += replacement.length - 1
        }
      }
      continue
    }

    if (child.type !== "element") {
      continue
    }

    const tagName = normalizeTagName(child.tagName)
    const nextBlocked = blocked || blockedTags.has(tagName)
    rewriteChildren(child.children, nextBlocked, compiledAbbreviations, blockedTags)
  }
}

function expandText(
  text: string,
  compiledAbbreviations: CompiledAbbreviations,
): Array<Text | Element> | null {
  if (!text) {
    return null
  }

  let lastIndex = 0
  let hasMatch = false
  const nodes: Array<Text | Element> = []

  while (true) {
    const match = findNextMatch(text, lastIndex, compiledAbbreviations)
    if (!match) {
      break
    }

    hasMatch = true

    if (match.index > lastIndex) {
      nodes.push({ type: "text", value: text.slice(lastIndex, match.index) })
    }

    nodes.push({
      type: "element",
      tagName: "abbr",
      properties: { title: match.title },
      children: [{ type: "text", value: match.value }],
    })

    lastIndex = match.index + match.value.length
  }

  if (!hasMatch) {
    return null
  }

  if (lastIndex < text.length) {
    nodes.push({ type: "text", value: text.slice(lastIndex) })
  }

  return nodes
}

function normalizeAbbreviations(
  abbreviations: RehypeAbbreviationEntry[],
): NormalizedAbbreviation[] {
  const normalized: NormalizedAbbreviation[] = []

  for (const entry of abbreviations) {
    if (!entry?.abbr || !entry?.title) {
      continue
    }

    normalized.push({
      key: entry.abbr,
      title: entry.title,
      matchCase: entry.matchCase ?? true,
    })
  }

  return normalized
}

function compileAbbreviations(abbreviations: NormalizedAbbreviation[]): CompiledAbbreviations {
  const sensitiveMap: Record<string, string> = {}
  const insensitiveMap: Record<string, string> = {}

  for (const abbreviation of abbreviations) {
    if (abbreviation.matchCase) {
      sensitiveMap[abbreviation.key] = abbreviation.title
    } else {
      insensitiveMap[abbreviation.key.toLowerCase()] = abbreviation.title
    }
  }

  return {
    sensitiveMap,
    insensitiveMap,
    sensitivePattern: createTokenPattern(Object.keys(sensitiveMap), false),
    insensitivePattern: createTokenPattern(Object.keys(insensitiveMap), true),
  }
}

function createTokenPattern(keys: string[], caseInsensitive: boolean): RegExp | null {
  if (!keys.length) {
    return null
  }

  const terms = keys
    .map(escapeRegExp)
    .sort((a, b) => b.length - a.length)
    .join("|")
  const flags = caseInsensitive ? "gi" : "g"
  return new RegExp(`(?<![A-Za-z0-9_])(${terms})(?![A-Za-z0-9_])`, flags)
}

function findNextMatch(
  text: string,
  fromIndex: number,
  compiled: CompiledAbbreviations,
): MatchResult | null {
  const sensitive = execFrom(compiled.sensitivePattern, text, fromIndex)
  const insensitive = execFrom(compiled.insensitivePattern, text, fromIndex)

  const sensitiveResult = sensitive
    ? {
        index: sensitive.index,
        value: sensitive[0],
        title: compiled.sensitiveMap[sensitive[0]],
      }
    : null

  const insensitiveResult = insensitive
    ? {
        index: insensitive.index,
        value: insensitive[0],
        title: compiled.insensitiveMap[insensitive[0].toLowerCase()],
      }
    : null

  if (!sensitiveResult && !insensitiveResult) {
    return null
  }

  if (sensitiveResult && !insensitiveResult) {
    return sensitiveResult
  }

  if (!sensitiveResult && insensitiveResult) {
    return insensitiveResult
  }

  if (!sensitiveResult || !insensitiveResult) {
    return null
  }

  if (sensitiveResult.index < insensitiveResult.index) {
    return sensitiveResult
  }

  if (insensitiveResult.index < sensitiveResult.index) {
    return insensitiveResult
  }

  if (sensitiveResult.value.length >= insensitiveResult.value.length) {
    return sensitiveResult
  }

  return insensitiveResult
}

function execFrom(pattern: RegExp | null, text: string, fromIndex: number): RegExpExecArray | null {
  if (!pattern) {
    return null
  }

  pattern.lastIndex = fromIndex
  return pattern.exec(text)
}

function normalizeTagName(value: string): string {
  return value.toLowerCase()
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}
