import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    visible: true
    width: 1120
    height: 720
    minimumWidth: 920
    minimumHeight: 620
    title: "LP-0016 Anonymous Forum"

    property int step: 0
    property var flow: [
        "Forum",
        "Register",
        "Post",
        "Moderate",
        "Vote",
        "Certificate",
        "History",
        "Slash",
        "Rejected"
    ]
    property var status: [
        "forum-a  K=2  N=2/3",
        "member 23047ffd active",
        "post 6ebad72b accepted",
        "post 600e8867 queued",
        "alice+bob signed",
        "share x=245196029927562062",
        "2 strikes ready",
        "commitment revoked",
        "post after slash rejected"
    ]

    color: "#f6f7f9"

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 248
            color: "#17202a"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 18
                spacing: 16

                Label {
                    text: "LP-0016"
                    color: "#ffffff"
                    font.pixelSize: 24
                    font.bold: true
                }

                Repeater {
                    model: window.flow
                    delegate: Button {
                        Layout.fillWidth: true
                        text: (index + 1) + ". " + modelData
                        highlighted: window.step === index
                        onClicked: window.step = index
                    }
                }

                Item { Layout.fillHeight: true }

                Label {
                    Layout.fillWidth: true
                    text: window.status[window.step]
                    color: "#d8dee6"
                    wrapMode: Text.WordWrap
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 76
                color: "#ffffff"
                border.color: "#dde2e8"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 24
                    anchors.rightMargin: 24

                    Label {
                        text: window.flow[window.step]
                        font.pixelSize: 26
                        font.bold: true
                        color: "#17202a"
                    }

                    Item { Layout.fillWidth: true }

                    Button {
                        text: "Back"
                        enabled: window.step > 0
                        onClicked: window.step = Math.max(0, window.step - 1)
                    }

                    Button {
                        text: window.step === window.flow.length - 1 ? "Reset" : "Next"
                        onClicked: window.step = window.step === window.flow.length - 1 ? 0 : window.step + 1
                    }
                }
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: window.step

                FlowScreen {
                    title: "forum-a"
                    primary: "K=2 strikes"
                    secondary: "N=2 of 3 moderators"
                    rows: ["threshold key 9b31d2aa", "membership root empty", "revocation root empty"]
                    action: "Create forum"
                }
                FlowScreen {
                    title: "member registration"
                    primary: "23047ffd"
                    secondary: "stake locked"
                    rows: ["membership root c4a7e992", "revocation root empty", "status active"]
                    action: "Register"
                }
                FlowScreen {
                    title: "anonymous post"
                    primary: "6ebad72b"
                    secondary: "receipt bound to roots"
                    rows: ["ciphertext 2f891a0c", "retro tag 41ca09e1", "share commitment 07db9130"]
                    action: "Publish"
                }
                FlowScreen {
                    title: "moderator dashboard"
                    primary: "600e8867"
                    secondary: "rule review"
                    rows: ["proof hash 91df44a8", "ciphertext hash 390e0afd", "reason spam"]
                    action: "Queue vote"
                }
                FlowScreen {
                    title: "moderation vote"
                    primary: "alice + bob"
                    secondary: "statement signed"
                    rows: ["mod set v1", "threshold key 9b31d2aa", "2 distinct signatures"]
                    action: "Cast vote"
                }
                FlowScreen {
                    title: "certificate"
                    primary: "N reached"
                    secondary: "DLEQ partials verified"
                    rows: ["share x=245196029927562062", "share y hidden until slash", "certificate stored"]
                    action: "Aggregate"
                }
                FlowScreen {
                    title: "history"
                    primary: "2 certificates"
                    secondary: "same hidden polynomial"
                    rows: ["600e8867 strike 1", "0f7f6f28 strike 2", "bundle ready"]
                    action: "Open bundle"
                }
                FlowScreen {
                    title: "slash"
                    primary: "23047ffd"
                    secondary: "commitment reconstructed"
                    rows: ["registry active before slash", "revocation root advanced", "stake claim recorded"]
                    action: "Submit slash"
                }
                FlowScreen {
                    title: "post rejected"
                    primary: "member revoked"
                    secondary: "receipt cannot prove non-membership"
                    rows: ["membership root current", "revocation root includes 23047ffd", "nullifier not consumed"]
                    action: "Review history"
                }
            }
        }
    }

    component FlowScreen: Rectangle {
        property string title
        property string primary
        property string secondary
        property var rows: []
        property string action

        color: "#f6f7f9"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 28
            spacing: 18

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 184
                radius: 6
                color: "#ffffff"
                border.color: "#dde2e8"

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 22
                    spacing: 8

                    Label {
                        text: title
                        color: "#5b6673"
                        font.pixelSize: 14
                    }
                    Label {
                        text: primary
                        color: "#17202a"
                        font.pixelSize: 34
                        font.bold: true
                    }
                    Label {
                        text: secondary
                        color: "#334155"
                        font.pixelSize: 18
                    }
                    Item { Layout.fillHeight: true }
                    Button {
                        text: action
                        Layout.preferredWidth: 180
                    }
                }
            }

            Repeater {
                model: rows
                delegate: Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 54
                    radius: 6
                    color: "#ffffff"
                    border.color: "#dde2e8"

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 18
                        anchors.rightMargin: 18

                        Rectangle {
                            Layout.preferredWidth: 10
                            Layout.preferredHeight: 10
                            radius: 5
                            color: "#1f8a70"
                        }
                        Label {
                            Layout.fillWidth: true
                            text: modelData
                            color: "#17202a"
                            font.pixelSize: 16
                        }
                    }
                }
            }

            Item { Layout.fillHeight: true }
        }
    }
}
