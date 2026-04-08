import { createMemo, createSignal } from "solid-js"

import {
  collectAncestorRanges,
  computeMemoryLayout,
  createMemoryLayoutRows,
  findMemoryLayoutItem,
  formatBytes,
  type MemoryStructDefinition,
} from "../memory-layout"
import { MemoryLayoutControls } from "./MemoryLayoutControls"
import { MemoryLayoutGrid } from "./MemoryLayoutGrid"
import { MemoryLayoutInspector } from "./MemoryLayoutInspector"
import { MemoryLayoutRangeList } from "./MemoryLayoutRangeList"

export type MemoryLayoutAppProps = {
  definition: MemoryStructDefinition
  title?: string
  caption?: string
  showControls?: boolean
}

export default function MemoryLayoutApp(props: MemoryLayoutAppProps) {
  const layout = createMemo(() => computeMemoryLayout(props.definition))
  const [selectedPath, setSelectedPath] = createSignal(layout().rootPath)
  const [showPadding, setShowPadding] = createSignal(true)

  const rows = createMemo(() => createMemoryLayoutRows(layout(), showPadding(), selectedPath()))
  const selectedItem = createMemo(
    () => findMemoryLayoutItem(layout(), selectedPath()) ?? layout().ranges[0],
  )
  const ancestors = createMemo(() => collectAncestorRanges(layout(), selectedItem().path))
  const title = createMemo(() => props.title ?? props.definition.name)

  return (
    <section class="not-content not-prose overflow-hidden rounded-[1.75rem] border border-white/10 bg-[linear-gradient(180deg,rgba(10,14,24,0.96),rgba(17,22,34,0.94))] shadow-[0_28px_70px_rgba(0,0,0,0.28)]">
      <div class="border-b border-white/8 bg-[radial-gradient(circle_at_top_left,rgba(104,132,255,0.18),transparent_28%),radial-gradient(circle_at_top_right,rgba(38,211,198,0.1),transparent_22%)] px-4 py-4 sm:px-6 sm:py-5">
        <div class="flex flex-col gap-4 min-[1100px]:flex-row min-[1100px]:items-start min-[1100px]:justify-between">
          <div class="max-w-3xl space-y-2">
            <p class="text-[0.68rem] font-semibold uppercase tracking-[0.24em] text-white/45">
              Interactive Memory Layout
            </p>
            <div class="space-y-2">
              <h3 class="text-2xl font-semibold leading-tight text-white sm:text-[1.9rem]">
                {title()}
              </h3>
              {props.caption ? (
                <p class="max-w-2xl text-sm leading-6 text-white/65 sm:text-[0.98rem]">
                  {props.caption}
                </p>
              ) : null}
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3 min-[1100px]:min-w-88">
            <StatCard label="Size" value={formatBytes(layout().size)} />
            <StatCard label="Alignment" value={`${layout().align} bytes`} />
            <StatCard label="Ranges" value={String(layout().ranges.length)} />
          </div>
        </div>

        {props.showControls !== false ? (
          <div class="mt-5">
            <MemoryLayoutControls
              showPadding={showPadding()}
              onShowPaddingChange={setShowPadding}
            />
          </div>
        ) : null}
      </div>

      <div class="grid gap-4 px-4 py-4 sm:px-6 sm:py-6 min-[1500px]:grid-cols-[minmax(0,1.65fr)_22rem]">
        <MemoryLayoutGrid rows={rows()} onSelect={setSelectedPath} />

        <div class="grid gap-4 self-start">
          <MemoryLayoutInspector item={selectedItem()} ancestors={ancestors()} />
          <MemoryLayoutRangeList
            ranges={layout().ranges}
            selectedPath={selectedPath()}
            onSelect={setSelectedPath}
          />
        </div>
      </div>
    </section>
  )
}

function StatCard(props: { label: string; value: string }) {
  return (
    <div class="rounded-2xl border border-white/10 bg-white/4 px-3 py-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
      <p class="text-[0.68rem] uppercase tracking-[0.18em] text-white/45">{props.label}</p>
      <p class="mt-2 text-sm font-semibold text-white sm:text-base">{props.value}</p>
    </div>
  )
}
