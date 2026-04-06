export function openSettingsModal() {
    const modal = document.getElementById("settings-modal");
    const backdrop = document.getElementById("settings-backdrop");

    if (!modal || !backdrop) return;

    modal.removeAttribute("hidden");
    backdrop.removeAttribute("hidden");
}

export function closeSettingsModal() {
    const modal = document.getElementById("settings-modal");
    const backdrop = document.getElementById("settings-backdrop");

    if (!modal || !backdrop) return;

    modal.setAttribute("hidden", "");
    backdrop.setAttribute("hidden", "");
}

export function setupSettingsModal() {
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