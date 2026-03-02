# AnatolevichConvert

**Быстрый оффлайн конвертер документов для Linux**

Десктопное GTK4/Rust приложение для пакетной конвертации файлов между 22 форматами. Работает полностью оффлайн — без загрузки файлов на сервера, без ограничений по размеру и количеству.

## Возможности

- **22 формата** — PDF, DOCX, ODT, RTF, TXT, MD, HTML, EPUB, XLSX, CSV, ODS, PPTX, ODP, JPG, PNG, WebP, SVG, BMP, TIFF
- **Пакетная конвертация** — любое количество файлов за один раз
- **Параллельная обработка** — rayon для изображений, LibreOffice batch mode для документов
- **Drag & Drop** — перетаскивание файлов прямо в окно
- **Умный подбор формата** — кнопка 🧠 рекомендует оптимальный формат
- **Переименование** — шаблоны \`{name}\`, \`{n}\`, \`{date}\` + редактирование каждого файла
- **Автопапки** — при 5+ файлах создаётся подпапка с настраиваемым именем
- **Тёмная/светлая тема** — с сохранением между запусками
- **Пользовательские обои** — фоновое изображение в окне приложения
- **GNOME уведомления** — с кнопкой «Открыть папку»
- **История конвертаций** — просмотр и очистка
- **Горячие клавиши** — Ctrl+O (файлы), Ctrl+Q (выход)
- **Безопасность** — защита от path traversal, symlink attacks, log injection

## Требования

- **Fedora 43+** (или другой Linux с GTK4)
- **LibreOffice** — для документов, таблиц, презентаций
- **Pandoc** — для MD, HTML, TXT, EPUB
- **Rust 1.70+** — для сборки

\`\`\`bash
sudo dnf install libreoffice pandoc gtk4-devel
\`\`\`

## Установка

\`\`\`bash
git clone https://github.com/your-username/anatolevich-convert.git
cd anatolevich-convert
chmod +x install.sh
./install.sh
\`\`\`

После установки найдите «AnatolevichConvert» в меню GNOME (Super → набрать "Anatolevich").

## Сборка вручную

\`\`\`bash
cargo build --release
./target/release/anatolevich-convert
\`\`\`

## Использование

1. **Добавьте файлы** — кнопка «📂 Выбрать файлы» или перетащите в окно
2. **Выберите формат** — из выпадающего списка или нажмите 🧠 для рекомендации
3. **Настройте имена** (необязательно) — шаблон или клик на имя файла
4. **Конвертируйте** — кнопка «⚡ Конвертировать» → выберите папку
5. **Откройте результат** — кнопка «📂 Открыть папку» или уведомление

## Структура проекта

\`\`\`
anatolevich-convert/
├── Cargo.toml
├── install.sh
├── anatolevich-convert.desktop
├── icons/
│   ├── anatolevich-convert.svg
│   ├── anatolevich-convert-128.png
│   ├── anatolevich-convert-256.png
│   └── anatolevich-convert-512.png
├── styles/
│   ├── badges.css
│   ├── light.css
│   └── dark.css
└── src/
    ├── main.rs
    ├── app.rs
    ├── file_entry.rs
    ├── formats.rs
    ├── settings.rs
    ├── dialogs.rs
    ├── conversion.rs
    ├── history.rs
    ├── history_window.rs
    ├── notifications.rs
    ├── tests.rs
    └── converter/
        ├── mod.rs
        ├── libreoffice.rs
        ├── pandoc.rs
        └── image_backend.rs
\`\`\`

## Тестирование

\`\`\`bash
cargo test
\`\`\`

70 юнит-тестов покрывают: форматы, конвертацию, переименование, историю, настройки, безопасность.

## Бэкенды конвертации

| Исходный формат | Бэкенд | Параллелизм |
|---|---|---|
| DOCX, XLSX, PPTX, ODT, ODS, ODP, RTF, CSV, PDF | LibreOffice | Batch (один процесс) |
| MD, HTML, TXT, EPUB | Pandoc | Параллельный (rayon) |
| JPG, PNG, WebP, SVG, BMP, TIFF | image crate | Параллельный (rayon) |

## Лицензия

MIT

## Автор

Андрей — [GitHub](https://github.com/your-username)
