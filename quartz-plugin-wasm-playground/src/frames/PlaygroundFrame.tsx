import { h } from "preact"
import type { PageFrame, PageFrameProps } from "@quartz-community/types"
import type { ComponentChildren } from "preact"

export const PlaygroundFrame: PageFrame = {
  name: "playground-frame",
  css: `
.page[data-frame="playground-frame"] > #quartz-body {
  grid-template-columns: 1fr;
  grid-template-areas: "center";
  max-width: 960px;
  margin: 0 auto;
}
`,
  render({ componentData, pageBody: Content, footer: Footer }: PageFrameProps) {
    const slot = (C: (p: typeof componentData) => unknown): ComponentChildren =>
      C(componentData) as ComponentChildren
    return (
      <div class="center">
        {slot(Content as any)}
        {slot(Footer as any)}
      </div>
    )
  },
}