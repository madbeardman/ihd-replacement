# ⚡ Home Energy Dashboard (IHD)

> Built for a dedicated always-on home display — fast, local-first, and
> energy-aware.

A lightweight, real-time home energy dashboard designed for a **5" 800×480
touchscreen**, combining:

- 🧠 Smart Octopus Agile pricing (dynamic window)
- 🔌 Live Home Assistant power usage
- ☀️ Solar generation monitoring
- 📊 Historical energy usage (day / week / month)
- ⚙️ Persistent user settings (local file-based)

Built with:
- 🦀 Rust backend (data + API + persistence)
- 🌐 Simple HTML/CSS/JS frontend (touch-first display)



## 📸 Features

### ⚡ Live Dashboard
- Real-time clock  
- House usage gauge (dynamic colour scaling)  
- Solar generation gauge (based on system max output)  
- Appliance status (washer / dishwasher / dryer)  
- Agile pricing chart (adaptive window size)  
- Auto-refresh every 10 seconds  



### 📊 History View
- Day view (30-minute slots)  
- Week view (daily totals)  
- Month view (daily totals)  
- Toggle between:
  - Cost (£)
  - kWh (future expansion)  



### ⚙️ Settings
- Adjustable Agile chart window:
  - 12 hours (24 slots)
  - 18 hours (36 slots)
  - 24 hours (48 slots)
- Persisted locally (`data/settings.json`)
- Applied instantly (no restart required)



## 🏗️ Architecture

```
Home Assistant  →  Rust Backend  →  Web UI
     (API)           (/api)        (Touch display)
                         ↓
                  Local storage
              (agile + history + settings)
```



## ⚙️ Setup

### 1. Clone the project

```bash
git clone <your-repo>
cd <your-repo>
```



### 2. Configure environment variables

Create a `.env` file:

```env
HOME_ASSISTANT_URL=http://homeassistant.local:8123
HOME_ASSISTANT_TOKEN=your_long_lived_access_token_here
```



### 3. Run backend

```bash
cargo run
```



### 4. Open dashboard

```
http://localhost:3000
```



## 🔑 Home Assistant Token

Home Assistant → Profile → Security → Long-Lived Access Tokens



## 🔄 Data Sources

| Purpose        | Entity ID                              |
|----------------|----------------------------------------|
| House Usage    | `sensor.total_power_being_used`         |
| Solar Output   | `sensor.solar_panel_led_sensor_power`  |
| Appliances     | (derived from power usage thresholds)  |



## 💾 Data Storage

Stored locally under `/data`:

```
data/
├── agile/       # Agile pricing (daily JSON files)
├── history/     # Electricity + gas history
└── settings.json
```



## 🔌 API Endpoints

| Endpoint                    | Description                     |
|----------------------------|--------------------------------|
| `/api/dashboard`           | Full dashboard state           |
| `/api/agile`               | Agile rolling window           |
| `/api/settings` (GET/POST) | Load / update settings         |
| `/api/history/day`         | Day history                    |
| `/api/history/week`        | Week history                   |
| `/api/history/month`       | Month history                  |



## 🚀 Roadmap

- Appliance scheduling recommendations (based on cheapest windows)  
- Battery integration (charge/discharge insight)  
- Cost forecasting (today / tomorrow projections)  
- Notifications / companion app  
- Advanced caching & performance tuning  



## 💡 Philosophy

> “Cost efficiency is great — simplicity should always win.”
