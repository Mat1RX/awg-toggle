# awg-toggle

A lightweight, high-performance **Waybar** plugin written in **Rust** for managing **AmneziaWG** (WireGuard protocol with obfuscation) connections.

---

## Features

* **Quick Toggle:** Turn the currently selected VPN on or off with a left-click.
* **Config Cycling:** Easily switch between different `.conf` files in `/etc/amnezia/amneziawg/` using the mouse wheel or right-click.
* **JSON Output:** Native Waybar integration with support for tooltips, text icons, and CSS classes.
* **Secure:** Designed to run with minimal elevated privileges using specific system policies.

---

## Installation

### 1. Build the binary

Ensure you have the Rust toolchain (Cargo) installed.

```bash
git clone https://github.com/Mat1RX/awg-toggle.git
cd awg-toggle
cargo build --release

```

### 2. System Deployment

Move the binary to a system path and set the correct ownership to ensure security:

```bash
sudo cp target/release/wg-toggle /usr/local/bin/wg-toggle
sudo chown root:root /usr/local/bin/wg-toggle
sudo chmod 755 /usr/local/bin/wg-toggle

```

---

## Security Configuration

Managing network interfaces requires root privileges. To allow members of the **wheel** group to run this plugin without a password prompt, choose **one** of the following methods:

### Option A: Via Sudoers (Recommended)

Run `sudo visudo` and append the following line to the end of the file:

```sudoers
%wheel ALL=(ALL) NOPASSWD: /usr/local/bin/awg-toggle

```

### Option B: Via Polkit

Create the file `/etc/polkit-1/rules.d/10-awg-toggle.rules`:

```javascript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.policykit.exec" &&
        action.lookup("program") == "/usr/local/bin/awg-toggle" &&
        subject.isInGroup("wheel")) {
        return polkit.Result.YES;
    }
});

```

---

## Waybar Configuration

Add the following module to your Waybar `config.jsonc`:

```jsonc
"custom/awg": {
    "exec": "sudo /usr/local/bin/wg-toggle --status", // Use 'pkexec' if using Polkit
    "return-type": "json",
    "interval": 10,
    "on-click": "sudo /usr/local/bin/awg-toggle",
    "on-click-right": "sudo /usr/local/bin/awg-toggle next",
    "on-scroll-up": "sudo /usr/local/bin/awg-toggle previous",
    "on-scroll-down": "sudo /usr/local/bin/awg-toggle next",
    "tooltip": true
}

```

---

## Styling (CSS)

Customize the look in your `style.css`. The plugin provides `connected` and `disconnected` classes:

```css
#custom-awg {
    padding: 0 10px;
    color: #ffffff;
}

#custom-awg.connected {
    background-color: #a6e3a1; /* Green */
    color: #1e1e2e;
}

#custom-awg.disconnected {
    background-color: #f38ba8; /* Red */
    color: #1e1e2e;
}

```

---

## Usage Controls

| Action | Result |
| --- | --- |
| **Left Click** | Toggle current VPN On/Off |
| **Right Click** | Switch to the next available config |
| **Scroll Up** | Switch to the previous config |
| **Scroll Down** | Switch to the next config |

---

## Requirements

* `amneziawg-tools` (provides `awg` and `awg-quick`)
* `sudo` or `polkit` for privilege management
* A functional Waybar setup

**Note:** Ensure your configuration files are located in `/etc/amnezia/amneziawg/` and end with the `.conf` extension.
