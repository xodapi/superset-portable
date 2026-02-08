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

RELEASE_NAME = "superset-portable-v6.2-rzd"
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
]

# Также включаем setup/create_rzd_dashboard.py на случай пересоздания
INCLUDE_EXTRA = [
    "setup/create_rzd_dashboard.py",
    "setup/fix_db_uri.py",
    "setup/generate_demo_data.py",
    "setup/install_superset.bat",
    "setup/download_python.ps1",
    "target/release/create_dashboard.exe",
    "target/release/superset-launcher.exe",
    "docs/HOWTO_UPDATE_DATA.md",
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
    # Python exclusions
    "python/Scripts",
    "python/Lib/test",
    "python/Lib/unittest",
    "python/Lib/site-packages/pip",
    "python/Lib/site-packages/setuptools",
    "python/share",
    "python/doc",
    "python/tcl",
    "python/tools",
    "*.pdb",
    "*.dist-info",
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
    # Set console to utf-8 just in case
    if sys.platform == "win32":
        os.system("chcp 65001")
        
    print()
    print("=" * 60)
    print("  [INFO] Building Superset Portable Release")
    print("=" * 60)
    print()

    # ── Проверки ──────────────────────────────────────────────

    errors = []
    for d in INCLUDE_DIRS:
        p = ROOT_DIR / d
        if not p.exists():
            errors.append(f"Directory not found: {d}/")

    for f in INCLUDE_FILES:
        p = ROOT_DIR / f
        if not p.exists():
             errors.append(f"File not found: {f}")

    if errors:
        print("  [ERROR] Release build failed:\n")
        for e in errors:
            print(f"    [X] {e}")
        print()
        print("  Check requirements:")
        print("    1. python/ dir exists")
        print("    2. examples.db exists")
        print("    3. superset_home/superset.db exists")
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

    # Extra files (with path flattening for target/release binaries)
    for fname in INCLUDE_EXTRA:
        full_path = ROOT_DIR / fname
        if full_path.exists():
            # Flatten path for binaries built in target/release
            if "target/release" in fname:
                archive_path = f"{RELEASE_NAME}/{Path(fname).name}"
            else:
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
    print("  [OK] Release Built Successfully!")
    print("=" * 60)
    print()
    print(f"  File:    {ZIP_PATH}")
    print(f"  Size:    {format_size(zip_size)} (compression {ratio:.0f}%)")
    print(f"  Files:   {packed_count}")
    print(f"  Time:    {elapsed:.1f} sec")
    print()
    print("  --- Offline Installation ---")
    print()
    print(f"  1. Copy {ZIP_PATH.name} to USB drive")
    print("  2. Unzip on target PC")
    print(f"  3. Run {RELEASE_NAME}\\start_superset.bat")
    print("  4. Browser opens: http://localhost:8088")
    print("  5. Login: admin / Password: admin")
    print()

    # ── Также создаём распакованную копию ─────────────────────

    unpacked_dir = RELEASE_DIR / RELEASE_NAME
    if unpacked_dir.exists():
        print(f"  [INFO] Распакованная копия уже есть: {unpacked_dir}")
    print()


if __name__ == "__main__":
    main()
