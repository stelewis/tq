import { defineConfig } from "vitepress";
import { rulesSidebarItems } from "./generated/rules-sidebar";

const guideSidebar = [
  {
    text: "Guide",
    link: "/guide/",
    collapsed: false,
    items: [
      { text: "What is tq?", link: "/guide/what-is-tq" },
      { text: "Quickstart", link: "/guide/quickstart" }
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
];

const developerSidebar = [
  {
    text: "Developer",
    collapsed: false,
    items: [
      { text: "Overview", link: "/developer/" },
      { text: "Context", link: "/developer/context" },
      { text: "Architecture", link: "/developer/architecture" },
      { text: "Security", link: "/developer/security" }
    ]
  },
  {
    text: "Standards",
    collapsed: false,
    items: [
      { text: "Code", link: "/developer/standards/code" },
      { text: "Docs", link: "/developer/standards/docs" },
      { text: "Git", link: "/developer/standards/git" },
      { text: "Policies", link: "/developer/standards/policies" },
      {
        text: "Supply-Chain Security",
        link: "/developer/standards/supply-chain-security"
      },
      { text: "Tests", link: "/developer/standards/tests" }
    ]
  },
  {
    text: "Tooling",
    collapsed: false,
    items: [
      { text: "Overview", link: "/developer/tools/" },
      { text: "Local Workflows", link: "/developer/tools/local-workflows" },
      { text: "CI and Automation", link: "/developer/tools/ci" },
      {
        text: "Pin Maintenance",
        link: "/developer/tools/pin-maintenance"
      },
      { text: "Docs and Release", link: "/developer/tools/docs-and-release" }
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
    text: "Governance",
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
      { text: "Releasing", link: "/developer/releasing" },
      {
        text: "Attestation Verification",
        link: "/developer/attestation"
      }
    ]
  }
];

export default defineConfig({
  base: "/tq/",
  lang: "en-US",
  title: "tq",
  description:
    "tq inspects a codebase's tests and enforces quality rules so tests remain discoverable, focused, actionable, and maintainable.",
  srcDir: ".",
  srcExclude: ["adr/**", "plans/**", "design/**", "**/0000-template.md"],
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
      { text: "Guide", link: "/guide/" },
      { text: "Reference", link: "/reference/" },
      { text: "Developer", link: "/developer/" }
    ],
    sidebar: {
      "/guide/": guideSidebar,
      "/reference/": guideSidebar,
      "/developer/": developerSidebar
    },
    socialLinks: [{ icon: "github", link: "https://github.com/stelewis/tq" }]
  }
});
