// Imported here rather than from App.tsx so the stylesheet stays part of the
// entry chunk and keeps its <link> in index.html. Pulling it in through the
// dynamic import below would defer it and flash unstyled content on every launch.
import "./App.css";
import { installCrashHandlers, reportFatal } from "./lib/crashScreen";
import { installCompatShims } from "./lib/compat";

installCrashHandlers();
installCompatShims();

// The rest of the app is loaded dynamically on purpose. `import` statements are
// hoisted, so anything imported statically here would evaluate *before* the two
// calls above — and a module-evaluation throw in i18n, the model store or React
// itself is exactly the failure we need to be installed in time to catch.
import("./bootstrap").catch((error) => reportFatal(error, "startup"));
