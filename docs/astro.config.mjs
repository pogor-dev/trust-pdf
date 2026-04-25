// @ts-check
import solidJs from "@astrojs/solid-js"
import starlight from "@astrojs/starlight"
import tailwindcss from "@tailwindcss/vite"
import { defineConfig } from "astro/config"
import rehypeAbbr from "./src/plugins/rehype-abbr"

// https://astro.build/config
export default defineConfig({
  integrations: [
    solidJs({ devtools: true }),
    starlight({
      title: "TRust PDF Documentation",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/withastro/starlight" }],
      sidebar: [
        {
          label: "Syntax",
          items: [{ label: "Green", autogenerate: { directory: "syntax/green" } }],
        },
      ],
      customCss: ["./src/styles/global.css"],
    }),
  ],
  vite: {
    plugins: [tailwindcss()],
  },
  markdown: {
    rehypePlugins: [
      [
        rehypeAbbr,
        {
          abbreviations: [
            { abbr: "EOL", title: "End of line", matchCase: true },
            { abbr: "CST", title: "Concrete Syntax Tree", matchCase: true },
            { abbr: "HIR", title: "High-level Intermediate Representation", matchCase: true },
            {
              abbr: "Trivia",
              title:
                "Non-semantic syntax text preserved for full fidelity (e.g. whitespace, comments, line endings)",
              matchCase: false,
            },
          ],
          targetTags: ["p", "li"],
          blockedTags: ["code", "pre", "script", "style", "abbr"],
        },
      ],
    ],
  },
  experimental: {
    contentIntellisense: true,
  },
})
