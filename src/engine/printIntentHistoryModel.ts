/** Print history restores metadata without replaying CAD geometry or presentation. */
export function printHistoryMechanicalModel(model: Record<string, unknown>): Record<string, unknown> {
  const result = {...model};
  for (const key of ['print_intent', 'counters', 'visibility', 'views', 'body_appearances', 'preferences', 'cam']) delete result[key];
  const assembly = result.assembly;
  if (assembly && typeof assembly === 'object' && !Array.isArray(assembly)) {
    const copy = {...assembly} as Record<string, unknown>;
    const structure = copy.component_structure;
    if (structure && typeof structure === 'object' && !Array.isArray(structure)) {
      const structureCopy = {...structure} as Record<string, unknown>;
      delete structureCopy.next_occurrence_id;
      copy.component_structure = structureCopy;
    }
    result.assembly = copy;
  }
  return result;
}

export function canonicalHistoryJson(value: unknown): string {
  const sorted = (item: unknown): unknown => {
    if (Array.isArray(item)) return item.map(sorted);
    if (item !== null && typeof item === 'object') return Object.fromEntries(
      Object.entries(item).sort(([left], [right]) => left.localeCompare(right)).map(([key, child]) => [key, sorted(child)]),
    );
    return item;
  };
  return JSON.stringify(sorted(value));
}

/** Geometry Redo must retain metadata edited or undone after that snapshot. */
export function replayWithCurrentPrintIntent(replay: string, current: string): string {
  const model = JSON.parse(replay) as Record<string, unknown>;
  const live = JSON.parse(current) as Record<string, unknown>;
  if (live.print_intent !== undefined) model.print_intent = live.print_intent;
  return JSON.stringify(model);
}
