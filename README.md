# 📊 Superset Portable — Russian Edition

> **Portable Apache Superset for Windows** — run from USB drive without installation!

[🇷🇺 Русская версия](#-superset-portable--русская-версия) | [🇬🇧 English version](#-features)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Superset](https://img.shields.io/badge/Based%20on-Apache%20Superset%206.0-orange.svg)](https://github.com/apache/superset)
[![Telegram](https://img.shields.io/badge/Telegram-@serg__borisovich-blue.svg)](https://t.me/serg_borisovich)

---

## 🇬🇧 English

### ✨ Features

- 🚀 **No installation required** — runs directly from USB
- 🇷🇺 **Russian interface** — full localization
- 📚 **Local documentation** — guides in Russian
- 🔒 **Works offline** — no internet required
- ⚡ **Rust launcher** — fast startup

### 🚀 Quick Start

#### Preparation (on PC with internet)

```powershell
# 1. Download Python Embeddable
.\setup\download_python.ps1

# 2. Install Superset
.\setup\install_superset.bat
```

#### Usage (on any PC)

```cmd
start_superset.bat
```

Browser opens at http://localhost:8088

| | |
|---|---|
| **Login** | `admin` |
| **Password** | `admin` |
| **Documentation** | http://localhost:8089 |

### 💼 Commercial Use

**Yes, allowed!** Apache 2.0 license permits:
- ✅ Commercial use
- ✅ Modification and distribution
- ✅ Selling services based on this software

### ⚙️ System Requirements

| Requirement | Minimum |
|-------------|---------|
| Windows | 10 / 11 |
| RAM | 2 GB |
| Disk | 500 MB |
| Internet | **Not required** |
| Admin rights | **Not required** |

---

## 🇷🇺 Superset Portable — Русская версия

### ✨ Особенности

- 🚀 **Без установки** — запускается прямо с флешки
- 🇷🇺 **Русский интерфейс** — полная локализация
- 📚 **Локальная документация** — руководство на русском
- 🔒 **Работает оффлайн** — не требует интернета
- ⚡ **Rust-лаунчер** — быстрый запуск

### 🚀 Быстрый старт

#### Подготовка (на ПК с интернетом)

```powershell
# 1. Скачать Python Embeddable
.\setup\download_python.ps1

# 2. Установить Superset
.\setup\install_superset.bat
```

#### Использование (на любом ПК)

```cmd
start_superset.bat
```

Откроется браузер на http://localhost:8088

| | |
|---|---|
| **Логин** | `admin` |
| **Пароль** | `admin` |
| **Документация** | http://localhost:8089 |

### 📚 Документация

| Раздел | Описание |
|--------|----------|
| [Руководство](docs/ru/user_guide.md) | Основы работы с Superset |
| [SQL Lab](docs/ru/sql_lab.md) | Работа с SQL-запросами |
| [Лицензия](docs/ru/licensing.md) | Apache 2.0 и форки |

### 💼 Коммерческое использование

**Да, разрешено!** Apache 2.0 лицензия позволяет:
- ✅ Использовать в коммерческих целях
- ✅ Модифицировать и распространять
- ✅ Продавать услуги на основе этого ПО

### ⚙️ Системные требования

| Требование | Минимум |
|------------|---------|
| Windows | 10 / 11 |
| RAM | 2 GB |
| Disk | 500 MB |
| Интернет | **Не нужен** |
| Права админа | **Не нужны** |

---

## 📁 Project Structure / Структура проекта

```
superset-portable/
├── start_superset.bat      # Start (Superset + Docs) / Запуск
├── superset-launcher.exe   # Rust CLI launcher / Rust лаунчер
├── python/                 # Embedded Python + Superset
├── superset_home/          # Database & config / БД и конфиг
├── docs/                   # Russian documentation / Документация
│   ├── index.html
│   └── ru/
├── src/                    # Rust source code / Исходный код Rust
├── LICENSE                 # Apache 2.0
└── NOTICE                  # Attribution / Атрибуция
```

## 🛠️ Build from Source / Сборка из исходников

```powershell
# Rust launcher
cargo build --release
```

## 📜 License / Лицензия

This project is based on [Apache Superset](https://github.com/apache/superset) and is distributed under **Apache License 2.0**.

Этот проект основан на [Apache Superset](https://github.com/apache/superset) и распространяется под лицензией **Apache License 2.0**.

See / Смотрите: [LICENSE](LICENSE) | [NOTICE](NOTICE)

## 📬 Contact / Контакты

**Telegram:** [@serg_borisovich](https://t.me/serg_borisovich)

---

<p align="center">
  Made with ❤️ for Russian-speaking Superset users
</p>
