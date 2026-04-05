let dashboardRequestInFlight = false;
let latestUsageMetrics = null;
let usageRotationIndex = 0;
let historyMetric = "cost";
let historyRange = "day";

function isDevMode() {
    const dashboard = document.getElementById("dashboard");
    return dashboard?.dataset.devMode === "true";
}

function formatPrice(value) {
    return `${value.toFixed(2)}p`;
}

function formatHourLabel(isoString) {
    const date = new Date(isoString);
    return date.getHours().toString().padStart(2, "0");
}

function formatClock(now = new Date()) {
    return now.toLocaleTimeString("en-GB", {
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
    });
}

function formatGbp(value) {
    if (value == null) return "£--";
    return `£${value.toFixed(2)}`;
}

function formatHistoryCost(value) {
    return `£${value.toFixed(3)}`;
}

function formatHistoryKwh(value) {
    return `${value.toFixed(3)} kWh`;
}

function getHistoryDisplayValue(rawValue) {
    if (historyMetric === "cost") {
        return formatHistoryCost(rawValue);
    }

    return formatHistoryKwh(rawValue);
}

function roundAxisMax(value) {
    return Math.ceil(value * 100) / 100; // round to 2dp
}

function getHistorySlotValue(slot) {
    if (!slot) return 0;

    if (historyMetric === "cost") {
        return typeof slot.cost_gbp === "number" ? slot.cost_gbp : 0;
    }

    return typeof slot.consumption_kwh === "number" ? slot.consumption_kwh : 0;
}

function renderUsageRotation() {
    if (!latestUsageMetrics) return;

    const valueEl = document.getElementById("usage-main-value");
    const subtextEl = document.getElementById("usage-subtext");

    if (!valueEl || !subtextEl) return;

    const states = [
        {
            value:
                typeof latestUsageMetrics.current_power_w === "number"
                    ? `${Math.round(latestUsageMetrics.current_power_w)}W`
                    : "--",
            subtext: "Electricity + Gas",
        },
        {
            value:
                typeof latestUsageMetrics.current_cost_per_hour_gbp === "number"
                    ? `${formatGbp(latestUsageMetrics.current_cost_per_hour_gbp)}/hr`
                    : "--",
            subtext:
                typeof latestUsageMetrics.current_price_p_per_kwh === "number"
                    ? `At ${formatPrice(latestUsageMetrics.current_price_p_per_kwh)}`
                    : "Current cost",
        },
        {
            value:
                typeof latestUsageMetrics.cost_today_gbp === "number"
                    ? formatGbp(latestUsageMetrics.cost_today_gbp)
                    : "--",
            subtext: "Cost Today",
        },
    ];

    const current = states[usageRotationIndex % states.length];
    valueEl.textContent = current.value;
    subtextEl.textContent = current.subtext;
}

function advanceUsageRotation() {
    if (!latestUsageMetrics) return;
    usageRotationIndex = (usageRotationIndex + 1) % 3;
    renderUsageRotation();
}

function updateClock() {
    const clock = document.getElementById("header-time");
    if (!clock) return;
    clock.textContent = formatClock();
}

function formatLastUpdated(now = new Date()) {
    return now.toLocaleTimeString("en-GB", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
    });
}

function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
}

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

function hidePollIndicator() {
    const el = document.getElementById("poll-indicator");
    if (!el) return;
    el.style.display = "none";
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

    // Fill arc only
    gaugeArc.setAttribute("stroke-dasharray", `${percentage} 100`);
    gaugeArc.style.stroke = colour;

    // Track stays grey via CSS
    gaugeTrack.style.stroke = "";
}

