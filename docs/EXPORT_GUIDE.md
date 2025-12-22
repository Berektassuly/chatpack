# 📤 Export Guide

How to export your chat history from different messengers.

## Telegram

### Desktop (recommended)

1. Open **Telegram Desktop** (not mobile app!)
2. Go to **Settings** → **Advanced** → **Export Telegram data**
3. Select the chat you want to export
4. Configure export settings:
   - ✅ **Format: JSON** (required!)
   - ❌ Uncheck: Photos, Videos, Voice messages, Stickers
   - ✅ Check: Text messages only
5. Choose export location
6. Click **Export** and wait
7. Find `result.json` in the export folder

```bash
chatpack tg result.json
```

### What's exported

| Field | Included |
|-------|----------|
| Message text | ✅ |
| Sender name | ✅ |
| Timestamp | ✅ |
| Reply reference | ✅ |
| Edit timestamp | ✅ |
| Message ID | ✅ |
| Photos/Videos | ❌ (text only) |

---

## WhatsApp

### iPhone

1. Open the chat
2. Tap contact name at the top
3. Scroll down → **Export Chat**
4. Choose **Without Media**
5. Send to yourself via:
   - Email
   - AirDrop
   - Save to Files

### Android

1. Open the chat
2. Tap **⋮** (three dots menu)
3. **More** → **Export chat**
4. Choose **Without media**
5. Save or send the `.txt` file

```bash
chatpack wa "WhatsApp Chat with Mom.txt"
```

### Supported date formats

chatpack auto-detects your locale:

| Region | Format | Example |
|--------|--------|---------|
| US | M/D/YY, H:MM AM/PM | `[1/15/24, 10:30 AM]` |
| EU (dot) | DD.MM.YY, HH:MM | `[15.01.24, 10:30]` |
| EU (slash) | DD/MM/YYYY, HH:MM | `15/01/2024, 10:30 -` |
| Russia | DD.MM.YYYY, HH:MM | `26.10.2025, 20:40 -` |

---

## Instagram

### Step 1: Request your data

1. Go to [instagram.com](https://instagram.com) (web browser)
2. Log in to your account
3. **Settings** → **Your activity** → **Download your information**
4. Click **Request a download**
5. Select **Some of your information**
6. Check only **Messages** ✅
7. Choose:
   - **Format:** JSON
   - **Date range:** All time
8. Click **Submit request**

### Step 2: Wait for email

Instagram will email you when ready (can take hours or even days).

### Step 3: Download and extract

1. Download the ZIP file from the email link
2. Extract the archive
3. Navigate to: `messages/inbox/username_id/`
4. Find `message_1.json` (or multiple files for long chats)

```bash
chatpack ig message_1.json
```

### ⚠️ Mojibake fix

Instagram exports have broken encoding (UTF-8 stored as ISO-8859-1). 

**Before chatpack:**
```
ÐŸÑ€Ð¸Ð²ÐµÑ‚
```

**After chatpack:**
```
Привет
```

chatpack fixes this automatically!

---

## Discord

Discord doesn't have built-in export. Use **DiscordChatExporter** — a free, open-source tool.

### Step 1: Get DiscordChatExporter

1. Go to [github.com/Tyrrrz/DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter)
2. Download the latest release:
   - **Windows:** `.zip` with GUI
   - **macOS/Linux:** Use CLI version or Docker
3. Extract and run

### Step 2: Get your Discord token

1. Open Discord in browser (not desktop app)
2. Press `F12` to open Developer Tools
3. Go to **Network** tab
4. Send any message or refresh
5. Click on any request → **Headers** → find `Authorization`
6. Copy the token value

⚠️ **Never share your token with anyone!**

### Step 3: Export chat

**GUI (Windows):**
1. Paste your token
2. Select server and channel
3. Choose export format: **JSON** (recommended), TXT, or CSV
4. Click **Export**

**CLI:**
```bash
# Export as JSON
./DiscordChatExporter.Cli export -t "YOUR_TOKEN" -c CHANNEL_ID -f Json -o chat.json

# Export as TXT
./DiscordChatExporter.Cli export -t "YOUR_TOKEN" -c CHANNEL_ID -f PlainText -o chat.txt

# Export as CSV
./DiscordChatExporter.Cli export -t "YOUR_TOKEN" -c CHANNEL_ID -f Csv -o chat.csv
```

### Step 4: Use with chatpack

```bash
# JSON format (full metadata)
chatpack dc chat.json

# TXT format
chatpack dc chat.txt

# CSV format
chatpack dc chat.csv
```

### Supported formats

| Format | Full Metadata | Attachments | Stickers |
|--------|---------------|-------------|----------|
| JSON | ✅ IDs, replies, edits | ✅ | ✅ |
| TXT | ⚠️ Timestamps only | ✅ | ✅ |
| CSV | ⚠️ Timestamps only | ✅ | ❌ |

---

## Tips

### Combine multiple files

```bash
# Telegram - usually single file
chatpack tg result.json

# WhatsApp - single file per chat
chatpack wa chat1.txt
chatpack wa chat2.txt

# Instagram - may have multiple parts
cat message_1.json message_2.json > combined.json
chatpack ig combined.json

# Discord - export each channel separately
chatpack dc general.json
chatpack dc random.json
```

### Check file format

```bash
# Should be JSON
head -1 result.json
# Expected: {"name": "Chat Name", ...

# Should be WhatsApp TXT
head -1 chat.txt
# Expected: [1/15/24, 10:30 AM] Alice: Hello

# Discord JSON
head -1 discord.json
# Expected: {"guild": ..., "channel": ..., "messages": ...
```