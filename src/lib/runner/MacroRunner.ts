import { invoke } from "@tauri-apps/api/core";
import type { Edge, Node } from "@xyflow/svelte";
import { register, unregisterAll, type ShortcutEvent } from "@tauri-apps/plugin-global-shortcut";
import {
  FLOW_OUT_HANDLE,
  edgeKind,
  submacroOutputHandle,
  valueHandle,
  valueHandleName,
} from "$lib/graph";
import {
  defaultConditionExpression,
  defaultValueForType,
  evaluateConditionValues,
} from "$lib/logic";
import type {
  ConditionExpression,
  MacroValue,
  SetVariableNodeData,
  SubmacroDefinition,
  UpdateVariableNodeData,
  ValueOperand,
  VariableDefinition,
  VariableScope,
} from "$lib/types";

type VariableSnapshot = Record<string, MacroValue>;
type SnapshotCallback = (snapshot: VariableSnapshot) => void;

class LoopBreakSignal {}
class LoopContinueSignal {}

function keyFor(nodeId: string, handle?: string | null) {
  return `${nodeId}:${handle ?? FLOW_OUT_HANDLE}`;
}

class GraphIndex {
  readonly nodesById = new Map<string, Node>();
  readonly triggerOutgoing = new Map<string, Edge[]>();
  readonly valueIncoming = new Map<string, Edge>();
  readonly boundaryOutputs = new Map<string, string[]>();
  readonly boundaryValueInputs = new Map<string, string>();

  constructor(
    nodes: Node[],
    edges: Edge[],
    boundaryOutputs: SubmacroDefinition["boundaryOutputs"] = [],
    boundaryValueInputs: SubmacroDefinition["boundaryValueInputs"] = [],
  ) {
    for (const node of nodes) {
      this.nodesById.set(node.id, node);
    }

    for (const edge of edges) {
      if (edgeKind(edge) === "value") {
        if (edge.targetHandle) {
          this.valueIncoming.set(keyFor(edge.target, edge.targetHandle), edge);
        }
        continue;
      }

      const mapKey = keyFor(edge.source, edge.sourceHandle);
      const list = this.triggerOutgoing.get(mapKey);
      if (list) {
        list.push(edge);
      } else {
        this.triggerOutgoing.set(mapKey, [edge]);
      }
    }

    for (const output of boundaryOutputs) {
      const mapKey = keyFor(output.sourceNodeId, output.sourceHandle);
      const list = this.boundaryOutputs.get(mapKey);
      if (list) {
        list.push(output.outputId);
      } else {
        this.boundaryOutputs.set(mapKey, [output.outputId]);
      }
    }

    for (const input of boundaryValueInputs) {
      this.boundaryValueInputs.set(keyFor(input.targetNodeId, input.targetHandle), input.inputId);
    }
  }

  getTriggerEdges(nodeId: string, sourceHandle?: string | null) {
    return this.triggerOutgoing.get(keyFor(nodeId, sourceHandle)) ?? [];
  }

  getValueEdge(nodeId: string, targetHandle: string) {
    return this.valueIncoming.get(keyFor(nodeId, targetHandle));
  }

  getBoundaryOutputIds(nodeId: string, sourceHandle?: string | null) {
    return this.boundaryOutputs.get(keyFor(nodeId, sourceHandle)) ?? [];
  }

  getBoundaryValueInputId(nodeId: string, targetHandle: string) {
    return this.boundaryValueInputs.get(keyFor(nodeId, targetHandle));
  }
}

class ExecutionContext {
  readonly values = new Map<string, MacroValue>();
  readonly definitions = new Map<string, VariableDefinition>();
  readonly submacroOutputs = new Map<string, Record<string, MacroValue>>();
  readonly triggeredOutputs = new Set<string>();
  stepCount = 0;

  constructor(
    readonly graph: GraphIndex,
    variables: VariableDefinition[],
    readonly parent?: ExecutionContext,
    readonly inputValues: Record<string, MacroValue> = {},
  ) {
    for (const variable of variables) {
      this.definitions.set(variable.name, variable);
      this.values.set(variable.name, this.initialValue(variable));
    }
  }

