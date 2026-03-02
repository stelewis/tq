import { defineConfig } from "vitepress";

export default defineConfig({
  lang: "en-US",
  title: "tq",
  description:
    "tq inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.",
  srcDir: ".",
  srcExclude: ["plans/**", "design/**", "**/0000-template.md"],
  outDir: ".vitepress/dist",
  cacheDir: ".vitepress/cache",
  ignoreDeadLinks: false,
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "Reference", link: "/reference/cli" },
      { text: "Developer", link: "/developer/context" },
      { text: "ADR", link: "/adr/README" }
    ],
    sidebar: {
      "/guide/": [
        {
          text: "Guide",
          items: [
            { text: "Installation", link: "/guide/installation" },
            { text: "Getting Started", link: "/guide/getting-started" }
          ]
        }
      ],
      "/reference/": [
        {
          text: "Reference",
          items: [
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" },
            { text: "Rules", link: "/reference/rules/" },
            { text: "Versioning", link: "/reference/versioning" }
          ]
        }
      ],
      "/developer/": [
        {
          text: "Developer",
          items: [
            { text: "Context", link: "/developer/context" },
            { text: "Standards", link: "/developer/README" },
            { text: "Rules", link: "/developer/tools/rules" },
            { text: "tq check", link: "/developer/tools/tq_check" },
            {
              text: "Attestation Verification",
              link: "/developer/tools/attestation_verification"
            },
            { text: "Releasing", link: "/developer/tools/releasing" }
          ]
        }
      ],
      "/adr/": [
        {
          text: "Architecture Decisions",
          items: [
            { text: "ADR Index", link: "/adr/README" },
            {
              text: "0001 CLI and Config Contract",
              link: "/adr/0001-tq-cli-config-contract"
            }
          ]
        }
      ]
    },
    editLink: {
      pattern: "https://github.com/stelewis/tq/edit/main/docs/:path",
      text: "Edit this page on GitHub"
    },
    socialLinks: [{ icon: "github", link: "https://github.com/stelewis/tq" }]
  }
});
