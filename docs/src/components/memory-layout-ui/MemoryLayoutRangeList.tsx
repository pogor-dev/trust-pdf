import { For } from "solid-js"

import { formatBytes, formatOffset, type LayoutRange } from "./memory-layout"

type MemoryLayoutRangeListProps = {
  ranges: LayoutRange[]
  selectedPath: string
  onSelect: (path: string) => void
}

export function MemoryLayoutRangeList(props: MemoryLayoutRangeListProps) {
  return (
    <section class="rounded-[1.35rem] border border-white/10 bg-white/4 p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
      <div class="mb-3 flex items-center justify-between gap-3">
        <h4 class="text-sm font-semibold uppercase tracking-[0.16em] text-white/55">
          Nested Ranges
        </h4>
        <p class="text-xs text-white/35">Click to focus</p>
      </div>

      <div class="grid gap-2">
        <For each={props.ranges}>
          {range => (
            <button
              type="button"
              class="grid gap-1 rounded-2xl border px-3 py-3 text-left transition duration-150 hover:translate-x-0.5"
              classList={{
                "border-white/20 bg-white/[0.07]": props.selectedPath === range.path,
                "border-white/8 bg-white/[0.02]": props.selectedPath !== range.path,
              }}
              style={{
                "background-image":
                  props.selectedPath === range.path
                    ? `${range.color}, linear-gradient(180deg, rgba(255,255,255,0.05), rgba(255,255,255,0.02))`
                    : `${range.color}, linear-gradient(180deg, rgba(255,255,255,0.03), rgba(255,255,255,0.01))`,
                "background-size": `${Math.min(16, 8 + range.depth * 2)}px 100%, 100% 100%`,
                "background-position": "left top, left top",
                "background-repeat": "no-repeat, no-repeat",
              }}
              onClick={() => props.onSelect(range.path)}
            >
              <span class="truncate text-sm font-semibold text-white/90">{range.label}</span>
              <span class="text-xs text-white/55">
                {formatOffset(range.offset)} · {formatBytes(range.size)}
              </span>
            </button>
          )}
        </For>
      </div>
    </section>
  )
}
