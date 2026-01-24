# Скрипт для добавления переводов в messages.po
$file = "c:\project\ass\superset-src\apache_superset-6.0.0rc4\superset\translations\ru\LC_MESSAGES\messages.po"
$content = Get-Content $file -Raw -Encoding UTF8

# Словарь переводов
$translations = @{
    'msgid "Add description that will be displayed when hovering over the label..."' = 'msgstr "Добавьте описание, которое будет отображаться при наведении на метку..."'
    'msgid "DELETE"' = 'msgstr "УДАЛИТЬ"'
    'msgid "Dynamic group by name"' = 'msgstr "Динамическая группировка по имени"'
    'msgid "ECharts"' = 'msgstr "ECharts"'
    'msgid "EMAIL_REPORTS_CTA"' = 'msgstr "EMAIL_REPORTS_CTA"'
    'msgid "Failed to remove system dark theme: %s"' = 'msgstr "Не удалось удалить системную тёмную тему: %s"'
    'msgid "GROUP BY"' = 'msgstr "ГРУППИРОВКА"'
    'msgid "Group by settings (%s)"' = 'msgstr "Настройки группировки (%s)"'
    'msgid "NOT GROUPED BY"' = 'msgstr "БЕЗ ГРУППИРОВКИ"'
    'msgid "Name your dynamic group by"' = 'msgstr "Назовите вашу динамическую группировку"'
    'msgid "OVERWRITE"' = 'msgstr "ПЕРЕЗАПИСАТЬ"'
    'msgid "TEMPORAL_RANGE"' = 'msgstr "ВРЕМЕННОЙ_ДИАПАЗОН"'
    'msgid "This is custom error message for a"' = 'msgstr "Это пользовательское сообщение об ошибке для a"'
    'msgid "This is custom error message for b"' = 'msgstr "Это пользовательское сообщение об ошибке для b"'
    'msgid "WFS"' = 'msgstr "WFS"'
    'msgid "WMS"' = 'msgstr "WMS"'
    'msgid "XYZ"' = 'msgstr "XYZ"'
    'msgid "Your changes will be lost if you leave without saving."' = 'msgstr "Ваши изменения будут потеряны, если вы покинете страницу без сохранения."'
    'msgid "bolt"' = 'msgstr "молния"'
    'msgid "crontab"' = 'msgstr "crontab"'
    'msgid "error_message"' = 'msgstr "сообщение_об_ошибке"'
    'msgid "pivoted_xlsx"' = 'msgstr "сводная_таблица_xlsx"'
    'msgid "sql"' = 'msgstr "sql"'
    'msgid "step-after"' = 'msgstr "ступенчатый-после"'
    'msgid "step-before"' = 'msgstr "ступенчатый-до"'
    'msgid "valuename"' = 'msgstr "имя_значения"'
}

$count = 0
foreach ($msgid in $translations.Keys) {
    $pattern = [regex]::Escape($msgid) + '\r?\nmsgstr ""'
    $replacement = $msgid + "`n" + $translations[$msgid]
    if ($content -match $pattern) {
        $content = $content -replace $pattern, $replacement
        $count++
        Write-Host "Translated: $msgid"
    }
}

# Сохраняем файл
$content | Set-Content $file -Encoding UTF8 -NoNewline
Write-Host "`nTotal translations added: $count"
