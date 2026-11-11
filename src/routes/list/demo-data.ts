import type { Section, Task } from "./types";

export function createDemoData(): { sections: Section[]; tasks: Task[] } {
  return {
    sections: [
      { id: 1, name: "Personal" },
      { id: 2, name: "Work" },
      { id: 3, name: "Reading list" },
    ],
    tasks: [
      {
        id: 1,
        title: "Book a table for Friday night",
        sectionId: 1,
        done: false,
      },
      {
        id: 2,
        title: "Get outside for a long walk",
        sectionId: 1,
        done: false,
      },
      {
        id: 3,
        title: "Explore ideas for the new website",
        sectionId: 2,
        done: false,
      },
      {
        id: 4,
        title: "Send the first design concepts",
        sectionId: 2,
        done: false,
      },
      { id: 5, title: "Finish reading a chapter", sectionId: 3, done: false },
      {
        id: 6,
        title: "Make a little space for the week",
        sectionId: 1,
        done: true,
      },
    ],
  };
}
