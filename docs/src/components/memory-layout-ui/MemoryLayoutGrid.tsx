import { For } from "solid-js"

import {
  MEMORY_LAYOUT_BYTES_PER_ROW,
  type MemoryLayoutFragment,
  type MemoryLayoutRows,
} from "../memory-layout"

type MemoryLayoutGridProps = {
  rows: MemoryLayoutRows
  onSelect: (path: string) => void
}

export function MemoryLayoutGrid(props: MemoryLayoutGridProps) {
  const templateColumns = () => `repeat(${MEMORY_LAYOUT_BYTES_PER_ROW}, minmax(3.5rem, 1fr))`
  const minWidth = () => `${MEMORY_LAYOUT_BYTES_PER_ROW * 74}px`

  return (
    <div class="rounded-[1.4rem] border border-white/10 bg-white/[0.035] p-3 sm:p-4">
      <div class="overflow-x-auto pb-2">
        <div class="min-w-full" style={{ "min-width": minWidth() }}>
          <div class="overflow-hidden rounded-[1.2rem] border border-white/8 bg-black/15">
            <div class="grid grid-cols-[4.5rem_minmax(0,1fr)] border-b border-white/8 bg-white/3 text-[0.7rem] uppercase tracking-[0.16em] text-white/35">
              <div class="flex items-center border-r border-white/8 px-3 py-3">Offset</div>
              <div class="grid text-center" style={{ "grid-template-columns": templateColumns() }}>
                <For each={Array.from({ length: MEMORY_LAYOUT_BYTES_PER_ROW }, (_, byte) => byte)}>
                  {byte => (
                    <span class="border-r border-white/8 px-2 py-3 last:border-r-0">{byte}</span>
                  )}
                </For>
              </div>
            </div>

            <For each={props.rows.rows}>
              {(row, index) => (
                <div
                  class="grid grid-cols-[4.5rem_minmax(0,1fr)]"
                  classList={{ "border-t border-white/8": index() > 0 }}
                >
                  <div class="flex items-center border-r border-white/8 px-3 py-4 text-[0.74rem] font-medium tracking-[0.12em] text-white/35">
                    {row.label}
                  </div>

                  <div
                    class="relative grid min-h-23"
                    style={{
                      "grid-template-columns": templateColumns(),
                      "grid-template-rows": "5.75rem",
                    }}
                  >
                    <For
                      each={Array.from(
                        { length: MEMORY_LAYOUT_BYTES_PER_ROW },
                        (_, byteIndex) => byteIndex,
                      )}
                    >
                      {byteIndex => (
                        <div
                          class="z-0 border-r border-white/8 bg-white/1.5 last:border-r-0"
                          style={{
                            "grid-column": `${byteIndex + 1} / span 1`,
                            "grid-row": "1",
                          }}
                        />
                      )}
                    </For>

                    <For each={row.fragments}>
                      {fragment => <SegmentCard fragment={fragment} onSelect={props.onSelect} />}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </div>
  )
}

function SegmentCard(props: { fragment: MemoryLayoutFragment; onSelect: (path: string) => void }) {
  const { fragment } = props
  const display = getFragmentDisplay(fragment)
  const horizontalInset = "0.35rem"
  const verticalInset = "0.5rem"
  const dividerOffsets = Array.from(
    { length: Math.max(fragment.span - 1, 0) },
    (_, index) => index + 1,
  )

  return (
    <button
      type="button"
      title={`${fragment.label} · ${fragment.size} byte${fragment.size === 1 ? "" : "s"}`}
      class="relative z-10 my-2 flex h-[calc(100%-1rem)] flex-col justify-center self-center overflow-hidden border text-left text-white transition duration-150 hover:-translate-y-0.5"
      classList={{
        "rounded-l-xl rounded-r-none": !fragment.continuesFromPrevious && fragment.continuesToNext,
        "rounded-r-xl rounded-l-none": fragment.continuesFromPrevious && !fragment.continuesToNext,
        "rounded-none": fragment.continuesFromPrevious && fragment.continuesToNext,
        "rounded-xl": !fragment.continuesFromPrevious && !fragment.continuesToNext,
        "border-white/35 shadow-[0_12px_24px_rgba(0,0,0,0.28)]": fragment.active,
        "border-white/15 shadow-[0_10px_22px_rgba(0,0,0,0.18)]": !fragment.active,
        "opacity-45": fragment.dimmed,
      }}
      style={{
        "grid-column": `${fragment.start + 1} / span ${fragment.span}`,
        "grid-row": "1",
        margin: `${verticalInset} ${horizontalInset}`,
        "background-image": fragment.color,
        "background-size": "100% 100%",
      }}
      onClick={() => props.onSelect(fragment.path)}
    >
      {fragment.continuesFromPrevious ? <RowBreakSeam side="left" /> : null}
      {fragment.continuesToNext ? <RowBreakSeam side="right" /> : null}
      <For each={dividerOffsets}>
        {offset => (
          <div
            class="pointer-events-none absolute top-0 bottom-0 z-10 border-r border-white/14"
            style={{
              left: `calc(((100% + (${horizontalInset} * 2)) * ${offset} / ${fragment.span}) - ${horizontalInset})`,
            }}
          />
        )}
      </For>
      <div class="absolute inset-0 bg-[linear-gradient(180deg,rgba(255,255,255,0.16),rgba(255,255,255,0))] opacity-60" />
      <div class="relative z-30 flex h-full flex-col justify-center px-3">
        <span class="truncate text-sm font-semibold leading-tight text-white">
          {display.primary}
        </span>
        {display.secondary ? (
          <span class="mt-1 truncate text-[0.68rem] font-medium uppercase tracking-[0.12em] text-white/75">
            {display.secondary}
          </span>
        ) : null}
      </div>
    </button>
  )
}

function RowBreakSeam(props: { side: "left" | "right" }) {
  const edgeStyle = props.side === "left" ? { left: "-1px" } : { right: "-1px" }

  return (
    <div
      class="pointer-events-none absolute -top-px -bottom-px z-20 w-3.5 bg-center bg-no-repeat"
      style={{
        ...edgeStyle,
        "background-image": rowBreakSeamImage(props.side),
        "background-size": "100% 100%",
      }}
    />
  )
}

function rowBreakSeamImage(side: "left" | "right"): string {
  if (side === "left") {
    return `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 96' preserveAspectRatio='none'%3E%3Cpolyline points='15,0 5,0 1,7 5,14 1,21 5,28 1,35 5,42 1,49 5,56 1,63 5,70 1,77 5,84 1,91 5,96 15,96' fill='none' stroke='rgba(8,12,18,0.55)' stroke-width='3.2' stroke-linejoin='round' stroke-linecap='round'/%3E%3Cpolyline points='15,0 5,0 1,7 5,14 1,21 5,28 1,35 5,42 1,49 5,56 1,63 5,70 1,77 5,84 1,91 5,96 15,96' fill='none' stroke='rgba(255,255,255,0.9)' stroke-width='1.5' stroke-linejoin='round' stroke-linecap='round'/%3E%3C/svg%3E")`
  }

  return `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 96' preserveAspectRatio='none'%3E%3Cpolyline points='1,0 11,0 15,7 11,14 15,21 11,28 15,35 11,42 15,49 11,56 15,63 11,70 15,77 11,84 15,91 11,96 1,96' fill='none' stroke='rgba(8,12,18,0.55)' stroke-width='3.2' stroke-linejoin='round' stroke-linecap='round'/%3E%3Cpolyline points='1,0 11,0 15,7 11,14 15,21 11,28 15,35 11,42 15,49 11,56 15,63 11,70 15,77 11,84 15,91 11,96 1,96' fill='none' stroke='rgba(255,255,255,0.9)' stroke-width='1.5' stroke-linejoin='round' stroke-linecap='round'/%3E%3C/svg%3E")`
}

function getFragmentDisplay(fragment: MemoryLayoutFragment): {
  primary: string
  secondary?: string
} {
  if (fragment.span <= 1) {
    return { primary: abbreviateLabel(fragment.label) }
  }

  if (fragment.span === 2) {
    return { primary: fragment.label }
  }

  if (fragment.kind === "padding") {
    return { primary: fragment.label, secondary: `${fragment.size}B` }
  }

  return {
    primary: fragment.label,
    secondary: fragment.typeLabel,
  }
}

function abbreviateLabel(label: string): string {
  if (label.length <= 3) {
    return label
  }

  return (
    label
      .split(/[^a-zA-Z0-9]+/)
      .filter(Boolean)
      .map(part => part[0])
      .join("")
      .slice(0, 3)
      .toUpperCase() || label.slice(0, 3).toUpperCase()
  )
}
