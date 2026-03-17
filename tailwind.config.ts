import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{vue,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        "bg-primary": "#0a0a0f",
        "bg-surface": "rgba(255,255,255,0.04)",
        "accent-neon": "#00f5d4",
        "accent-ember": "#ff7b00",
        "accent-error": "#ff2d55",
        "accent-success": "#39ff14",
      },
      fontFamily: {
        iceland: ["Iceland", "monospace"],
      },
      backdropBlur: {
        glass: "16px",
      },
    },
  },
  plugins: [],
} satisfies Config;
