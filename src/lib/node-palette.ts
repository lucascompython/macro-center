import { defaultConditionExpression } from "$lib/logic";

export type NodePaletteCategory = "Triggers" | "Actions" | "Logic" | "Variables";

export type NodeTemplate = {
  label: string;
  category: NodePaletteCategory;
  type: string;
  data: Record<string, unknown>;
};

export const nodeTemplates: NodeTemplate[] = [
  {
    label: "Key Bind",
    category: "Triggers",
    type: "keyBindNode",
    data: { title: "Key Bind", subline: "Trigger macro" },
  },
  {
    label: "Type Text",
    category: "Actions",
    type: "typeNode",
    data: { title: "Type Text", text: "Hello" },
  },
  {
    label: "Key Press",
    category: "Actions",
    type: "keyNode",
    data: { title: "Key Press", mode: "click" },
  },
  {
    label: "Mouse Click",
    category: "Actions",
    type: "mousePressNode",
    data: { title: "Mouse Click", button: "left" },
  },
  {
    label: "Move Mouse",
    category: "Actions",
    type: "mouseMoveNode",
    data: { title: "Move Mouse", x: 0, y: 0 },
  },
  {
    label: "Scroll Mouse",
    category: "Actions",
    type: "scrollMouseNode",
    data: { title: "Scroll Mouse", amount: 3 },
  },
  {
    label: "Delay",
    category: "Logic",
    type: "delayNode",
    data: { title: "Delay", delay: 1000 },
  },
  {
    label: "If",
    category: "Logic",
    type: "conditionalNode",
    data: { title: "If", condition: "true", conditionExpression: defaultConditionExpression },
  },
  {
    label: "Repeat",
    category: "Logic",
    type: "repeatLoopNode",
    data: { title: "Repeat", iterations: 3, indexVariable: "index" },
  },
  {
    label: "For Each",
    category: "Logic",
    type: "forEachLoopNode",
    data: { title: "For Each", items: [], itemVariable: "item", indexVariable: "index" },
  },
  {
    label: "While",
    category: "Logic",
    type: "whileLoopNode",
    data: {
      title: "While",
      conditionExpression: defaultConditionExpression,
      indexVariable: "index",
      maxIterations: 100,
    },
  },
  {
    label: "Break",
    category: "Logic",
    type: "breakLoopNode",
    data: { title: "Break" },
  },
  {
    label: "Continue",
    category: "Logic",
    type: "continueLoopNode",
    data: { title: "Continue" },
  },
  {
    label: "Value",
    category: "Variables",
    type: "valueNode",
    data: { title: "Value", valueType: "text", value: "" },
  },
  {
    label: "Set Variable",
    category: "Variables",
    type: "setVariableNode",
    data: { title: "Set Variable", variableName: "value", valueType: "number", value: 0, scope: "macro" },
  },
  {
    label: "Get Variable",
    category: "Variables",
    type: "getVariableNode",
    data: { title: "Get Variable", variableName: "value" },
  },
  {
    label: "Update Variable",
    category: "Variables",
    type: "updateVariableNode",
    data: { title: "Update Variable", variableName: "value", operation: "increment", value: 1 },
  },
  {
    label: "Compare",
    category: "Variables",
    type: "compareNode",
    data: { title: "Compare", conditionExpression: defaultConditionExpression },
  },
];

export const nodeTemplateCategories: NodePaletteCategory[] = [
  "Triggers",
  "Actions",
  "Logic",
  "Variables",
];

export function nodeTemplatesForCategory(category: NodePaletteCategory) {
  return nodeTemplates.filter((template) => template.category === category);
}

export function nodeTemplateCategoryClass(category: NodePaletteCategory) {
  switch (category) {
    case "Triggers":
      return "trigger";
    case "Actions":
      return "action";
    case "Logic":
      return "logic";
    case "Variables":
      return "variable";
  }
}