function renderAgileChart(data) {
    const chart = document.getElementById("agile-chart");
    const timeAxis = document.getElementById("agile-time-axis");

    if (!chart || !timeAxis) return;

    timeAxis.innerHTML = "";
    chart.innerHTML = "";

    if (!data?.slots || data.slots.length === 0) {
        chart.innerHTML = "<div style='color: var(--muted);'>No data</div>";
        return;
    }

    const values = data.slots.map((slot) => slot.value_inc_vat);
    const hasNegative = values.some((v) => v < 0);
    const minBarHeight = 12;

    chart.className = hasNegative ? "agile-chart mixed" : "agile-chart positive-only";

    if (!hasNegative) {
        const maxPrice = Math.max(...values, 0.001);

        for (const slot of data.slots) {
            const bar = document.createElement("div");
            const heightPercent = (slot.value_inc_vat / maxPrice) * 95;
            const finalHeight = Math.max(slot.value_inc_vat > 0 ? minBarHeight : 0, heightPercent);

            bar.className = `agile-bar ${slot.band}${slot.is_now ? " now" : ""}`;
            bar.style.height = `${finalHeight}%`;
            bar.dataset.price = formatPrice(slot.value_inc_vat);
            bar.title =
                `${slot.source_day} slot ${slot.source_index}\n` +
                `${formatPrice(slot.value_inc_vat)}\n` +
                `${slot.valid_from} → ${slot.valid_to}` +
                (slot.is_now ? "\nCURRENT SLOT" : "");

            chart.appendChild(bar);
        }

        const bars = Array.from(chart.children);
        const chartRect = chart.getBoundingClientRect();

        data.slots.forEach((slot, index) => {
            const date = new Date(slot.valid_from);

            if (date.getMinutes() === 0 && bars[index]) {
                const barRect = bars[index].getBoundingClientRect();
                const label = document.createElement("div");

                label.className = "agile-time-label";
                label.textContent = formatHourLabel(slot.valid_from);

                const barCenter = barRect.left - chartRect.left + barRect.width / 2;
                label.style.left = `${barCenter}px`;

                timeAxis.appendChild(label);
            }
        });

        return;
    }

    const maxPositive = Math.max(...values.filter((v) => v > 0), 0);
    const maxNegativeAbs = Math.max(...values.filter((v) => v < 0).map((v) => Math.abs(v)), 0);
    const scaleMax = Math.max(maxPositive, maxNegativeAbs, 0.001);

    for (const slot of data.slots) {
        const wrap = document.createElement("div");
        wrap.className = "agile-bar-wrap";

        const bar = document.createElement("div");
        const value = slot.value_inc_vat;
        const heightPercent = (Math.abs(value) / scaleMax) * 50;
        const finalHeight = Math.max(Math.abs(value) > 0 ? minBarHeight : 0, heightPercent);

        bar.className = `agile-bar ${slot.band}${slot.is_now ? " now" : ""} ${value < 0 ? "negative" : "positive"}`;
        bar.style.height = `${finalHeight}%`;
        bar.dataset.price = formatPrice(slot.value_inc_vat);
        bar.title =
            `${slot.source_day} slot ${slot.source_index}\n` +
            `${formatPrice(slot.value_inc_vat)}\n` +
            `${slot.valid_from} → ${slot.valid_to}` +
            (slot.is_now ? "\nCURRENT SLOT" : "");

        wrap.appendChild(bar);
        chart.appendChild(wrap);
    }

    const wraps = Array.from(chart.querySelectorAll(".agile-bar-wrap"));
    const chartRect = chart.getBoundingClientRect();

    data.slots.forEach((slot, index) => {
        const date = new Date(slot.valid_from);

        if (date.getMinutes() === 0 && wraps[index]) {
            const wrapRect = wraps[index].getBoundingClientRect();
            const label = document.createElement("div");

            label.className = "agile-time-label";
            label.textContent = formatHourLabel(slot.valid_from);

            const center = wrapRect.left - chartRect.left + wrapRect.width / 2;
            label.style.left = `${center}px`;

            timeAxis.appendChild(label);
        }
    });
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

async function loadDashboard() {
    if (dashboardRequestInFlight) return;
    dashboardRequestInFlight = true;

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

        latestUsageMetrics = data.usage_metrics ?? null;
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

        showPollIndicator();      // keep it visible
        setPollIndicatorError();  // turn it red

        if (output) {
            output.textContent = String(error);
        }

    } finally {
        dashboardRequestInFlight = false;
    }
}

