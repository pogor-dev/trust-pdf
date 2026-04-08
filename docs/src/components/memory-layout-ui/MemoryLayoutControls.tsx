type MemoryLayoutControlsProps = {
  showPadding: boolean
  onShowPaddingChange: (value: boolean) => void
}

export function MemoryLayoutControls(props: MemoryLayoutControlsProps) {
  return (
    <div class="flex justify-end">
      <label class="inline-flex items-center gap-3 rounded-2xl border border-white/10 bg-white/4 px-4 py-3 text-sm font-medium text-white/80 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
        <input
          class="h-4 w-4 rounded border-white/20 bg-transparent accent-indigo-500"
          type="checkbox"
          checked={props.showPadding}
          onChange={event => props.onShowPaddingChange(event.currentTarget.checked)}
        />
        <span>Show compiler padding</span>
      </label>
    </div>
  )
}
