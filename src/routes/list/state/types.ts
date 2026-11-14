import type { Section as StoredSection, Task as StoredTask } from "$lib/data/types";

export type Section = StoredSection & { isDndShadowItem?: boolean };
export type Task = StoredTask & { isDndShadowItem?: boolean };
