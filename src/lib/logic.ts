import type {
  ConditionExpression,
  ConditionOperator,
  MacroValue,
  ValueOperand,
  ValueType,
} from "$lib/types";

export const conditionOperatorLabels: Record<ConditionOperator, string> = {
  truthy: "is truthy",
  equals: "equals",
  notEquals: "not equals",
  greaterThan: "greater than",
  greaterThanOrEqual: "greater than or equal",
  lessThan: "less than",
  lessThanOrEqual: "less than or equal",
  contains: "contains",
  startsWith: "starts with",
  endsWith: "ends with",
};

export const defaultConditionExpression: ConditionExpression = {
  left: { source: "literal", value: true },
  operator: "truthy",
  right: { source: "literal", value: true },
};

export function defaultValueForType(type: ValueType): MacroValue {
  switch (type) {
    case "boolean":
      return false;
    case "number":
      return 0;
    case "point":
      return { x: 0, y: 0 };
    case "list":
      return [];
    case "key":
    case "mouseButton":
    case "text":
      return "";
  }
}

export function parseLiteral(raw: string): MacroValue {
  const value = raw.trim();
  if (value === "") return "";
  if (value === "true") return true;
  if (value === "false") return false;
  if (value === "null") return null;

  const numeric = Number(value);
  if (!Number.isNaN(numeric) && value !== "") return numeric;

  if (value.startsWith("[") || value.startsWith("{")) {
    try {
      return JSON.parse(value) as MacroValue;
    } catch {
      return value;
    }
  }

  return value;
}

export function coerceValueByType(value: MacroValue, type: ValueType): MacroValue {
  switch (type) {
    case "boolean":
      return Boolean(value);
    case "number": {
      const numeric = Number(value);
      return Number.isNaN(numeric) ? 0 : numeric;
    }
    case "point": {
      if (
        value &&
        typeof value === "object" &&
        !Array.isArray(value) &&
        "x" in value &&
        "y" in value
      ) {
        return {
          x: Number(value.x) || 0,
          y: Number(value.y) || 0,
        };
      }
      return { x: 0, y: 0 };
    }
    case "list":
      return Array.isArray(value) ? value : value === "" || value === null ? [] : [value];
    case "key":
    case "mouseButton":
    case "text":
      return value === null ? "" : String(value);
  }
}

export function valueToInput(value: MacroValue): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function operandToInput(operand?: ValueOperand): string {
  if (!operand) return "";
  if (operand.source === "variable") return "";
  return valueToInput(operand.value ?? "");
}

export function parseOperandInput(raw: string): ValueOperand {
  return {
    source: "literal",
    value: parseLiteral(raw),
  };
}

export function expressionNeedsRightOperand(operator: ConditionOperator) {
  return operator !== "truthy";
}

export function evaluateConditionValues(
  left: MacroValue,
  operator: ConditionOperator,
  right: MacroValue,
) {
  switch (operator) {
    case "truthy":
      return Boolean(left);
    case "equals":
      return left === right;
    case "notEquals":
      return left !== right;
    case "greaterThan":
      return Number(left) > Number(right);
    case "greaterThanOrEqual":
      return Number(left) >= Number(right);
    case "lessThan":
      return Number(left) < Number(right);
    case "lessThanOrEqual":
      return Number(left) <= Number(right);
    case "contains":
      return String(left).includes(String(right));
    case "startsWith":
      return String(left).startsWith(String(right));
    case "endsWith":
      return String(left).endsWith(String(right));
  }
}
