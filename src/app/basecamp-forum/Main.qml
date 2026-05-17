import QtQuick
import QtQuick.Controls

ApplicationWindow {
    visible: true
    width: 960
    height: 640
    title: "LP-0016 Anonymous Forum Demo"

    Column {
        anchors.centerIn: parent
        spacing: 16

        Text {
            text: "LP-0016 Anonymous Forum"
            font.pixelSize: 28
        }

        Text {
            width: 720
            wrapMode: Text.WordWrap
            text: "This is the Basecamp placeholder. Wire these buttons to the moderation SDK module: create forum, register, post, moderate, aggregate certificate, submit slash, and show rejected post after revocation."
        }

        Row {
            spacing: 8
            Button { text: "Create forum" }
            Button { text: "Register" }
            Button { text: "Post" }
            Button { text: "Moderate" }
            Button { text: "Slash" }
        }
    }
}