function setupDebugToggle() {
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

function openSettingsModal() {
    const modal = document.getElementById("settings-modal");
    const backdrop = document.getElementById("settings-backdrop");

    if (!modal || !backdrop) return;

    modal.removeAttribute("hidden");
    backdrop.removeAttribute("hidden");
}

function closeSettingsModal() {
    const modal = document.getElementById("settings-modal");
    const backdrop = document.getElementById("settings-backdrop");

    if (!modal || !backdrop) return;

    modal.setAttribute("hidden", "");
    backdrop.setAttribute("hidden", "");
}

function setupSettingsModal() {
    const openButton = document.getElementById("settings-button");
    const closeButton = document.getElementById("settings-close-button");
    const backdrop = document.getElementById("settings-backdrop");

    if (openButton) {
        openButton.addEventListener("click", openSettingsModal);
    }

    if (closeButton) {
        closeButton.addEventListener("click", closeSettingsModal);
    }

    if (backdrop) {
        backdrop.addEventListener("click", closeSettingsModal);
    }

    document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
            closeSettingsModal();
        }
    });
}

/* History Modal */
async function loadHistoryModalPartial() {
    const root = document.getElementById("history-modal-root");
    if (!root) return;

    const response = await fetch("/static/partials/history-modal.html", {
        headers: { Accept: "text/html" },
        cache: "no-store",
    });

    if (!response.ok) {
        throw new Error(`Failed to load history modal partial: HTTP ${response.status}`);
    }

    root.innerHTML = await response.text();
}

function openHistoryModal() {
    const modal = document.getElementById("history-modal");
    const backdrop = document.getElementById("history-backdrop");

    if (!modal || !backdrop) return;

    modal.removeAttribute("hidden");
    backdrop.removeAttribute("hidden");

    loadHistoryYesterday();
}

function closeHistoryModal() {
    const modal = document.getElementById("history-modal");
    const backdrop = document.getElementById("history-backdrop");

    if (!modal || !backdrop) return;

    modal.setAttribute("hidden", "");
    backdrop.setAttribute("hidden", "");
}

function formatHistoryDateLabel(isoDate) {
    const date = new Date(`${isoDate}T12:00:00`);
    return date.toLocaleDateString("en-GB", {
        weekday: "short",
        day: "numeric",
        month: "long",
        year: "numeric",
    });
}

function renderHistoryChart(chartId, axisId, slots, fuel, yMaxId) {
    const chart = document.getElementById(chartId);
    const axis = document.getElementById(axisId);

    if (!chart || !axis) return;

    chart.innerHTML = "";
    axis.innerHTML = "";

    if (!slots || slots.length === 0) {
        chart.innerHTML = "<div style='color: var(--muted);'>No data</div>";
        return;
    }

    const values = slots.map((slot) => getHistorySlotValue(slot));
    const maxValue = Math.max(...values, 0.001);

    const yMaxEl = document.getElementById(yMaxId);
    if (yMaxEl) {
        yMaxEl.textContent =
            historyMetric === "cost"
                ? formatHistoryCost(maxValue)
                : formatHistoryKwh(maxValue);
    }

    slots.forEach((slot, index) => {
        const rawValue = getHistorySlotValue(slot);
        const ratio = rawValue / maxValue;
        const heightPercent = Math.max(6, ratio * 100);

        const bar = document.createElement("button");
        bar.type = "button";
        bar.className = `history-bar ${fuel}`;
        bar.style.height = `${heightPercent}%`;

        const labelValue =
            historyMetric === "cost"
                ? formatHistoryCost(rawValue)
                : formatHistoryKwh(rawValue);

        const start = new Date(slot.interval_start);
        const end = new Date(slot.interval_end);

        const startText = start.toLocaleTimeString("en-GB", {
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
        });

        const endText = end.toLocaleTimeString("en-GB", {
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
        });

        bar.dataset.value = labelValue;
        bar.title = `${labelValue}\n${startText} → ${endText}`;
        bar.setAttribute("aria-label", `${fuel} ${labelValue}, ${startText} to ${endText}`);

        bar.addEventListener("click", () => {
            const existing = chart.querySelector(".history-bar.selected");
            if (existing && existing !== bar) {
                existing.classList.remove("selected");
            }

            bar.classList.toggle("selected");
        });

        chart.appendChild(bar);

        if (start.getMinutes() === 0) {
            const label = document.createElement("div");
            label.className = "history-time-label";
            label.textContent = start.getHours().toString().padStart(2, "0");
            label.style.left = `${(index / slots.length) * 100}%`;
            axis.appendChild(label);
        }
    });
}

