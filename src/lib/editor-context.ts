import type { MacroValue, VariableDefinition } from "$lib/types";

export const SUBFLOW_RENAME_CONTEXT = Symbol("macro-center-subflow-rename");
export const VALUE_SOURCE_CONTEXT = Symbol("macro-center-value-source");

export type RenameSubflow = (definitionId: string, name: string) => void;

export interface ValueSourceContext {
  getVariables: () => VariableDefinition[];
  getSnapshot: () => Record<string, MacroValue>;
}
