import { createApp } from "vue";
import { setupCalendar } from "v-calendar";
import "v-calendar/style.css";
import './style.css';
import App from "./App.vue";
import router from "./router";

const app = createApp(App);

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

app.mount("#app");
