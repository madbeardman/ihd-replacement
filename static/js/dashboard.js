import { state } from "./state.js";
import { renderAgileChart } from "./agile.js";
import {
    clamp,
    formatClock,
    formatGbp,
    formatPrice,
    isDevMode,
} from "./utils.js";

function getHouseUsageColour(watts) {
    if (watts < 100) return "var(--usage-green-bright)";
    if (watts < 200) return "var(--usage-green-soft)";
    if (watts < 1000) return "var(--normal)";
    if (watts < 2000) return "var(--usage-orange)";
    return "var(--usage-red)";
}

function getSolarColor(_watts) {
    return "#16a34a";
}

function showPollIndicator() {
    const el = document.getElementById("poll-indicator");
    if (!el) return;
    el.style.display = "inline-block";
}

function pulsePollIndicator() {
    const el = document.getElementById("poll-indicator");
    if (!el) return;

    el.classList.remove("pulse");
    void el.offsetWidth;
    el.classList.add("pulse");

    setTimeout(() => {
        el.classList.remove("pulse");
    }, 1000);
}

function setPollIndicatorOk() {
    const el = document.getElementById("poll-indicator");
    if (!el) return;

    el.classList.remove("error");
}

function setPollIndicatorError() {
    const el = document.getElementById("poll-indicator");
    if (!el) return;

    el.classList.remove("pulse");
    el.classList.add("error");
}

function renderUsageRotation() {
    if (!state.latestUsageMetrics) return;

    const valueEl = document.getElementById("usage-main-value");
    const subtextEl = document.getElementById("usage-subtext");

    if (!valueEl || !subtextEl) return;

    const states = [
        {
            value:
                typeof state.latestUsageMetrics.current_power_w === "number"
                    ? `${Math.round(state.latestUsageMetrics.current_power_w)}W`
                    : "--",
            subtext: "Electricity + Gas",
        },
        {
            value:
                typeof state.latestUsageMetrics.current_cost_per_hour_gbp === "number"
                    ? `${formatGbp(state.latestUsageMetrics.current_cost_per_hour_gbp)}/hr`
                    : "--",
            subtext:
                typeof state.latestUsageMetrics.current_price_p_per_kwh === "number"
                    ? `At ${formatPrice(state.latestUsageMetrics.current_price_p_per_kwh)}`
                    : "Current cost",
        },
        {
            value:
                typeof state.latestUsageMetrics.cost_today_gbp === "number"
                    ? formatGbp(state.latestUsageMetrics.cost_today_gbp)
                    : "--",
            subtext: "Cost Today",
        },
    ];

    const current = states[state.usageRotationIndex % states.length];
    valueEl.textContent = current.value;
    subtextEl.textContent = current.subtext;
}

export function advanceUsageRotation() {
    if (!state.latestUsageMetrics) return;
    state.usageRotationIndex = (state.usageRotationIndex + 1) % 3;
    renderUsageRotation();
}

export function updateClock() {
    const clock = document.getElementById("header-time");
    if (!clock) return;
    clock.textContent = formatClock();
}

function updateHouseUsageGauge(watts) {
    const gaugeArc = document.getElementById("usage-gauge-electric");

    if (!gaugeArc || typeof watts !== "number") return;

    const maxWatts = 4000;
    const clampedWatts = clamp(watts, 0, maxWatts);

    const linearRatio = clampedWatts / maxWatts;
    let percentage = Math.sqrt(linearRatio) * 100;

    if (clampedWatts > 0) {
        percentage = Math.max(percentage, 10);
    } else {
        percentage = 0;
    }

    const colour = getHouseUsageColour(clampedWatts);

    gaugeArc.setAttribute("stroke-dasharray", `${percentage} 100`);
    gaugeArc.style.stroke = colour;
}