  private initialValue(variable: VariableDefinition) {
    if (variable.scope === "persistent" && typeof localStorage !== "undefined") {
      const raw = localStorage.getItem(this.storageKey(variable.name));
      if (raw !== null) {
        try {
          return JSON.parse(raw) as MacroValue;
        } catch {
          return structuredClone(variable.defaultValue);
        }
      }
    }

    return structuredClone(variable.defaultValue);
  }

  hasLocalVariable(name: string) {
    return this.values.has(name);
  }

  hasVariable(name: string): boolean {
    return this.values.has(name) || Boolean(this.parent?.hasVariable(name));
  }

  getVariable(name: string): MacroValue {
    if (this.values.has(name)) return this.values.get(name) ?? null;
    return this.parent?.getVariable(name) ?? null;
  }

  setVariable(name: string, value: MacroValue, scope: VariableScope = "macro") {
    if (scope !== "local" && this.parent?.hasVariable(name) && !this.values.has(name)) {
      this.parent.setVariable(name, value, scope);
      return;
    }

    this.values.set(name, value);
    const definition = this.definitions.get(name);
    if (
      (scope === "persistent" || definition?.scope === "persistent") &&
      typeof localStorage !== "undefined"
    ) {
      localStorage.setItem(this.storageKey(name), JSON.stringify(value));
    }
  }

  snapshot(): VariableSnapshot {
    const values = this.parent?.snapshot() ?? {};
    for (const [name, value] of this.values) {
      values[name] = value;
    }
    return values;
  }

  private storageKey(name: string) {
    return `macro-center:persistent:${name}`;
  }
}

export class MacroRunner {
  private nodes: Node[];
  private edges: Edge[];
  private variables: VariableDefinition[];
  private submacros: SubmacroDefinition[];
  private graph: GraphIndex;
  private submacroGraphs = new Map<string, GraphIndex>();
  private isRunning = false;
  private registeredShortcuts: string[] = [];
  private rootContext?: ExecutionContext;
  private readonly maxSteps = 50_000;

  constructor(
    nodes: Node[],
    edges: Edge[],
    variables: VariableDefinition[] = [],
    submacros: SubmacroDefinition[] = [],
    private readonly onSnapshot?: SnapshotCallback,
  ) {
    this.nodes = nodes;
    this.edges = edges;
    this.variables = variables;
    this.submacros = submacros;
    this.graph = new GraphIndex(nodes, edges);
    this.rebuildSubmacroIndexes();
  }

  public updateGraph(
    nodes: Node[],
    edges: Edge[],
    variables: VariableDefinition[] = this.variables,
    submacros: SubmacroDefinition[] = this.submacros,
  ) {
    this.nodes = nodes;
    this.edges = edges;
    this.variables = variables;
    this.submacros = submacros;
    this.graph = new GraphIndex(nodes, edges);
    this.rebuildSubmacroIndexes();
    void this.refreshShortcuts();
  }

  public async refreshShortcuts() {
    try {
      await unregisterAll();
      this.registeredShortcuts = [];

      for (const node of this.nodes) {
        if (node.type !== "keyBindNode") continue;

        const shortcut = node.data.shortcut as string | undefined;
        if (!shortcut) continue;

        try {
          await register(shortcut, (event: ShortcutEvent) => {
            if (event.state === "Pressed") {
              void this.runMacro(node.id);
            }
          });
          this.registeredShortcuts.push(shortcut);
        } catch (error) {
          console.error(`Failed to register shortcut ${shortcut}:`, error);
        }
      }
    } catch (error) {
      console.error("Error refreshing shortcuts:", error);
    }
  }

  public async runMacro(startNodeId: string) {
    if (this.isRunning) return;
    this.isRunning = true;

    const context = new ExecutionContext(this.graph, this.variables);
    this.rootContext = context;
    this.emitSnapshot(context);

    try {
      await this.executeNode(startNodeId, context);
    } catch (error) {
      if (error instanceof LoopBreakSignal || error instanceof LoopContinueSignal) {
        console.error("Break/Continue was used outside of a loop.");
      } else {
        console.error("Macro execution error:", error);
      }
    } finally {
      this.isRunning = false;
      this.emitSnapshot(context);
    }
  }

  private rebuildSubmacroIndexes() {
    const graphs = new Map<string, GraphIndex>();
    for (const submacro of this.submacros) {
      graphs.set(
        submacro.id,
        new GraphIndex(
          submacro.nodes,
          submacro.edges,
          submacro.boundaryOutputs,
          submacro.boundaryValueInputs,
        ),
      );
    }
    this.submacroGraphs = graphs;
  }

