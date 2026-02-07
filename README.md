# 📊 Superset Portable

[![License](https://img.shields.io/github/license/user/superset-portable?style=flat-square)](LICENSE)
[![Release](https://img.shields.io/github/v/release/user/superset-portable?style=flat-square)](https://github.com/user/superset-portable/releases)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-Embedded-blue?style=flat-square)](https://www.python.org/)

**Superset Portable** is a standalone, USB-ready distribution of [Apache Superset](https://github.com/apache/superset) for Windows. It requires **no installation**, **no admin rights**, and works **offline**.

Perfect for secure environments (closed loop), demos, and rapid analytics deployment.

[🇷🇺 Русская версия](#-русская-версия) | [🇬🇧 English Version](#-english-version)

---

## 🚀 Key Features

- **Portable**: Runs from a USB flash drive or any folder.
- **Standalone**: Embedded Python and SQLite. No dependencies required.
- **Fast**: High-performance Rust launcher with server pre-warming.
- **Secure**: Offline-first design, no external requests.
- **Easy Data Update**: Includes `create_dashboard.exe` tool to update data from Excel/CSV without Python.

---

## 🛠️ Quick Start

### 1. Download
Get the latest release from the [Releases Page](../../releases).

### 2. Run
Extract the ZIP archive and run:
```cmd
start_superset.bat
```

### 3. Login
- **URL**: [http://localhost:8088](http://localhost:8088)
- **User**: `admin`
- **Password**: `admin`

---

## 🏗️ Project Structure

```
superset-portable/
├── start_superset.bat      # 🚀 Entry point
├── superset-launcher.exe   # 🦀 Main Rust executable
├── create_dashboard.exe    # 📊 Data update tool
├── data/                   # 📂 Excel/CSV source files
├── python/                 # 🐍 Embedded Python 3.8
├── superset_home/          # 🗄️ Database & Config
└── docs/                   # 📚 Documentation
```

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository.
2. Create your feature branch.
3. Commit your changes.
4. Push to the branch.
5. Open a Pull Request.

---

## 📜 License

Distributed under the **Apache 2.0 License**. See [LICENSE](LICENSE) for more information.

---

<br>

## 🇷🇺 Русская версия

**Superset Portable** — это готовая к работе версия Apache Superset для Windows, которая запускается без установки.

Идеально подходит для закрытых контуров (без интернета), демонстраций и быстрого развертывания аналитики.

### ✨ Особенности

- **Портативность**: Работает с флешки.
- **Автономность**: Встроенный Python и SQLite. Ничего не нужно устанавливать.
- **Скорость**: Быстрый лаунчер на Rust с предзагрузкой сервера.
- **Обновление данных**: Утилита `create_dashboard.exe` обновляет дашборды из Excel/CSV.

### 🚀 Быстрый старт

#### 1. Скачать
Загрузите архив со страницы [Релизы](../../releases).

#### 2. Запустить
Распакуйте и запустите:
```cmd
start_superset.bat
```

#### 3. Войти
- **Адрес**: [http://localhost:8088](http://localhost:8088)
- **Логин**: `admin`
- **Пароль**: `admin`

### 📚 Документация

- [Как обновить данные (Excel/CSV)](docs/HOWTO_UPDATE_DATA.md)
- [История версий](docs/RELEASES.md)

### 📬 Контакты

Автор: [@serg_borisovich](https://t.me/serg_borisovich)
