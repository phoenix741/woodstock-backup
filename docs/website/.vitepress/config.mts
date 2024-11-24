import { defineConfig } from "vitepress";
import imsize from "markdown-it-imsize";
import plantuml from "markdown-it-plantuml";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Woodstock Backup",
  description: "Centralized Backup Software (based on BTRFS)",
  markdown: {
    linkify: true,
    config: (md) => {
      md.use(imsize);
      md.use(plantuml);
    },
  },
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Home", link: "/" },
      { text: "About", link: "/about/" },
      { text: "Documentation", link: "/doc/" },
      {
        text: "Download",
        link: "https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/releases",
      },
    ],

    sidebar: {
      "/doc/": [
        {
          text: "Documentation",
          items: [
            { text: "Introduction", link: "/doc/" },
            { text: "Installation", link: "/doc/installation" },
            { text: "Configuration", link: "/doc/configuration" },
            { text: "Scheduler", link: "/doc/scheduler" },
            { text: "FAQ", link: "/doc/faq" },
            { text: "Roadmap", link: "/doc/roadmap" },
          ],
        },
        {
          text: "Internal Documentation",
          items: [
            { text: "Pool", link: "/doc/internal/pool" },
            {
              text: "Client Authentication",
              link: "/doc/internal/client_auth",
            },
          ],
        },
      ],
    },

    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/phoenix741/woodstock-backup",
      },
      {
        icon: "gitea",
        link: "https://gogs.shadoware.org/ShadowareOrg/woodstock-backup",
      },
    ],
  },
});
