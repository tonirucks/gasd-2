export type Section = {
  id: number;
  name: string;
  isDndShadowItem?: boolean;
};

export type Task = {
  id: number;
  title: string;
  sectionId: number;
  done: boolean;
  isDndShadowItem?: boolean;
};
