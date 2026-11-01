import QtQuick
import qs.Ui
import "Model.js" as Model

// Walking-skeleton widget: an icon and, later, the count of new critical items.
// WidgetButton renders its text with Text.PlainText.
BarWidget {
  id: root
  moduleName: "davifelype.thedailybugle"

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    // Nothing is fetched yet (phase 2), so the count is always zero.
    text: "" + Model.barLabel(0)
    horizontalMargin: 7.5
  }
}
