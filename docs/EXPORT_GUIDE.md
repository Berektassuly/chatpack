# Export Guide

Use this guide to create files that `chatpack` can parse. Messenger UI labels change over time, but the important requirements stay the same: prefer machine-readable exports, choose JSON where available, and export without media when you only need text for LLM/RAG processing.

## Telegram

`chatpack` supports JSON exports from Telegram Desktop.

### Individual Chat

1. Install or open the official Telegram Desktop app.
2. Open the chat, group, or channel.
3. Open the chat menu and choose **Export chat history**.
4. Select **JSON** as the format.
5. Disable large media types unless you need them elsewhere.
6. Export and locate `result.json`.

```bash
chatpack tg result.json
```

### Full Account Export

Telegram Desktop also supports full data export via **Settings > Advanced > Export Telegram data**. `chatpack` is designed for chat-message JSON files, so point it at the relevant `result.json` from the export.

### Parsed Fields

| Field | Support |
|-------|---------|
| Message text | Yes |
| Sender name | Yes |
| Timestamp | Yes |
| Message ID | Yes |
| Reply reference | Yes |
| Edited timestamp | Yes |
| Formatted text entities | Flattened to readable text |
| Service messages | Filtered |
| Media files | Not imported as binary files |

## WhatsApp

`chatpack` supports the plain-text `.txt` file created by WhatsApp's per-chat export.

### iPhone

1. Open the chat.
2. Tap the contact or group name.
3. Choose **Export Chat**.
4. Choose **Without Media** for the cleanest text export.
5. Save or share the `.txt` file.

### Android

1. Open the chat.
2. Open the three-dot menu.
3. Choose **More > Export chat**.
4. Choose **Without media** for the cleanest text export.
5. Save or share the `.txt` file.

```bash
chatpack wa "WhatsApp Chat.txt"
```

### Supported Date Formats

`chatpack` auto-detects these WhatsApp timestamp styles:

| Variant | Example |
|---------|---------|
| US bracketed, 12-hour or 24-hour | `[1/15/24, 10:30 AM] Alice: Hello` |
| EU dot bracketed | `[15.01.24, 10:30] Alice: Hello` |
| EU dot without brackets | `15.01.2024, 10:30 - Alice: Hello` |
| EU slash without brackets | `15/01/2024, 10:30 - Alice: Hello` |
| EU slash bracketed | `[15/01/2024, 10:30] Alice: Hello` |

Multiline messages are preserved. Common WhatsApp system notices are filtered, while media placeholders such as `<Media omitted>` are preserved as message content.

## Instagram

`chatpack` supports JSON message exports from Meta's Instagram data export.

1. Open [Instagram Accounts Center](https://accountscenter.instagram.com/info_and_permissions/).
2. Go to **Your information and permissions**.
3. Choose **Download or export your information**.
4. Select the Instagram account.
5. Choose **Some of your information** and select **Messages**.
6. Choose **Download to device**.
7. Select **JSON** as the format and choose the date range.
8. Wait for Meta to prepare the archive, then download and extract it.
9. Find files like `messages/inbox/<thread>/message_1.json`.

```bash
chatpack ig message_1.json
```

Long conversations can be split across `message_1.json`, `message_2.json`, and so on. Process each file separately or combine the parsed messages in your own pipeline.

### Encoding Fix

Instagram exports can contain mojibake, where UTF-8 text is stored as if it were ISO-8859-1. `chatpack` fixes this by default.

```text
Before: ÐŸÑ€Ð¸Ð²ÐµÑ‚
After:  Привет
```

## Discord

Discord does not provide a first-party channel export that matches `chatpack`'s parser. Use [DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter), which can export Discord channels and DMs to JSON, TXT, CSV, and HTML.

> [!IMPORTANT]
> Prefer a bot token when you can access the channel with a bot. DiscordChatExporter supports user tokens, but its own README warns that automating user accounts may violate Discord's Terms of Service and can result in a ban. Treat any Discord token like a password.

### Recommended Export

Use JSON when possible; it preserves the most metadata.

```bash
DiscordChatExporter.Cli export -t "TOKEN" -c CHANNEL_ID -f Json -o chat.json
chatpack dc chat.json
```

`chatpack` also supports DiscordChatExporter TXT and CSV:

```bash
DiscordChatExporter.Cli export -t "TOKEN" -c CHANNEL_ID -f PlainText -o chat.txt
DiscordChatExporter.Cli export -t "TOKEN" -c CHANNEL_ID -f Csv -o chat.csv

chatpack dc chat.txt
chatpack dc chat.csv
```

HTML exports are useful for humans, but `chatpack` does not parse HTML.

### Supported Discord Metadata

| Format | IDs | Timestamps | Replies | Edits | Attachments | Stickers |
|--------|-----|------------|---------|-------|-------------|----------|
| JSON | Yes | Yes | Yes | Yes | Yes | Yes |
| TXT | No | Yes | No | No | Yes | Yes |
| CSV | No | Yes | No | No | Yes | No |

## References

- [Telegram: Chat Export Tool](https://telegram.org/blog/export-and-more)
- [Meta Help Center: Export Instagram information](https://www.facebook.com/help/181231772500920)
- [DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter)