  private async executeNode(nodeId: string, context: ExecutionContext): Promise<void> {
    if (++context.stepCount > this.maxSteps) {
      throw new Error(`Macro stopped after ${this.maxSteps} execution steps.`);
    }

    const node = context.graph.nodesById.get(nodeId);
    if (!node) return;

    switch (node.type) {
      case "typeNode":
        await this.executeTypeNode(node, context);
        break;
      case "keyNode":
        await this.executeKeyNode(node, context);
        break;
      case "mousePressNode":
        await this.executeMousePressNode(node, context);
        break;
      case "mouseMoveNode":
        await this.executeMouseMoveNode(node, context);
        break;
      case "scrollMouseNode":
        await this.executeScrollMouseNode(node, context);
        break;
      case "delayNode":
        await this.executeDelayNode(node, context);
        break;
      case "setVariableNode":
        this.executeSetVariableNode(node, context);
        break;
      case "updateVariableNode":
        this.executeUpdateVariableNode(node, context);
        break;
      case "conditionalNode":
        await this.executeConditionalNode(node, context);
        return;
      case "repeatLoopNode":
        await this.executeRepeatLoopNode(node, context);
        return;
      case "forEachLoopNode":
        await this.executeForEachLoopNode(node, context);
        return;
      case "whileLoopNode":
        await this.executeWhileLoopNode(node, context);
        return;
      case "breakLoopNode":
        throw new LoopBreakSignal();
      case "continueLoopNode":
        throw new LoopContinueSignal();
      case "submacroNode":
        await this.executeSubmacroNode(node, context);
        return;
      case "keyBindNode":
      case "getVariableNode":
      case "compareNode":
      case "valueNode":
        break;
    }

    await this.continueFrom(node.id, FLOW_OUT_HANDLE, context);
  }

  private async continueFrom(
    nodeId: string,
    sourceHandle: string | null | undefined,
    context: ExecutionContext,
  ) {
    for (const outputId of context.graph.getBoundaryOutputIds(nodeId, sourceHandle)) {
      context.triggeredOutputs.add(outputId);
    }

    for (const edge of context.graph.getTriggerEdges(nodeId, sourceHandle)) {
      await this.executeNode(edge.target, context);
    }
  }

  private async executeTypeNode(node: Node, context: ExecutionContext) {
    const text = String(this.getInputValue(node, "text", node.data.text ?? "", context));
    await invoke("simulate_type_text", { text });
  }

  private async executeKeyNode(node: Node, context: ExecutionContext) {
    const key = String(this.getInputValue(node, "key", node.data.key ?? "", context));
    const mode = String(node.data.mode ?? "click");
    await invoke("simulate_key_action", { key, mode });
  }

  private async executeMousePressNode(node: Node, context: ExecutionContext) {
    const button = String(this.getInputValue(node, "button", node.data.button ?? "left", context));
    const mode = String(node.data.mode ?? "click");
    await invoke("simulate_mouse_click", { button, mode });
  }

  private async executeMouseMoveNode(node: Node, context: ExecutionContext) {
    const x = Number(this.getInputValue(node, "x", node.data.x ?? 0, context)) || 0;
    const y = Number(this.getInputValue(node, "y", node.data.y ?? 0, context)) || 0;
    const mode = String(node.data.coordinateMode ?? "absolute");
    await invoke("simulate_mouse_move", { x, y, mode });
  }

  private async executeScrollMouseNode(node: Node, context: ExecutionContext) {
    const axis = String(node.data.axis ?? "vertical");
    const amount = Number(this.getInputValue(node, "amount", node.data.amount ?? 0, context)) || 0;
    await invoke("simulate_scroll", { axis, amount });
  }

  private async executeDelayNode(node: Node, context: ExecutionContext) {
    const delayMs =
      Number(this.getInputValue(node, "delay", node.data.delay ?? 1000, context)) || 0;
    await this.delay(Math.max(0, delayMs));
  }

  private executeSetVariableNode(node: Node, context: ExecutionContext) {
    const data = node.data as SetVariableNodeData;
    const name = data.variableName?.trim();
    if (!name) return;

    const fallback = data.value ?? defaultValueForType(data.valueType ?? "text");
    const value = this.getInputValue(node, "value", fallback, context);
    context.setVariable(name, value, data.scope ?? "macro");
    this.emitSnapshot(this.rootContext ?? context);
  }

