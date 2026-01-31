import { invoke } from "@tauri-apps/api/core";
import type { Node, Edge } from "@xyflow/svelte";
import {
    register,
    unregisterAll,
    isRegistered,
    type ShortcutEvent,
} from "@tauri-apps/plugin-global-shortcut";

export class MacroRunner {
    private nodes: Node[];
    private edges: Edge[];
    private isRunning: boolean = false;
    private registeredShortcuts: string[] = [];

    constructor(nodes: Node[], edges: Edge[]) {
        this.nodes = nodes;
        this.edges = edges;
    }

    public updateGraph(nodes: Node[], edges: Edge[]) {
        this.nodes = nodes;
        this.edges = edges;
        this.refreshShortcuts();
    }

    public async refreshShortcuts() {
        console.log("Refreshing shortcuts...");
        try {
            // we might want to be more selective,
            // but for now unregister all to avoid conflicts
            await unregisterAll();
            this.registeredShortcuts = [];

            const keyBindNodes = this.nodes.filter((n) => n.type === "keyBindNode");

            for (const node of keyBindNodes) {
                const shortcut = node.data.shortcut as string;
                if (!shortcut) continue;

                // basic validation or normalization could go here
                // for now assuming the input format matches accelerator requirements
                try {
                    await register(shortcut, (event: ShortcutEvent) => {
                        if (event.state === "Pressed") {
                            console.log(`Shortcut ${shortcut} triggered!`);
                            this.runMacro(node.id);
                        }
                    });
                    this.registeredShortcuts.push(shortcut);
                    console.log(`Registered shortcut: ${shortcut}`);
                } catch (e) {
                    console.error(`Failed to register shortcut ${shortcut}:`, e);
                }
            }
        } catch (e) {
            console.error("Error refreshing shortcuts:", e);
        }
    }

    public async runMacro(startNodeId: string) {
        if (this.isRunning) return; // prevent concurrent execution atleast for now
        this.isRunning = true;
        console.log(`Starting macro from node: ${startNodeId}`);

        try {
            await this.executeNode(startNodeId);
        } catch (e) {
            console.error("Macro execution error:", e);
        } finally {
            this.isRunning = false;
            console.log("Macro execution finished");
        }
    }

    private async executeNode(nodeId: string) {
        const node = this.nodes.find((n) => n.id === nodeId);
        if (!node) return;

        console.log(`Executing node: ${node.type} (${node.id})`);

        switch (node.type) {
            case "typeNode":
                await this.executeTypeNode(node);
                break;
            case "keyNode":
                await this.executeKeyNode(node);
                break;
            case "mousePressNode":
                await this.executeMousePressNode(node);
                break;
            case "mouseMoveNode":
                await this.executeMouseMoveNode(node);
                break;
            case "scrollMouseNode":
                await this.executeScrollMouseNode(node);
                break;
            case "delayNode":
                await this.executeDelayNode(node);
                break;
            case "conditionalNode":
                // conditional handles its own traversal
                await this.executeConditionalNode(node);
                return;
            case "keyBindNode":
                // just a trigger, pass through
                break;
        }

        // find next connected node(s)
        // for non-conditional nodes, we just look for outgoing edges from 'source' handle
        // or just any outgoing edge if handle not specified (default)
        const outgoingEdges = this.edges.filter((e) => e.source === nodeId);

        for (const edge of outgoingEdges) {
            // sequential execution for multiple outputs?
            // for MVP assume linear single path or parallel.
            // we'll await them sequentially.
            await this.executeNode(edge.target);
        }
    }

    private async executeTypeNode(node: Node) {
        const text = (node.data.text as string) || "";
        await invoke("simulate_type_text", { text });
    }

    private async executeKeyNode(node: Node) {
        const key = (node.data.key as string) || "";
        const mode = (node.data.mode as string) || "click";
        await invoke("simulate_key_action", { key, mode });
    }

    private async executeMousePressNode(node: Node) {
        const button = (node.data.button as string) || "left";
        const mode = (node.data.mode as string) || "click";
        await invoke("simulate_mouse_click", { button, mode });
    }

    private async executeMouseMoveNode(node: Node) {
        const x = Number(node.data.x) || 0;
        const y = Number(node.data.y) || 0;
        const mode = (node.data.mode as string) || "absolute";
        await invoke("simulate_mouse_move", { x, y, mode });
    }

    private async executeScrollMouseNode(node: Node) {
        const axis = (node.data.axis as string) || "vertical";
        const amount = Number(node.data.amount) || 0;
        await invoke("simulate_scroll", { axis, amount });
    }

    private async executeDelayNode(node: Node) {
        const delayMs = Number(node.data.delay) || 1000;
        await this.delay(delayMs);
    }

    private async executeConditionalNode(node: Node) {
        // this needs logic evaluation. For MVP let's assume simple JS eval
        // or just a placeholder 'true' branch.
        // realistically we need a context/variable system.
        // for now, let's just assume it always goes True for demo purposes
        // unless specific data says otherwise.

        const condition = node.data.condition as string;
        // TODO: Implement actual condition evaluation
        const result = true; // Placeholder

        const sourceHandle = result ? "true" : "false";

        const outgoingEdges = this.edges.filter(
            (e) => e.source === node.id && e.sourceHandle === sourceHandle,
        );

        for (const edge of outgoingEdges) {
            await this.executeNode(edge.target);
        }
    }

    private delay(ms: number) {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }

    public cleanup() {
        unregisterAll();
    }
}
