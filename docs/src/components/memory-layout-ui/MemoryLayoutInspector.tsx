import { For, Show } from "solid-js"

import {
  formatBytes,
  formatOffset,
  type LayoutRange,
  type MemoryLayoutItem,
} from "../memory-layout"

type MemoryLayoutInspectorProps = {
  item: MemoryLayoutItem
  ancestors: LayoutRange[]
}

export function MemoryLayoutInspector(props: MemoryLayoutInspectorProps) {
  return (
    <section class="rounded-[1.35rem] border border-white/10 bg-white/4 p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
      <div class="flex flex-wrap gap-2">
        <For each={props.ancestors}>
          {ancestor => (
            <span class="rounded-full border border-white/8 bg-white/5 px-2.5 py-1 text-[0.68rem] uppercase tracking-[0.14em] text-white/45">
              {ancestor.label}
            </span>
          )}
        </For>
      </div>

      <div class="mt-4">
        <h4 class="text-xl font-semibold text-white">{props.item.label}</h4>
        <Show when={props.item.description}>
          <p class="mt-2 text-sm leading-6 text-white/65">{props.item.description}</p>
        </Show>
      </div>

      <dl class="mt-5 grid grid-cols-[5.5rem_1fr] gap-x-3 gap-y-3 text-sm">
        <MetaRow label="Offset" value={formatOffset(props.item.offset)} />
        <MetaRow label="Size" value={formatBytes(props.item.size)} />
        <MetaRow label="Align" value={`${props.item.align} bytes`} />
        <MetaRow label="Kind" value={props.item.kind} />
        <Show when={props.item.typeLabel}>
          <MetaRow label="Type" value={props.item.typeLabel ?? ""} />
        </Show>
      </dl>
    </section>
  )
}

function MetaRow(props: { label: string; value: string }) {
  return (
    <>
      <dt class="text-white/40">{props.label}</dt>
      <dd class="m-0 font-medium text-white/85">{props.value}</dd>
    </>
  )
}
