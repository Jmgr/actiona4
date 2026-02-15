import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: "Actiona 4",
  tagline: "Desktop automation since 2005",
  favicon: "img/favicon.ico",

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: "https://your-docusaurus-site.example.com",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: "/",

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "Jmgr", // Usually your GitHub org/user name.
  projectName: "actiona4", // Usually your repo name.

  onBrokenLinks: "throw",

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl: "https://github.com/Jmgr/actiona4",
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],
  plugins: [
    [
      "@easyops-cn/docusaurus-search-local",
      {
        indexDocs: true,
        indexBlog: false,
        indexPages: true,
        hashed: true,
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        plugin: [
          "typedoc-plugin-mdn-links",
          "./plugins/typedoc-plugin-intrinsic-links.mjs",
        ],
        entryPoints: ["../run/assets/index.d.ts"],
        tsconfig: "../run/assets/tsconfig.json",
        out: "docs/api",
        entryFileName: "index",
        readme: "none",
        skipErrorChecking: true,
        suppressCommentWarningsInDeclarationFiles: true,
        defaultCategory: "Misc",
        categorizeByGroup: false,
        disableSources: true,
        navigation: {
          includeCategories: true,
          includeGroups: false,
          includeFolders: false,
          excludeReferences: true,
        },
        sidebar: {
          autoConfiguration: true,
          typescript: false,
        },
      },
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: "img/docusaurus-social-card.jpg",
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: "Actiona 4",
      logo: {
        alt: "Actiona logo",
        src: "img/icon.png",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docsSidebar",
          position: "left",
          label: "Documentation",
        },
        {
          href: "https://github.com/Jmgr/actiona4",
          label: "GitHub",
          position: "right",
        },
        { type: "search", position: "right" },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Documentation",
              to: "/docs/intro",
            },
          ],
        },
        {
          title: "Community",
          items: [
            {
              label: "Discord",
              href: "https://discord.gg/ubTjJu3dVZ",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Jonathan Mercier-Ganady. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