async function loadHistoryYesterday() {
    const response = await fetch("/api/history/yesterday", {
        headers: { Accept: "application/json" },
        cache: "no-store",
    });

    if (!response.ok) {
        throw new Error(`Failed to load history: HTTP ${response.status}`);
    }

    const data = await response.json();

    const dateLabel = document.getElementById("history-date-label");
    if (dateLabel && data.electricity?.date) {
        dateLabel.textContent = formatHistoryDateLabel(data.electricity.date);
    }

    const electricitySummary = document.getElementById("history-electricity-summary");
    const gasSummary = document.getElementById("history-gas-summary");

    if (electricitySummary) {
        electricitySummary.textContent = historyMetric === "cost" ? "Cost" : "kWh";
    }

    if (gasSummary) {
        gasSummary.textContent = historyMetric === "cost" ? "Cost" : "kWh";
    }

    const electricityTotal = document.getElementById("history-electricity-total");
    const electricityStanding = document.getElementById("history-electricity-standing");
    const gasTotal = document.getElementById("history-gas-total");
    const gasStanding = document.getElementById("history-gas-standing");

    if (electricityTotal) {
        electricityTotal.textContent =
            historyMetric === "cost"
                ? `Total ${formatGbp(data.electricity?.total_cost_gbp)}`
                : `Total ${formatHistoryKwh(data.electricity?.total_consumption_kwh)}`;
    }

    if (electricityStanding) {
        electricityStanding.textContent =
            historyMetric === "cost"
                ? `Standing ${formatGbp(data.electricity?.standing_charge_gbp)}`
                : "";
    }

    if (gasTotal) {
        gasTotal.textContent =
            historyMetric === "cost"
                ? `Total ${formatGbp(data.gas?.total_cost_gbp)}`
                : `Total ${formatHistoryKwh(data.gas?.total_consumption_kwh)}`;
    }

    if (gasStanding) {
        gasStanding.textContent =
            historyMetric === "cost"
                ? `Standing ${formatGbp(data.gas?.standing_charge_gbp)}`
                : "";
    }

    renderHistoryChart(
        "history-electricity-chart",
        "history-electricity-axis",
        data.electricity.slots,
        "electricity",
        "electricity-y-max"
    );

    renderHistoryChart(
        "history-gas-chart",
        "history-gas-axis",
        data.gas.slots,
        "gas",
        "gas-y-max"
    );
}

function setupHistoryModal() {
    const historyButton = document.getElementById("history-button");
    const root = document.getElementById("history-modal-root");

    if (!historyButton || !root) return;

    historyButton.addEventListener("click", openHistoryModal);

    root.addEventListener("click", (event) => {
        const target = event.target;

        if (!(target instanceof HTMLElement)) return;

        if (
            target.id === "history-close-button" ||
            target.id === "history-backdrop"
        ) {
            closeHistoryModal();
        }

        if (target.id === "history-metric-cost") {
            historyMetric = "cost";
            target.classList.add("active");
            document.getElementById("history-metric-kwh")?.classList.remove("active");
            loadHistoryYesterday();
        }

        if (target.id === "history-metric-kwh") {
            historyMetric = "kwh";
            target.classList.add("active");
            document.getElementById("history-metric-cost")?.classList.remove("active");
            loadHistoryYesterday();
        }
    });
}

async function init() {
    setupDebugToggle();
    setupSettingsModal();
    updateClock();

    await loadHistoryModalPartial();  // ✅ now valid

    setupHistoryModal();
    loadDashboard();

    setInterval(updateClock, 1000);
    setInterval(loadDashboard, 10000);
    setInterval(advanceUsageRotation, 8000);
}

init();