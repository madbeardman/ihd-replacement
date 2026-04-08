import { setupDebugToggle, loadDashboard, updateClock, advanceUsageRotation } from "./dashboard.js";
import { setupSettingsModal, loadSettingsModalPartial } from "./settings.js";
import { loadHistoryModalPartial } from "./history.js";

async function init() {
    setupDebugToggle();
    updateClock();

    await loadHistoryModalPartial();
    await loadSettingsModalPartial();

    setupSettingsModal(async () => {
        await loadDashboard();
    });

    await loadDashboard();

    setInterval(updateClock, 1000);
    setInterval(loadDashboard, 10000);
    setInterval(advanceUsageRotation, 8000);
}

init();