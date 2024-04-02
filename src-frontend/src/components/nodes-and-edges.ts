import type { Node, Edge } from "@xyflow/svelte";
import { Direction, CoordinateMode, Axis } from "@utils";

export const initialNodes: Node[] = [
  {
    id: "1",
    position: { x: 0, y: 0 },
    data: { title: "Key Bind", subline: "Bint the output to a key" },
    type: "base",
  },
  {
    id: "2",
    position: { x: 500, y: 0 },
    data: { title: "Type Text", subline: "Writes a given text" },
    type: "base",
  },
  {
    id: "3",
    position: { x: 0, y: 250 },
    data: { title: "Press Key", subline: "Presses a given key1" },
    type: "base",
  },
  {
    id: "4",
    position: { x: 500, y: 250 },
    data: { title: "Mouse Press", subline: "Presses a mouse button" },
    type: "base",
  },
  {
    id: "5",
    position: { x: 1000, y: 125 },
    data: { title: "Delay", subline: "Waits for a given amount of time" },
    type: "base",
  },
  {
    id: "6",
    position: { x: 1400, y: 125 },
    data: { title: "Scroll Mouse" },
    type: "base",
  },
  {
    id: "7",
    position: { x: 1500, y: 250 },
    data: {
      title: "Type Text",
      text: "Hello World!",
      subline: "Types a given text",
      // cancelKey: "K",
      delay: 0.6,
    },
    type: "typeNode",
  },
  {
    id: "8",
    position: { x: 1500, y: 500 },
    data: {
      title: "Key Bind",
      subline: "Define a Key Bind",
    },
    type: "keyBindNode",
  },
  {
    id: "9",
    position: { x: 1750, y: 500 },
    data: {
      title: "Key Click",
      subline: "Click a keyboard or mouse key",
      key: "K",
      numTimes: 3,
      delay: 0.5,
      cancelKey: "esc",
      mode: Direction.CLICK,
    },
    type: "keyClickNode",
  },
  {
    id: "10",
    position: { x: 2150, y: 250 },
    data: {
      title: "Delay",
      subline: "Wait for x seconds",
      delay: 1.7,
    },
    type: "delayNode",
  },
  {
    id: "11",
    position: { x: 2250, y: 250 },
    data: {
      title: "Move Mouse",
      subline: "Move the mouse to a given position",
      mode: CoordinateMode.ABSOLUTE,
      x: 100,
      y: 100,
    },
    type: "moveMouseNode",
  },
  {
    id: "12",
    position: { x: 2250, y: 500 },
    data: {
      axis: Axis.VERTICAL,
      title: "Scroll Mouse",
      subline: "Scroll the mouse wheel",
      length: 3,
    },
    type: "scrollMouseNode",
  },
];

export const initialEdges: Edge[] = [
  {
    id: "e1-2",
    source: "1",
    target: "2",
  },
  {
    id: "e3-4",
    source: "3",
    target: "4",
  },
  {
    id: "e2-5",
    source: "2",
    target: "5",
  },
  {
    id: "e4-5",
    source: "4",
    target: "5",
  },
  {
    id: "e5-6",
    source: "5",
    target: "6",
  },
];
