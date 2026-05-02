/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "media",
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Inter",
          "Pretendard",
          "Apple SD Gothic Neo",
          "Noto Sans KR",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Malgun Gothic",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};
