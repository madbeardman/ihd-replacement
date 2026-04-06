import { setupDebugToggle, loadDashboard, updateClock, advanceUsageRotation } from "./dashboard.js";
import { setupSettingsModal } from "./settings.js";
import { loadHistoryModalPartial, setupHistoryModal } from "./history.js";

async function init() {
    setupDebugToggle();
    setupSettingsModal();
    updateClock();

    await loadHistoryModalPartial();
    setupHistoryModal();

    await loadDashboard();

    setInterval(updateClock, 1000);
    setInterval(loadDashboard, 10000);
    setInterval(advanceUsageRotation, 8000);
}

init();