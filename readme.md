# Rusty Bridge
### CLI Alternative to [VBridger](https://store.steampowered.com/app/1898830/VBridger/ "VBridger")

Mostly made out of interest using Rust

## Usage
1. Run VTubeStudio on PC and IPhone
2. Run this tool `rusty-bridge.exe <ip> <config_file>`
3. Allow connection in VTubeStudio
4. Done

Run params:
`ip` - IPhone ipv4 local addres
`config_file` - path to config .json file

Example:
`rusty-bridge.exe 192.168.0.174 test.json`

Config File Example:
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