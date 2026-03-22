function formatPrice(value) {
    return `${value.toFixed(2)}p`;
}

function formatHourLabel(isoString) {
    const date = new Date(isoString);
    return date
        .toLocaleTimeString("en-GB", {
            hour: "numeric",
            hour12: true,
        })
        .toLowerCase();
}

function formatClock(now = new Date()) {
    return now.toLocaleTimeString("en-GB", {
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
    });
}

function updateClock() {
    const clock = document.getElementById("header-time");
    if (!clock) return;
    clock.textContent = formatClock();
}

function updateHouseUsageGauge(watts) {
    const gaugeArc = document.getElementById("usage-gauge-electric");

    if (!gaugeArc || typeof watts !== "number") return;

    const maxWatts = 4000;
    const clampedWatts = clamp(watts, 0, maxWatts);

    // Non-linear visual scaling so low values are easier to see
    const linearRatio = clampedWatts / maxWatts;
    let percentage = Math.sqrt(linearRatio) * 100;

    // Keep a visible minimum when non-zero
    if (clampedWatts > 0) {
        percentage = Math.max(percentage, 10);
    }

    const colour = getHouseUsageColour(clampedWatts);

    gaugeArc.setAttribute("stroke-dasharray", `${percentage} 100`);
    gaugeArc.style.stroke = colour;
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

function getSolarColor(watts) {
    if (watts < 200) return "#22c55e";   // green
    if (watts < 1500) return "#84cc16";  // lime
    if (watts < 3000) return "#f97316";  // orange
    return "#ef4444";                    // red
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

    // Track always stays grey
    gaugeTrack.style.stroke = "";
}

function renderAgileChart(data) {
    const chart = document.getElementById("agile-chart");
    const summary = document.getElementById("agile-summary");
    const timeAxis = document.getElementById("agile-time-axis");

    timeAxis.innerHTML = "";
    chart.innerHTML = "";

    if (!data.slots || data.slots.length === 0) {
        summary.textContent = "No future Agile slots available";
        chart.innerHTML = "<div style='color: var(--muted);'>No data</div>";
        return;
    }

    const maxPrice = Math.max(...data.slots.map((slot) => slot.value_inc_vat));
    const minBarHeight = 12;

    summary.textContent = `${data.slot_count} future slots loaded`;

    for (const slot of data.slots) {
        const bar = document.createElement("div");
        const heightPercent = (slot.value_inc_vat / maxPrice) * 85;
        const finalHeight = Math.max(minBarHeight, heightPercent);

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
        const minutes = date.getMinutes();

        if (minutes === 0 && bars[index]) {
            const barRect = bars[index].getBoundingClientRect();
            const label = document.createElement("div");

            label.className = "agile-time-label";
            label.textContent = formatHourLabel(slot.valid_from);

            const barCenter = barRect.left - chartRect.left + barRect.width / 2;
            label.style.left = `${barCenter}px`;

            timeAxis.appendChild(label);
        }
    });
}

async function loadDashboard() {
    const status = document.getElementById("status");
    const output = document.getElementById("output");

    try {
        status.textContent = "Fetching /api/dashboard …";

        const response = await fetch("/api/dashboard", {
            headers: { Accept: "application/json" },
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();

        status.textContent = `Loaded ${data.agile.slot_count} slots`;
        output.textContent = JSON.stringify(data, null, 2);

        if (typeof data.live?.house_power_w === "number") {
            const power = Math.round(data.live.house_power_w);

            const valueEl = document.querySelector("#usage-panel .panel-value");
            if (valueEl) valueEl.textContent = `${power}W`;

            updateHouseUsageGauge(power);
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
            updateSolarGauge(0);
        }

        renderAgileChart(data.agile);
    } catch (error) {
        status.textContent = "Failed to load data";
        output.textContent = String(error);
    }
}

function setupDebugToggle() {
    const debugToggle = document.getElementById("debug-toggle");

    debugToggle.addEventListener("click", () => {
        const debug = document.getElementById("debug");
        const button = document.getElementById("debug-toggle");
        const isHidden = debug.hasAttribute("hidden");

        if (isHidden) {
            debug.removeAttribute("hidden");
            button.textContent = "Hide raw JSON";
        } else {
            debug.setAttribute("hidden", "");
            button.textContent = "Show raw JSON";
        }
    });
}

function init() {
    setupDebugToggle();
    updateClock();
    loadDashboard();

    setInterval(updateClock, 1000);
    setInterval(loadDashboard, 60000);
}

init();