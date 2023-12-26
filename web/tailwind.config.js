/** @type {import('tailwindcss').Config} */

// Tailwind wasn't working with `cargo leptos watch`. I wasn't seeing any output to 
// indicate that tailwind was being hot reloaded.
// First, I had to add a `style-file` which was an empty file. Now, I could see the tailwind
// build output in `cargo leptos watch`. It still wasn't right though, because it was building from
// the root of the project. I had to update content to check inside `web/`.
module.exports = {
  content: ["*.html", "./web/src/**/*.rs",],
  theme: {
    extend: {},
  },
  plugins: [],
}
