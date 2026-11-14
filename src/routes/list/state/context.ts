import { createContext } from "svelte";
import type { Board } from "./model.svelte";

export const [getBoard, provideBoard] = createContext<Board>();
