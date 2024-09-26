# Rusty Bridge

Receive tracking data from [VTubeStudio](https://github.com/DenchiSoft/VTubeStudio) on IPhone then modify data according to config and send to [VTubeStudio](https://github.com/DenchiSoft/VTubeStudio) on PC

Basically it's alternative to [VBridger](https://store.steampowered.com/app/1898830/VBridger/)

## Usage

There 2 ways to use it
CLI and UI

### UI

Launch `rusty-bridge-ui.exe`  
Set path to config file (type it or use button)  
Type **Local** IPhone Ip  
Press Connect (Now you can close window)

> [!TIP]
> To close or exit use menu in tray

### CLI

For CLI use `rusty-bridge.exe` with launch params

#### Params:

| Command                               | Example              | Description         |
| ------------------------------------- | -------------------- | ------------------- |
| `-t <path>`, `--transform-cfg <path>` | `-t test.json`       | Path to config file |
| `-p <IPv4>`, `--phone-ip <IPv4>`      | `-p "192.168.0.174"` | Local Iphone Ip     |
| `-h `, `--help`                       | `-h`                 | Show Help           |
| `-V `, `--version`                    | `-V`                 | Show Version        |

## Transform config

Json file where you define transformations and new Params.

There list of params send from IPhone

#### Cords: - ranged from negative to positive probably won't out of -45...45

```
HeadRotX, HeadRotY, HeadRotZ
HeadPosX, HeadPosY, HeadPosZ
```

#### BlendShapes: - ranged from 0 to 1

```
BrowDownLeft
BrowDownRight
BrowInnerUp
BrowOuterUpLeft
BrowOuterUpRight

CheekPuff
CheekSquintLeft
CheekSquintRight

EyeBlinkLeft
EyeBlinkRight
EyeLookDownLeft
EyeLookDownRight
EyeLookInLeft
EyeLookInRight
EyeLookOutLeft
EyeLookOutRight
EyeLookUpLeft
EyeLookUpRight
EyeSquintLeft
EyeSquintRight
EyeWideLeft
EyeWideRight

JawForward
JawLeft
JawOpen
JawRight

MouthClose
MouthDimpleLeft
MouthDimpleRight
MouthFrownLeft
MouthFrownRight
MouthFunnel
MouthLeft
MouthLowerDownLeft
MouthLowerDownRight
MouthPressLeft
MouthPressRight
MouthPucker
MouthRight
MouthRollLower
MouthRollUpper
MouthShrugLower
MouthShrugUpper
MouthSmileLeft
MouthSmileRight
MouthStretchLeft
MouthStretchRight
MouthUpperUpLeft
MouthUpperUpRight

NoseSneerLeft
NoseSneerRight

TongueOut
```

### Example:

```json
[
  {
    "name": "FaceAngleY",
    "func": "- HeadRotY * 1",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "FaceAngleX",
    "func": "(((HeadRotX * ((90 - math::abs(HeadRotY)) / 90)) + (HeadRotZ * (HeadRotY / 45))))",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "FaceAngleZ",
    "func": "((HeadRotZ * ((90 - math::abs(HeadRotY)) / 90)) - (HeadRotX * (HeadRotY / 45)))",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "FacePositionX",
    "func": "HeadPosX * - 1",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  },
  {
    "name": "FacePositionY",
    "func": "HeadPosY",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  },
  {
    "name": "FacePositionZ",
    "func": "HeadPosZ",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  },
  {
    "name": "MouthOpen",
    "func": "(((JawOpen - MouthClose) - ((MouthRollUpper + MouthRollLower) * .2) + (MouthFunnel * .2)))",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeRightX",
    "func": "(EyeLookInLeft - .1) - EyeLookOutLeft",
    "min": -1.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeRightY",
    "func": "(EyeLookUpLeft - EyeLookDownLeft) + (BrowOuterUpLeft * .15) + (HeadRotX / 30)",
    "min": -1.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeOpenLeft",
    "func": ".5 + ((EyeBlinkLeft * - .8) + (EyeWideLeft * .8))",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeOpenRight",
    "func": ".5 + ((EyeBlinkRight * - .8) + (EyeWideRight * .8))",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "MouthSmile",
    "func": "(2 - ((MouthFrownLeft + MouthFrownRight + MouthPucker) / 1) + ((MouthSmileRight + MouthSmileLeft + ((MouthDimpleLeft + MouthDimpleRight) / 2)) / 1)) / 4",
    "min": -1.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeSquintL",
    "func": "EyeSquintLeft",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "EyeSquintR",
    "func": "EyeSquintRight",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "MouthX",
    "func": "(((MouthLeft - MouthRight) + (MouthSmileLeft - MouthSmileRight)) * (1 - TongueOut))",
    "min": -1.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "CheekPuff",
    "func": "CheekPuff",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "TongueOut",
    "func": "TongueOut",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "MouthPucker",
    "func": "(((MouthDimpleRight + MouthDimpleLeft) * 2) - MouthPucker) * (1 - TongueOut)",
    "min": -1.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "MouthFunnel",
    "func": "(MouthFunnel * (1 - TongueOut)) - (JawOpen * .2)",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "JawOpen",
    "func": "JawOpen",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "MouthPressLipOpen",
    "func": "(((MouthUpperUpRight + MouthUpperUpLeft + MouthLowerDownRight + MouthLowerDownLeft) / 1.8) - (MouthRollLower + MouthRollUpper)) * (1 - TongueOut)",
    "min": -1.3,
    "max": 1.3,
    "defaultValue": 0
  },
  {
    "name": "MouthShrug",
    "func": "((MouthShrugUpper + MouthShrugLower + MouthPressRight + MouthPressLeft) / 4) * (1 - TongueOut)",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "BrowInnerUp",
    "func": "BrowInnerUp",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "BrowLeftY",
    "func": ".5 + (BrowOuterUpLeft - BrowDownLeft) + ((MouthRight - MouthLeft) / 8)",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "BrowRightY",
    "func": ".5 + (BrowOuterUpRight - BrowDownRight) + ((MouthLeft - MouthRight) / 8)",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0
  },
  {
    "name": "Brows",
    "func": ".5 + (BrowOuterUpRight + BrowOuterUpLeft - BrowDownLeft - BrowDownRight) / 4",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0.5
  },
  {
    "name": "VoiceFrequencyPlusMouthSmile",
    "func": "(2 - ((MouthFrownLeft + MouthFrownRight + MouthPucker) / 1) + ((MouthSmileRight + MouthSmileLeft + ((MouthDimpleLeft + MouthDimpleRight) / 2)) / 1)) / 4",
    "min": 0.0,
    "max": 1.0,
    "defaultValue": 0.5
  },
  {
    "name": "BodyAngleX",
    "func": "- HeadRotY * 1.5",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "BodyAngleY",
    "func": "( - HeadRotX * 1.5)  + ( (EyeBlinkLeft + EyeBlinkRight) * - 1)",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "BodyAngleZ",
    "func": "HeadRotZ * 1.5",
    "min": -40.0,
    "max": 40.0,
    "defaultValue": 0
  },
  {
    "name": "BodyPositionX",
    "func": "HeadPosX * - 1",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  },
  {
    "name": "BodyPositionY",
    "func": "HeadPosY * 1",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  },
  {
    "name": "BodyPositionZ",
    "func": "HeadPosZ * - .5",
    "min": -15.0,
    "max": 15.0,
    "defaultValue": 0
  }
]
```
