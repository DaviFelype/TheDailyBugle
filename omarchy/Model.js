.pragma library

// Pure display logic for the bar widget, kept out of QML so node can test it
// (tests/js). No QML or Quickshell APIs here.
//
// The binary already guarantees the JSON contract (schema/feed-v1.schema.json).
// Everything is checked again here anyway: the plugin must stay safe even with
// a wrong or tampered binary on PATH.

// The bar has room for two digits; anything above reads as "a lot".
var MAX_SHOWN_COUNT = 99

var SCHEMA_VERSION = 1
var MAX_ITEMS = 200
var MAX_TITLE_CHARS = 300
var MAX_SOURCE_CHARS = 64
var MAX_URL_LENGTH = 2048
var CATEGORIES = ["vulnerability", "ransomware", "news"]

// Nerd Font glyphs (Material Design Icons): newspaper and alert.
var FEED_GLYPH = "\u{F0395}"
var ALERT_GLYPH = "\u{F0026}"

// Fixed texts: the binary's stderr is never shown, so a hostile one cannot
// put words in the panel.
var ERROR_MESSAGE = "Could not load the feed. Is thedailybugle installed and on the PATH?"
var INCOMPATIBLE_MESSAGE = "The thedailybugle binary and this plugin do not match. Update both."

function barLabel(count) {
  if (typeof count !== "number" || !Number.isInteger(count) || count <= 0) return ""
  return count > MAX_SHOWN_COUNT ? MAX_SHOWN_COUNT + "+" : String(count)
}

// Before the first run finishes.
function initialState() {
  return { status: "loading", items: [] }
}

// Reads one run of `thedailybugle feed --json`. `exitCode` is null when the
// binary never started. Status is "ok", "error" or "incompatible".
function parseFeed(stdout, exitCode) {
  if (exitCode !== 0 || typeof stdout !== "string") return { status: "error", items: [] }

  var document
  try {
    document = JSON.parse(stdout)
  } catch (error) {
    return { status: "error", items: [] }
  }
  if (!isPlainObject(document)) return { status: "error", items: [] }
  if (typeof document.schemaVersion === "number" && document.schemaVersion !== SCHEMA_VERSION)
    return { status: "incompatible", items: [] }
  if (document.schemaVersion !== SCHEMA_VERSION || !Array.isArray(document.items))
    return { status: "error", items: [] }

  var items = []
  for (var index = 0; index < document.items.length && items.length < MAX_ITEMS; index++) {
    var item = cleanItem(document.items[index])
    if (item) items.push(item)
  }
  return { status: "ok", items: items }
}

// The state after a run. A failed run keeps the last good items on screen.
function nextState(previous, stdout, exitCode) {
  var parsed = parseFeed(stdout, exitCode)
  if (parsed.status === "ok") return parsed
  var keptItems = previous && Array.isArray(previous.items) ? previous.items : []
  return { status: parsed.status, items: keptItems }
}

function barText(state) {
  var status = state ? state.status : "loading"
  if (status === "error" || status === "incompatible") return ALERT_GLYPH
  var label = status === "ok" ? barLabel(state.items.length) : ""
  return label === "" ? FEED_GLYPH : FEED_GLYPH + " " + label
}

function errorMessage(status) {
  if (status === "error") return ERROR_MESSAGE
  if (status === "incompatible") return INCOMPATIBLE_MESSAGE
  return ""
}

// Only these links reach xdg-open.
function isOpenableUrl(url) {
  if (typeof url !== "string" || url.length > MAX_URL_LENGTH) return false
  // No whitespace or control characters anywhere, and no user name before
  // the host (https://bank.example@evil.example/).
  return /^https:\/\/[^\/?#@\s\u0000-\u001F\u007F-\u009F]+([\/?#][^\s\u0000-\u001F\u007F-\u009F]*)?$/.test(url)
}

// A fresh object with only the known fields, so nothing else from the JSON
// (such as a "__proto__" key) travels further.
function cleanItem(raw) {
  if (!isPlainObject(raw)) return null
  if (typeof raw.id !== "string" || raw.id === "") return null
  if (typeof raw.title !== "string" || raw.title === "") return null
  if (CATEGORIES.indexOf(raw.category) === -1) return null
  if (!isOpenableUrl(raw.url)) return null
  return {
    id: raw.id,
    category: raw.category,
    title: truncate(raw.title, MAX_TITLE_CHARS),
    url: raw.url,
    source: typeof raw.source === "string" ? truncate(raw.source, MAX_SOURCE_CHARS) : ""
  }
}

// Counts code points, like the binary, so an emoji is never split in half.
function truncate(text, maxChars) {
  var characters = Array.from(text)
  if (characters.length <= maxChars) return text
  return characters.slice(0, maxChars - 1).join("") + "…"
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value)
}
