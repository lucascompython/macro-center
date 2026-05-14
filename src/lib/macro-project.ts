import type { Node } from "@xyflow/svelte";

export function detachedSubflowNode(node: Node) {
  const next = {
    ...node,
    data: { ...node.data },
    position: { x: node.position.x, y: node.position.y },
  } as Node & {
    parentId?: string;
    extent?: string;
    expandParent?: boolean;
  };

  delete next.parentId;
  delete next.extent;
  delete next.expandParent;
  return next;
}
