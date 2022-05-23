import "./main.scss";
import "@mdi/font/css/materialdesignicons.css";
import { createVuetify, type ThemeDefinition } from "vuetify";
import { aliases, mdi } from "vuetify/lib/iconsets/mdi";

const woodstockLightTheme: ThemeDefinition = {
  dark: false,
  colors: {
    background: "#f1f5f9",
    surface: "#FFFFFF",
    primary: "#059669",
    "primary-darken-1": "#047857",
    secondary: "#1e293b",
    "secondary-darken-1": "#0f172a",
    error: "#ef4444",
    info: "#2196F3",
    success: "#10b981",
    warning: "#f97316",
  },
};

export default createVuetify({
  theme: {
    defaultTheme: "woodstockLightTheme",
    themes: {
      woodstockLightTheme,
    },
  },
  icons: {
    defaultSet: "mdi",
    aliases,
    sets: {
      mdi,
    },
  },
});