  private executeUpdateVariableNode(node: Node, context: ExecutionContext) {
    const data = node.data as UpdateVariableNodeData;
    const name = data.variableName?.trim();
    if (!name) return;

    const current = context.getVariable(name);
    const inputValue = this.getInputValue(node, "value", data.value ?? 1, context);
    let next: MacroValue = current;

    switch (data.operation ?? "increment") {
      case "increment":
        next = Number(current) + Number(inputValue);
        break;
      case "decrement":
        next = Number(current) - Number(inputValue);
        break;
      case "set":
        next = inputValue;
        break;
      case "append":
        next = `${current ?? ""}${inputValue ?? ""}`;
        break;
      case "toggle":
        next = !current;
        break;
      case "clear":
        next = null;
        break;
    }

    context.setVariable(name, next);
    this.emitSnapshot(this.rootContext ?? context);
  }

  private async executeConditionalNode(node: Node, context: ExecutionContext) {
    const result = this.evaluateConditionForNode(node, context);
    await this.continueFrom(node.id, result ? "true" : "false", context);
  }

  private async executeRepeatLoopNode(node: Node, context: ExecutionContext) {
    const iterations = Math.max(
      0,
      Math.floor(
        Number(this.getInputValue(node, "iterations", node.data.iterations ?? 0, context)) || 0,
      ),
    );
    const indexName = String(node.data.indexVariable ?? "index");

    for (let index = 0; index < iterations; index += 1) {
      context.setVariable(indexName, index);
      this.emitSnapshot(this.rootContext ?? context);

      try {
        await this.continueFrom(node.id, "body", context);
      } catch (error) {
        if (error instanceof LoopBreakSignal) break;
        if (error instanceof LoopContinueSignal) continue;
        throw error;
      }
    }

    await this.continueFrom(node.id, "done", context);
  }

  private async executeForEachLoopNode(node: Node, context: ExecutionContext) {
    const rawItems = this.getInputValue(node, "items", node.data.items ?? [], context);
    const items = Array.isArray(rawItems) ? rawItems : [];
    const itemName = String(node.data.itemVariable ?? "item");
    const indexName = String(node.data.indexVariable ?? "index");

    for (let index = 0; index < items.length; index += 1) {
      context.setVariable(itemName, items[index] ?? null);
      context.setVariable(indexName, index);
      this.emitSnapshot(this.rootContext ?? context);

      try {
        await this.continueFrom(node.id, "body", context);
      } catch (error) {
        if (error instanceof LoopBreakSignal) break;
        if (error instanceof LoopContinueSignal) continue;
        throw error;
      }
    }

    await this.continueFrom(node.id, "done", context);
  }

  private async executeWhileLoopNode(node: Node, context: ExecutionContext) {
    const maxIterations = Math.max(0, Math.floor(Number(node.data.maxIterations ?? 100) || 0));
    const indexName = String(node.data.indexVariable ?? "index");

    for (let index = 0; index < maxIterations; index += 1) {
      context.setVariable(indexName, index);
      this.emitSnapshot(this.rootContext ?? context);

      if (!this.evaluateConditionForNode(node, context)) break;

      try {
        await this.continueFrom(node.id, "body", context);
      } catch (error) {
        if (error instanceof LoopBreakSignal) break;
        if (error instanceof LoopContinueSignal) continue;
        throw error;
      }
    }

    await this.continueFrom(node.id, "done", context);
  }

