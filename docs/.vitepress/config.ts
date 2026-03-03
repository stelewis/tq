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
  lastUpdated: false,
  head: [
    ["link", { rel: "icon", href: "/tq/favicon.ico" }],
    [
      "link",
      { rel: "icon", type: "image/png", href: "/tq/tq-logo-mini.png" }
    ],
    [
      "link",
      { rel: "icon", type: "image/svg+xml", href: "/tq/tq-logo-mini.svg" }
    ],
    ["link", { rel: "apple-touch-icon", href: "/tq/tq-logo-mini.png" }],
    ["meta", { name: "theme-color", content: "#6366f1" }]
  ],
  themeConfig: {
    logo: { src: "/tq-logo-mini.svg", width: 24, height: 24 },
    search: {
      provider: "local"
    },
    nav: [
      { text: "Docs", link: "/guide/what-is-tq" },
      { text: "Developer", link: "/developer/" }
    ],
    sidebar: {
      "/guide/": [
        {
          text: "Guide",
          link: "/guide/",
          collapsed: false,
          items: [
            { text: "What is tq?", link: "/guide/what-is-tq" },
            { text: "QuickStart", link: "/guide/quickstart" }
          ]
        },
        {
          text: "Reference",
          link: "/reference/",
          collapsed: false,
          items: [
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" }
          ]
        },
        {
          text: "Rules",
          link: "/reference/rules/",
          collapsed: false,
          items: [...rulesSidebarItems]
        }
      ],
      "/reference/": [
        {
          text: "Guide",
          link: "/guide/",
          collapsed: false,
          items: [
            { text: "What is tq?", link: "/guide/what-is-tq" },
            { text: "QuickStart", link: "/guide/quickstart" }
          ]
        },
        {
          text: "Reference",
          link: "/reference/",
          collapsed: false,
          items: [
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" }
          ]
        },
        {
          text: "Rules",
          link: "/reference/rules/",
          collapsed: false,
          items: [...rulesSidebarItems]
        }
      ],
      "/developer/": [
        {
          text: "Developer",
          collapsed: false,
          items: [
            { text: "Overview", link: "/developer/" },
            { text: "Context", link: "/developer/context" }
          ]
        },
        {
          text: "Standards",
          collapsed: false,
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
          collapsed: false,
          items: [
            { text: "CLI", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "Exit Codes", link: "/reference/exit-codes" },
            { text: "Rules", link: "/reference/rules/" }
          ]
        },
        {
          text: "Project Policy",
          collapsed: false,
          items: [
            { text: "Versioning", link: "/developer/versioning" },
            { text: "Governance", link: "/developer/governance" }
          ]
        },
        {
          text: "Operations",
          collapsed: false,
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
          collapsed: false,
          items: [
            { text: "Architecture", link: "/developer/architecture" }
          ]
        }
      ]
    },
    socialLinks: [{ icon: "github", link: "https://github.com/stelewis/tq" }]
  }
});
