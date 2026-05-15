import { invoke } from "@tauri-apps/api/core";
import type { Node } from "@xyflow/svelte";
import type { MacroValue, ShellCommandNodeData } from "$lib/types";

export type ShellCommandOutput = {
  exitCode: number | null;
  stdout: string;
  stderr: string;
};

interface ShellCommandContext {
  shellOutputs: Map<string, ShellCommandOutput>;
}

type ResolveInputValue = (node: Node, inputName: string, fallback: unknown) => MacroValue;

export async function executeShellCommandNode(
  node: Node,
  context: ShellCommandContext,
  getInputValue: ResolveInputValue,
) {
  const data = node.data as ShellCommandNodeData;
  const command = String(getInputValue(node, "command", data.command ?? ""));
  const timeoutMs = Math.max(0, Number(getInputValue(node, "timeoutMs", data.timeoutMs ?? 0)) || 0);
  const env = String(getInputValue(node, "env", data.env ?? ""));
  const cwd = String(getInputValue(node, "cwd", data.cwd ?? "")).trim();

  try {
    const output = await invoke<ShellCommandOutput>("execute_shell_command", {
      command,
      timeoutMs,
      env,
      cwd,
      runInBackground: Boolean(data.runInBackground),
    });
    context.shellOutputs.set(node.id, output);
  } catch (error) {
    context.shellOutputs.set(node.id, {
      exitCode: 1,
      stdout: "",
      stderr: error instanceof Error ? error.message : String(error),
    });
  }
}
