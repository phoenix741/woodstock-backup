module.exports = {
  title: "Woodstock Backup",
  description: "Centralized Backup Software (based on BTRFS)",
  markdown: {
    linkify: true,
    extendMarkdown: (md) => {
      md.use(require("markdown-it-imsize"));
    },
  },
  themeConfig: {
    search: false,
    smoothScroll: true,
    repo: "phoenix741/woodstock-backup",
    nav: [
      { text: "Home", link: "/" },
      { text: "About", link: "/about/" },
      { text: "Documentation", link: "/doc/" },
    ],
    sidebar: [
      {
        title: "Documentation",
        path: "/doc/",
        sidebarDepth: 2,
        children: [
          "/doc/installation.md",
          "/doc/addnewhost.md",
          "/doc/updatetools.md",
          "/doc/updatescheduler.md",
          "/doc/faq.md",
          "/doc/roadmap.md",
        ],
      },
      "/about/",
    ],
  },
  plugins: [
    [
      "vuepress-plugin-matomo",
      {
        siteId: 37,
        trackerUrl: "https://stats.shadoware.org/",
      },
    ],
  ],
};
