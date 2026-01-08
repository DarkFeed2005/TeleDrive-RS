# 🚀 TeleDrive-RS

**TeleDrive-RS** is a lightweight desktop application built in **Rust** that transforms Telegram into a personal cloud storage system. By utilizing the Telegram MTProto API, it allows you to bypass traditional bot limits and store files up to 2GB (4GB for Premium) with ease.

<p align="center">
<a href="https://www.w3schools.com/html/" target="_blank" rel="noreferrer"> <img src="https://skillicons.dev/icons?i=rust" alt="Rust" width="70" height="70"/> </a>
</p>


## ✨ Features
* **Unlimited Storage:** Store as much data as you want on Telegram's servers.
* **High Performance:** Built with Rust and `Tokio` for blazing-fast asynchronous uploads.
* **File Chunking:** Automatically splits large files into manageable parts for reliable transfer.
* **Modern GUI:** Clean interface built with `Slint`.
* **Local Metadata:** Uses `SQLite` to keep track of your file names and Telegram message IDs.
* **Secure:** Sensitive API credentials are managed via environment variables.

## 🛠️ Tech Stack
- **Language:** [Rust](https://www.rust-lang.org/)
- **API Wrapper:** [gramme-rs](https://github.com/m-rezaa/grammers) (MTProto)
- **GUI Framework:** [Slint](https://slint.dev/)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **Database:** [SQLite](https://sqlite.org/)

## 🚀 Getting Started

### 1. Prerequisites
- Install the [Rust toolchain](https://rustup.rs/).
- Get your `API_ID` and `API_HASH` from [my.telegram.org](https://my.telegram.org).

### 2. Setup
Clone the repository and create a `.env` file:
```bash
git clone [https://github.com/YOUR_USERNAME/TeleDrive-RS.git](https://github.com/YOUR_USERNAME/TeleDrive-RS.git)
cd TeleDrive-RS
touch .env
```
### 3. Add your credentials to the .env file:
```
TG_API_ID=1234567
TG_API_HASH=your_api_hash_here
```
### 4. Run
```
cargo run --release
```

## 📥 Download
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/YOUR_USERNAME/TeleDrive-RS?style=for-the-badge)](https://github.com/YOUR_USERNAME/TeleDrive-RS/releases/latest)

Click the badge above or go to the [Releases](https://github.com/YOUR_USERNAME/TeleDrive-RS/releases) page to download the latest `.exe` for Windows.


## 👨‍💻 Author

- **KpolitX** <a href="https://github.com/yourusername" target="_blank" rel="noreferrer"> <img src="https://skillicons.dev/icons?i=github" alt="github" width="20" height="20"/> </a>
- LinkedIn <a href="https://www.linkedin.com/in/yourprofile/" target="_blank" rel="noreferrer"> <img src="https://raw.githubusercontent.com/devicons/devicon/master/icons/linkedin/linkedin-original.svg" alt="linkedin" width="20" height="20"/> </a>
- Instagram <a href="https://www.instagram.com/yourusername/" target="_blank" rel="noreferrer"> <img src="https://skillicons.dev/icons?i=instagram" alt="instagram" width="20" height="20"/> </a>

