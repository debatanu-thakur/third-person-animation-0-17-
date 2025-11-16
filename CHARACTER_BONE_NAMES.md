# Character Bone Names Data

This document stores the character's bone hierarchy for reference when implementing animation sampling and IK.

## How to Generate

Press 'O' during gameplay to dump complete bone hierarchy to `assets/bones/character_bones.ron`.

## Bone Structure (from brian_parkour.glb)

```
brian (root)
├── Ch01_Body
├── Ch01_Eyelashes
├── Ch01_Pants
├── Ch01_Shirt
├── Ch01_Sneakers
└── mixamorig12:Hips
    ├── mixamorig12:Spine
    │   ├── mixamorig12:Spine1
    │   │   ├── mixamorig12:Spine2
    │   │   │   ├── mixamorig12:Neck
    │   │   │   │   └── mixamorig12:Head
    │   │   │   │       └── mixamorig12:HeadTop_End
    │   │   │   ├── mixamorig12:LeftShoulder
    │   │   │   │   └── mixamorig12:LeftArm
    │   │   │   │       └── mixamorig12:LeftForeArm
    │   │   │   │           └── mixamorig12:LeftHand
    │   │   │   │               ├── mixamorig12:LeftHandThumb1
    │   │   │   │               ├── mixamorig12:LeftHandIndex1
    │   │   │   │               ├── mixamorig12:LeftHandMiddle1
    │   │   │   │               ├── mixamorig12:LeftHandRing1
    │   │   │   │               └── mixamorig12:LeftHandPinky1
    │   │   │   └── mixamorig12:RightShoulder
    │   │   │       └── mixamorig12:RightArm
    │   │   │           └── mixamorig12:RightForeArm
    │   │   │               └── mixamorig12:RightHand
    │   │   │                   ├── mixamorig12:RightHandThumb1
    │   │   │                   ├── mixamorig12:RightHandIndex1
    │   │   │                   ├── mixamorig12:RightHandMiddle1
    │   │   │                   ├── mixamorig12:RightHandRing1
    │   │   │                   └── mixamorig12:RightHandPinky1
    ├── mixamorig12:LeftUpLeg
    │   └── mixamorig12:LeftLeg
    │       └── mixamorig12:LeftFoot
    │           └── mixamorig12:LeftToeBase
    │               └── mixamorig12:LeftToe_End
    └── mixamorig12:RightUpLeg
        └── mixamorig12:RightLeg
            └── mixamorig12:RightFoot
                └── mixamorig12:RightToeBase
                    └── mixamorig12:RightToe_End
```

## Key Bones for IK

### Hands (for vaulting/climbing)
- `mixamorig12:LeftHand`
- `mixamorig12:RightHand`
- IK Chain: Hand → ForeArm → Arm (chain_length: 2)

### Feet (for landing/stepping)
- `mixamorig12:LeftFoot`
- `mixamorig12:RightFoot`
- IK Chain: Foot → Leg → UpLeg (chain_length: 2)

### Body Reference
- `mixamorig12:Hips` - Root of skeleton
- `mixamorig12:Spine`, `Spine1`, `Spine2` - Torso bending

## Total Bones

- **65 mixamorig bones** - Animated skeleton
- **5 mesh nodes** - Visual geometry (Ch01_Body, etc.)
- **1 root** - "brian" (must match animation root!)

## Animation Compatibility

For animations to work:
1. Root node must be named "brian" (not "Armature")
2. All bones must use "mixamorig12:" prefix
3. Entity hierarchy paths must match exactly

See `ANIMATION_RETARGETING.md` for details on fixing mismatches.
