# ⚡ Home Energy Dashboard (IHD)

A lightweight, real-time home energy dashboard designed for a **5" 800×400 display**, combining:

- 🧠 Smart Octopus Agile pricing (today + tomorrow)
- 🔌 Live Home Assistant power usage
- ☀️ Solar generation monitoring
- 🔋 Battery status (placeholder / future integration)

Built with:
- 🦀 Rust backend (data + API)
- 🌐 Simple HTML/CSS/JS frontend (display only)

---

## 📸 Features

- Real-time clock  
- House usage gauge (dynamic colour scaling)  
- Solar generation gauge (based on max system output)  
- Battery level indicator  
- 48-slot Agile pricing graph (today + tomorrow)  
- Auto-refresh every 60 seconds  

---

## 🏗️ Architecture

```
Home Assistant  →  Rust Backend  →  Web UI
     (API)           (/api)        (Display only)
```

---

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

---

## 🔑 Home Assistant Token

Profile → Security → Long-Lived Access Tokens → Create Token

---

## 🔄 Data Sources

| Purpose        | Entity ID                              |
|----------------|----------------------------------------|
| House Usage    | `sensor.total_power_being_used`         |
| Solar Output   | `sensor.solar_panel_led_sensor_power`  |

---

## 🚀 Roadmap

- Appliance scheduling suggestions  
- Appliance detection via HA  
- Battery integration  
- Historical charts  

---

## 💡 Philosophy

> “Cost efficiency is great — simplicity should always win.”