function updateSolarGauge(watts) {
    const gaugeArc = document.getElementById("solar-gauge-fill");
    const gaugeTrack = document.getElementById("solar-gauge-track");

    if (!gaugeArc || !gaugeTrack || typeof watts !== "number") return;

    const maxWatts = 3480;
    const clampedWatts = clamp(watts, 0, maxWatts);

    const linearRatio = clampedWatts / maxWatts;
    let percentage = Math.sqrt(linearRatio) * 100;

    if (clampedWatts > 0) {
        percentage = Math.max(percentage, 8);
    } else {
        percentage = 0;
    }

    const colour = getSolarColor(clampedWatts);

    gaugeArc.setAttribute("stroke-dasharray", `${percentage} 100`);
    gaugeArc.style.stroke = colour;
    gaugeTrack.style.stroke = "";
}

function updateApplianceRow(appliances) {
    const washerEl = document.getElementById("appliance-washing-machine");
    const dishwasherEl = document.getElementById("appliance-dishwasher");
    const dryerEl = document.getElementById("appliance-tumble-dryer");

    if (isDevMode()) {
        console.log("Updating appliances:", appliances);
    }

    if (!washerEl || !dishwasherEl || !dryerEl || !appliances) return;

    const washer = appliances.washing_machine?.display ?? "--";
    const dishwasher = appliances.dishwasher?.display ?? "--";
    const dryer = appliances.tumble_dryer?.display ?? "--";

    if (isDevMode()) {
        console.log("Appliance display values:", { washer, dishwasher, dryer });
    }

    washerEl.textContent = washer;
    dishwasherEl.textContent = dishwasher;
    dryerEl.textContent = dryer;

    washerEl.classList.toggle("running", appliances.washing_machine?.running === true);
    dishwasherEl.classList.toggle("running", appliances.dishwasher?.running === true);
    dryerEl.classList.toggle("running", appliances.tumble_dryer?.running === true);
}

export async function loadDashboard() {
    if (state.dashboardRequestInFlight) return;
    state.dashboardRequestInFlight = true;

    const output = document.getElementById("output");

    try {
        const response = await fetch("/api/dashboard", {
            headers: { Accept: "application/json" },
            cache: "no-store",
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();

        showPollIndicator();
        setPollIndicatorOk();
        pulsePollIndicator();

        const indicator = document.getElementById("poll-indicator");
        if (indicator) {
            indicator.style.display = "inline-block";
        }

        if (output) {
            output.textContent = JSON.stringify(data, null, 2);
        }

        state.latestUsageMetrics = data.usage_metrics ?? null;
        renderUsageRotation();

        if (typeof data.live?.house_power_w === "number") {
            updateHouseUsageGauge(Math.round(data.live.house_power_w));
        } else {
            updateHouseUsageGauge(0);
        }

        if (typeof data.live?.solar_generation_w === "number") {
            let solar = Math.round(data.live.solar_generation_w);

            if (solar < 10) {
                solar = 0;
            }

            const valueEl = document.querySelector("#solar-panel .panel-value");
            if (valueEl) valueEl.textContent = `${solar}W`;

            updateSolarGauge(solar);
        } else {
            const valueEl = document.querySelector("#solar-panel .panel-value");
            if (valueEl) valueEl.textContent = "--";

            updateSolarGauge(0);
        }

        renderAgileChart(data.agile);
        updateApplianceRow(data.appliances);
    } catch (error) {
        const dashboard = document.getElementById("dashboard");
        const devMode = dashboard?.dataset.devMode === "true";

        const updatedEl = document.getElementById("last-updated");
        if (updatedEl) {
            updatedEl.textContent = devMode ? "Update failed" : "";
        }

        showPollIndicator();
        setPollIndicatorError();

        if (output) {
            output.textContent = String(error);
        }
    } finally {
        state.dashboardRequestInFlight = false;
    }
}

export function setupDebugToggle() {
    const debugToggle = document.getElementById("debug-toggle");
    if (!debugToggle) return;

    debugToggle.addEventListener("click", () => {
        const debug = document.getElementById("debug");
        const button = document.getElementById("debug-toggle");
        const isHidden = debug?.hasAttribute("hidden");

        if (!debug || !button) return;

        if (isHidden) {
            debug.removeAttribute("hidden");
            button.textContent = "Hide raw JSON";
        } else {
            debug.setAttribute("hidden", "");
            button.textContent = "Show raw JSON";
        }
    });
}