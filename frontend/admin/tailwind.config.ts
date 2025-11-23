import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        success: {
          DEFAULT: "hsl(142 76% 36%)",
          foreground: "hsl(144 61% 97%)",
        },
        warning: {
          DEFAULT: "hsl(38 92% 50%)",
          foreground: "hsl(48 96% 89%)",
        },
        info: {
          DEFAULT: "hsl(199 89% 48%)",
          foreground: "hsl(204 94% 94%)",
        },
      },
      backgroundImage: {
        "gradient-blue-cyan": "linear-gradient(to bottom right, rgb(59 130 246), rgb(34 211 238))",
        "gradient-purple-pink": "linear-gradient(to bottom right, rgb(168 85 247), rgb(244 114 182))",
        "gradient-emerald-teal": "linear-gradient(to bottom right, rgb(16 185 129), rgb(45 212 191))",
        "gradient-orange-red": "linear-gradient(to bottom right, rgb(249 115 22), rgb(248 113 113))",
        "gradient-amber-yellow": "linear-gradient(to bottom right, rgb(245 158 11), rgb(250 204 21))",
        "gradient-violet-indigo": "linear-gradient(to bottom right, rgb(139 92 246), rgb(99 102 241))",
        "gradient-indigo-purple": "linear-gradient(to bottom right, rgb(79 70 229), rgb(168 85 247))",
        "gradient-slate-blue-indigo": "linear-gradient(to bottom right, rgb(248 250 252), rgb(239 246 255), rgb(238 242 255))",
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        "slide-in-from-right": {
          from: { transform: "translateX(100%)" },
          to: { transform: "translateX(0)" },
        },
        "slide-out-to-right": {
          from: { transform: "translateX(0)" },
          to: { transform: "translateX(100%)" },
        },
        "fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        "fade-out": {
          from: { opacity: "1" },
          to: { opacity: "0" },
        },
        shimmer: {
          "0%": {
            transform: "translateX(-100%)",
          },
          "100%": {
            transform: "translateX(100%)",
          },
        },
        "hover-lift": {
          from: { transform: "translateY(0)" },
          to: { transform: "translateY(-4px)" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "slide-in-from-right": "slide-in-from-right 0.3s ease-out",
        "slide-out-to-right": "slide-out-to-right 0.3s ease-in",
        "fade-in": "fade-in 0.2s ease-out",
        "fade-out": "fade-out 0.2s ease-in",
        shimmer: "shimmer 2s infinite",
        "hover-lift": "hover-lift 0.3s ease-out",
      },
      fontFamily: {
        sans: ["var(--font-inter)", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", "monospace"],
      },
      spacing: {
        18: "4.5rem",
        88: "22rem",
        112: "28rem",
        128: "32rem",
      },
      maxWidth: {
        "8xl": "88rem",
        "9xl": "96rem",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};

export default config;
