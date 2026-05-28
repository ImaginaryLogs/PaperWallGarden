// src/frames/PlaygroundFrame.tsx
import { jsxs } from "preact/jsx-runtime";
var PlaygroundFrame = {
  name: "playground-frame",
  css: `
.page[data-frame="playground-frame"] > #quartz-body {
  grid-template-columns: 1fr;
  grid-template-areas: "center";
  max-width: 960px;
  margin: 0 auto;
}
`,
  render({ componentData, pageBody: Content, footer: Footer }) {
    const slot = (C) => C(componentData);
    return /* @__PURE__ */ jsxs("div", { class: "center", children: [
      slot(Content),
      slot(Footer)
    ] });
  }
};
export {
  PlaygroundFrame
};
