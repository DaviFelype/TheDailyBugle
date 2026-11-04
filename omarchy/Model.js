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
var MAX_POC_COUNT = 10000
var MAX_FAILED_SOURCES = 10
var PUBLISHED_AT_PATTERN = /^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$/
var DETAIL_SEPARATOR = " · "
var FAILURE_PREFIX = "Could not update: "

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
  return { status: "loading", items: [], failedSources: [] }
}

// Reads one run of `thedailybugle feed --json`. `exitCode` is null when the
// binary never started. Status is "ok", "error" or "incompatible".
function parseFeed(stdout, exitCode) {
  if (exitCode !== 0 || typeof stdout !== "string") return { status: "error", items: [], failedSources: [] }

  var document
  try {
    document = JSON.parse(stdout)
  } catch (error) {
    return { status: "error", items: [], failedSources: [] }
  }
  if (!isPlainObject(document)) return { status: "error", items: [], failedSources: [] }
  if (typeof document.schemaVersion === "number" && document.schemaVersion !== SCHEMA_VERSION)
    return { status: "incompatible", items: [], failedSources: [] }
  if (document.schemaVersion !== SCHEMA_VERSION || !Array.isArray(document.items))
    return { status: "error", items: [], failedSources: [] }

  var items = []
  for (var index = 0; index < document.items.length && items.length < MAX_ITEMS; index++) {
    var item = cleanItem(document.items[index])
    if (item) items.push(item)
  }
  return { status: "ok", items: items, failedSources: cleanFailedSources(document.failedSources) }
}

// The state after a run. A failed run keeps the last good items on screen.
function nextState(previous, stdout, exitCode) {
  var parsed = parseFeed(stdout, exitCode)
  if (parsed.status === "ok") return parsed
  var keptItems = previous && Array.isArray(previous.items) ? previous.items : []
  return { status: parsed.status, items: keptItems, failedSources: [] }
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

// The line under the title: "CISA KEV · 2026-11-03 · 3 PoC · ransomware",
// each part only when the item has it.
function itemDetail(item) {
  if (!isPlainObject(item)) return ""
  var parts = []
  if (typeof item.source === "string" && item.source !== "") parts.push(item.source)
  if (typeof item.publishedAt === "string") parts.push(item.publishedAt)
  if (typeof item.pocCount === "number") parts.push(item.pocCount + " PoC")
  if (item.knownRansomwareUse === true) parts.push("ransomware")
  return parts.join(DETAIL_SEPARATOR)
}

// Shown above the items when some source could not be updated. The bar icon
// stays as it is: the other items are still good.
function failureNotice(state) {
  if (!state || !Array.isArray(state.failedSources) || state.failedSources.length === 0) return ""
  return FAILURE_PREFIX + state.failedSources.join(", ")
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
  var item = {
    id: raw.id,
    category: raw.category,
    title: truncate(raw.title, MAX_TITLE_CHARS),
    url: raw.url,
    source: typeof raw.source === "string" ? truncate(raw.source, MAX_SOURCE_CHARS) : ""
  }
  // Optional fields: kept only when valid, left out otherwise.
  if (typeof raw.publishedAt === "string" && PUBLISHED_AT_PATTERN.test(raw.publishedAt))
    item.publishedAt = raw.publishedAt
  if (Number.isInteger(raw.pocCount) && raw.pocCount >= 0 && raw.pocCount <= MAX_POC_COUNT)
    item.pocCount = raw.pocCount
  if (typeof raw.knownRansomwareUse === "boolean") item.knownRansomwareUse = raw.knownRansomwareUse
  return item
}

// At most 10 non-empty names, each cut like a source.
function cleanFailedSources(raw) {
  if (!Array.isArray(raw)) return []
  var names = []
  for (var index = 0; index < raw.length && names.length < MAX_FAILED_SOURCES; index++) {
    if (typeof raw[index] === "string" && raw[index] !== "")
      names.push(truncate(raw[index], MAX_SOURCE_CHARS))
  }
  return names
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
