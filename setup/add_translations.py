# -*- coding: utf-8 -*-
import re

file_path = r"c:\project\ass\superset-src\apache_superset-6.0.0rc4\superset\translations\ru\LC_MESSAGES\messages.po"

# Translations dictionary
translations = {
    'Add description that will be displayed when hovering over the label...': 'Добавьте описание, которое будет отображаться при наведении на метку...',
    'DELETE': 'УДАЛИТЬ',
    'Dynamic group by name': 'Динамическая группировка по имени',
    'ECharts': 'ECharts',
    'EMAIL_REPORTS_CTA': 'EMAIL_REPORTS_CTA',
    'Failed to remove system dark theme: %s': 'Не удалось удалить системную тёмную тему: %s',
    'GROUP BY': 'ГРУППИРОВКА',
    'Group by settings (%s)': 'Настройки группировки (%s)',
    'NOT GROUPED BY': 'БЕЗ ГРУППИРОВКИ',
    'Name your dynamic group by': 'Назовите вашу динамическую группировку',
    'OVERWRITE': 'ПЕРЕЗАПИСАТЬ',
    'TEMPORAL_RANGE': 'ВРЕМЕННОЙ_ДИАПАЗОН',
    'This is custom error message for a': 'Это пользовательское сообщение об ошибке для a',
    'This is custom error message for b': 'Это пользовательское сообщение об ошибке для b',
    'WFS': 'WFS',
    'WMS': 'WMS',
    'XYZ': 'XYZ',
    'Your changes will be lost if you leave without saving.': 'Ваши изменения будут потеряны, если вы покинете страницу без сохранения.',
    'bolt': 'молния',
    'crontab': 'crontab',
    'error_message': 'сообщение_об_ошибке',
    'pivoted_xlsx': 'сводная_таблица_xlsx',
    'sql': 'sql',
    'step-after': 'ступенчатый-после',
    'step-before': 'ступенчатый-до',
    'valuename': 'имя_значения',
}

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

count = 0
for msgid, msgstr in translations.items():
    # Pattern to find untranslated strings
    pattern = rf'msgid "{re.escape(msgid)}"\nmsgstr ""'
    replacement = f'msgid "{msgid}"\nmsgstr "{msgstr}"'
    
    if re.search(pattern, content):
        content = re.sub(pattern, replacement, content)
        count += 1
        print(f"Translated: {msgid[:50]}...")

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)

print(f"\nTotal translations added: {count}")
