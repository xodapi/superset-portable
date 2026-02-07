#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
=============================================================
  Build Release — Сборка портативного релиза Superset
=============================================================

Собирает минимальный ZIP-архив для переноса на закрытый контур:
  - python/ (embedded Python + Superset + все зависимости)
  - superset_home/ (superset.db с дашбордом, конфиг)
  - examples.db (данные РЖД)
  - docs/ (документация + демо CSV)
  - start_superset.bat
  - README.md, QUICKSTART.md, LICENSE, NOTICE

НЕ включает: src/, Cargo.*, target/, setup/, tests/, .agent/ и пр.

Запуск:
    python\python.exe build_release.py
"""

import os
import sys
import zipfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent

RELEASE_NAME = "superset-portable-v6.0-rzd"
RELEASE_DIR = ROOT_DIR / "release"
ZIP_PATH = RELEASE_DIR / f"{RELEASE_NAME}.zip"

# ─── Что включаем в релиз ───────────────────────────────────────────────────

# Директории (рекурсивно)
INCLUDE_DIRS = [
    "python",
    "superset_home",
    "docs",
]

# Отдельные файлы из корня
INCLUDE_FILES = [
    "start_superset.bat",
    "examples.db",
    "README.md",
    "QUICKSTART.md",
    "LICENSE",
    "NOTICE",
    "superset-launcher.exe",
]

# Также включаем setup/create_rzd_dashboard.py на случай пересоздания
INCLUDE_EXTRA = [
    "setup/create_rzd_dashboard.py",
    "setup/fix_db_uri.py",
    "setup/generate_demo_data.py",
    "setup/install_superset.bat",
    "setup/download_python.ps1",
]

# Исключения внутри включаемых директорий
EXCLUDE_PATTERNS = [
    "__pycache__",
    ".pyc",
    ".pyo",
    "*.egg-info",
    ".git",
    ".DS_Store",
    "Thumbs.db",
    # superset_home exclusions
    "superset_home/uploads",
    "superset_home/__pycache__",
]


def should_exclude(rel_path_str):
    """Проверить, нужно ли исключить файл."""
    for pattern in EXCLUDE_PATTERNS:
        if pattern in rel_path_str:
            return True
    return False


def get_dir_size(path):
    """Размер директории в байтах."""
    total = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            try:
                total += os.path.getsize(fp)
            except OSError:
                pass
    return total


def format_size(size_bytes):
    """Человекочитаемый размер."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 * 1024:
        return f"{size_bytes / 1024:.1f} KB"
    elif size_bytes < 1024 * 1024 * 1024:
        return f"{size_bytes / (1024 * 1024):.1f} MB"
    else:
        return f"{size_bytes / (1024 * 1024 * 1024):.2f} GB"


def main():
    print()
    print("=" * 60)
    print("  📦 Сборка портативного релиза Superset")
    print("=" * 60)
    print()

    # ── Проверки ──────────────────────────────────────────────

    errors = []
    for d in INCLUDE_DIRS:
        p = ROOT_DIR / d
        if not p.exists():
            errors.append(f"Директория не найдена: {d}/")

    for f in INCLUDE_FILES:
        p = ROOT_DIR / f
        if not p.exists():
            if f == "superset-launcher.exe":
                print(f"  [WARN] {f} не найден — пропускаю (опционально)")
            else:
                errors.append(f"Файл не найден: {f}")

    if errors:
        print("  [ОШИБКА] Не удалось собрать релиз:\n")
        for e in errors:
            print(f"    ❌ {e}")
        print()
        print("  Убедитесь что:")
        print("    1. python/ содержит embedded Python + Superset")
        print("    2. examples.db создана (python\\python.exe setup\\create_rzd_dashboard.py)")
        print("    3. superset_home/superset.db существует")
        sys.exit(1)

    # ── Сбор файлов ───────────────────────────────────────────

    print("  Сбор файлов...")
    files_to_pack = []  # (source_path, archive_path)

    # Директории
    for dir_name in INCLUDE_DIRS:
        dir_path = ROOT_DIR / dir_name
        for root, dirs, files in os.walk(dir_path):
            # Filter out excluded directories in-place
            dirs[:] = [d for d in dirs if not should_exclude(d)]
            for fname in files:
                full_path = Path(root) / fname
                rel_path = full_path.relative_to(ROOT_DIR)
                rel_str = str(rel_path)
                if not should_exclude(rel_str):
                    archive_path = f"{RELEASE_NAME}/{rel_str}"
                    files_to_pack.append((str(full_path), archive_path))

    # Отдельные файлы
    for fname in INCLUDE_FILES:
        full_path = ROOT_DIR / fname
        if full_path.exists():
            archive_path = f"{RELEASE_NAME}/{fname}"
            files_to_pack.append((str(full_path), archive_path))

    # Дополнительные файлы
    for fname in INCLUDE_EXTRA:
        full_path = ROOT_DIR / fname
        if full_path.exists():
            archive_path = f"{RELEASE_NAME}/{fname}"
            files_to_pack.append((str(full_path), archive_path))

    total_source_size = sum(
        os.path.getsize(f[0]) for f in files_to_pack if os.path.isfile(f[0])
    )
    print(f"  Файлов: {len(files_to_pack)}")
    print(f"  Размер до сжатия: {format_size(total_source_size)}")
    print()

    # ── Сжатие ────────────────────────────────────────────────

    RELEASE_DIR.mkdir(exist_ok=True)

    if ZIP_PATH.exists():
        ZIP_PATH.unlink()
        print(f"  [OK] Старый архив удалён")

    print(f"  Упаковка в {ZIP_PATH.name}...")
    start_time = time.time()

    packed_count = 0
    with zipfile.ZipFile(
        str(ZIP_PATH), "w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=6,
    ) as zf:
        for source_path, archive_path in files_to_pack:
            try:
                zf.write(source_path, archive_path)
                packed_count += 1
                if packed_count % 500 == 0:
                    print(f"    ... {packed_count} файлов упаковано")
            except Exception as e:
                print(f"  [WARN] Пропущен {source_path}: {e}")

    elapsed = time.time() - start_time
    zip_size = ZIP_PATH.stat().st_size
    ratio = (1 - zip_size / total_source_size) * 100 if total_source_size > 0 else 0

    print()
    print("=" * 60)
    print("  ✅ Релиз собран!")
    print("=" * 60)
    print()
    print(f"  📦 Файл:    {ZIP_PATH}")
    print(f"  📊 Размер:  {format_size(zip_size)} (сжатие {ratio:.0f}%)")
    print(f"  📁 Файлов:  {packed_count}")
    print(f"  ⏱️  Время:   {elapsed:.1f} сек")
    print()
    print("  ─── Инструкция для закрытого контура ───")
    print()
    print(f"  1. Скопируйте {ZIP_PATH.name} на флешку")
    print("  2. Распакуйте в любую папку на целевом ПК")
    print(f"  3. Запустите {RELEASE_NAME}\\start_superset.bat")
    print("  4. Откроется браузер: http://localhost:8088")
    print("  5. Логин: admin / Пароль: admin")
    print()
    print("  Дашборд РЖД: http://localhost:8088/superset/dashboard/rzd_analytics/")
    print()

    # ── Также создаём распакованную копию ─────────────────────

    unpacked_dir = RELEASE_DIR / RELEASE_NAME
    if unpacked_dir.exists():
        print(f"  [INFO] Распакованная копия уже есть: {unpacked_dir}")
    print()


if __name__ == "__main__":
    main()
