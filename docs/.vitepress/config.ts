import { defineConfig } from "vitepress";
import { rulesSidebarItems } from "./generated/rules-sidebar";

export default defineConfig({
  base: "/tq/",
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
    search: {
      provider: "local"
    },
    nav: [
      { text: "Docs", link: "/guide/getting-started" },
      { text: "Developer", link: "/developer/" }
    ],
    sidebar: {
      "/guide/": [
        {
          text: "Guide",
          items: [
            { text: "Installation", link: "/guide/installation" },
            { text: "Getting Started", link: "/guide/getting-started" }
          ]
        },
        {
          text: "Reference",
          items: [
            {
              text: "Rules",
              link: "/reference/rules/",
              collapsed: true,
              items: [...rulesSidebarItems]
            },
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" }
          ]
        }
      ],
      "/reference/": [
        {
          text: "Guide",
          items: [
            { text: "Installation", link: "/guide/installation" },
            { text: "Getting Started", link: "/guide/getting-started" }
          ]
        },
        {
          text: "Reference",
          items: [
            {
              text: "Rules",
              link: "/reference/rules/",
              collapsed: true,
              items: [...rulesSidebarItems]
            },
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" }
          ]
        }
      ],
      "/developer/": [
        {
          text: "Developer",
          items: [
            { text: "Overview", link: "/developer/" },
            { text: "Context", link: "/developer/context" }
          ]
        },
        {
          text: "Standards",
          items: [
            { text: "Code Standards", link: "/developer/standards/code" },
            { text: "Docs Standards", link: "/developer/standards/docs" },
            { text: "Git Standards", link: "/developer/standards/git" },
            {
              text: "Policy Standards",
              link: "/developer/standards/policies"
            },
            { text: "Test Standards", link: "/developer/standards/tests" }
          ]
        },
        {
          text: "User Contract",
          items: [
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" },
            { text: "Rules", link: "/reference/rules/" }
          ]
        },
        {
          text: "Project Policy",
          items: [
            { text: "Versioning", link: "/developer/versioning" },
            { text: "Governance", link: "/developer/governance" }
          ]
        },
        {
          text: "Operations",
          items: [
            {
              text: "Attestation Verification",
              link: "/developer/attestation"
            },
            { text: "Releasing", link: "/developer/releasing" }
          ]
        },
        {
          text: "Architecture",
          items: [
            { text: "Architecture", link: "/developer/architecture" }
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
