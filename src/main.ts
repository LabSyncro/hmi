import { createApp } from "vue";
import { setupCalendar } from "v-calendar";
import "v-calendar/style.css";
import './style.css';
import App from "./App.vue";
import router from "./router";

// Create the app instance
const app = createApp(App);

// Use plugins
app.use(setupCalendar, {
  locales: {
    'vi': {
      firstDayOfWeek: 2,
      masks: {
        L: 'DD/MM/YYYY'
      }
    }
  }
});

app.use(router);

// Mount the app
app.mount("#app");

// Log a message about the database state issue
console.warn(
  "DEVELOPER NOTE: The Tauri backend needs to be updated to properly manage state.\n" +
  "In the Rust backend, ensure you call .manage() on the state before using database commands.\n" +
  "This is typically done in the main.rs file where the Tauri app is initialized."
);
