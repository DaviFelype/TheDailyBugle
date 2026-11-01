.pragma library

// Pure display logic for the bar widget, kept out of QML so node can test it
// (tests/js). No QML or Quickshell APIs here.

// The bar has room for two digits; anything above reads as "a lot".
var MAX_SHOWN_COUNT = 99

function barLabel(count) {
  if (typeof count !== "number" || !Number.isInteger(count) || count <= 0) return ""
  return count > MAX_SHOWN_COUNT ? MAX_SHOWN_COUNT + "+" : String(count)
}