  private async executeSubmacroNode(node: Node, context: ExecutionContext) {
    const definitionId = String(node.data.definitionId ?? "");
    const definition = this.submacros.find((submacro) => submacro.id === definitionId);
    const graph = this.submacroGraphs.get(definitionId);
    if (!definition || !graph) {
      await this.continueFrom(node.id, FLOW_OUT_HANDLE, context);
      return;
    }

    const inputValues: Record<string, MacroValue> = {};
    for (const input of definition.valueInputs) {
      inputValues[input.id] = this.getInputValue(
        node,
        input.id,
        input.defaultValue ?? defaultValueForType(input.type),
        context,
      );
    }

    const childContext = new ExecutionContext(graph, definition.variables, context, inputValues);

    for (const entryNodeId of definition.entryNodeIds) {
      await this.executeNode(entryNodeId, childContext);
    }

    const valueOutputs: Record<string, MacroValue> = {};
    for (const output of definition.valueOutputBindings) {
      valueOutputs[output.outputId] = this.evaluateValueSource(
        output.sourceNodeId,
        output.sourceHandle,
        childContext,
      );
    }
    context.submacroOutputs.set(node.id, valueOutputs);

    if (childContext.triggeredOutputs.size === 0) {
      const defaultOutput = definition.triggerOutputs[0]?.id ?? "done";
      await this.continueFrom(node.id, submacroOutputHandle(defaultOutput), context);
      return;
    }

    for (const outputId of childContext.triggeredOutputs) {
      await this.continueFrom(node.id, submacroOutputHandle(outputId), context);
    }
  }

  private getInputValue(
    node: Node,
    inputName: string,
    fallback: unknown,
    context: ExecutionContext,
  ): MacroValue {
    const targetHandle = valueHandle(inputName);
    const boundaryInputId = context.graph.getBoundaryValueInputId(node.id, targetHandle);
    if (boundaryInputId) {
      return context.inputValues[boundaryInputId] ?? (fallback as MacroValue);
    }

    const incomingEdge = context.graph.getValueEdge(node.id, targetHandle);
    if (!incomingEdge) return fallback as MacroValue;

    return this.evaluateValueSource(incomingEdge.source, incomingEdge.sourceHandle, context);
  }

  private hasInputValue(node: Node, inputName: string, context: ExecutionContext) {
    const targetHandle = valueHandle(inputName);
    return (
      Boolean(context.graph.getBoundaryValueInputId(node.id, targetHandle)) ||
      Boolean(context.graph.getValueEdge(node.id, targetHandle))
    );
  }

  private evaluateValueSource(
    nodeId: string,
    sourceHandle: string | null | undefined,
    context: ExecutionContext,
  ): MacroValue {
    const node = context.graph.nodesById.get(nodeId);
    if (!node) return null;

    const outputName = valueHandleName(sourceHandle) ?? "value";

    switch (node.type) {
      case "getVariableNode":
        return context.getVariable(String(node.data.variableName ?? ""));
      case "compareNode":
        return this.evaluateConditionForNode(node, context);
      case "valueNode":
        return (node.data.value ?? null) as MacroValue;
      case "submacroNode":
        return context.submacroOutputs.get(node.id)?.[outputName] ?? null;
      case "setVariableNode":
      case "updateVariableNode":
        return context.getVariable(String(node.data.variableName ?? ""));
      default: {
        const value = node.data[outputName];
        return value === undefined ? null : (value as MacroValue);
      }
    }
  }

  private evaluateConditionForNode(node: Node, context: ExecutionContext) {
    if (this.hasInputValue(node, "condition", context)) {
      return Boolean(this.getInputValue(node, "condition", false, context));
    }

    const expression =
      (node.data.conditionExpression as ConditionExpression | undefined) ??
      this.legacyConditionExpression(node.data.condition as string | undefined);

    const left = this.hasInputValue(node, "left", context)
      ? this.getInputValue(node, "left", null, context)
      : this.evaluateOperand(expression.left, context);
    const right = this.hasInputValue(node, "right", context)
      ? this.getInputValue(node, "right", null, context)
      : this.evaluateOperand(expression.right, context);

    return evaluateConditionValues(left, expression.operator, right);
  }

  private evaluateOperand(
    operand: ValueOperand | undefined,
    context: ExecutionContext,
  ): MacroValue {
    if (!operand) return null;
    if (operand.source === "variable") {
      return context.getVariable(operand.variableName ?? "");
    }
    return operand.value ?? null;
  }

  private legacyConditionExpression(condition?: string): ConditionExpression {
    if (condition === "false") {
      return {
        left: { source: "literal", value: false },
        operator: "truthy",
      };
    }

    if (condition && condition !== "true") {
      return {
        left: { source: "variable", variableName: condition.replace(/^\$/, "") },
        operator: "truthy",
      };
    }

    return defaultConditionExpression;
  }

  private emitSnapshot(context: ExecutionContext) {
    this.onSnapshot?.(context.snapshot());
  }

  private delay(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  public cleanup() {
    void unregisterAll();
  }
}
